use crate::{
    rinha_conf::RINHA_HOST,
    rinha_domain::{Payment, PaymentRequest, Target},
    rinha_http::JSON_CONTENT_TYPE,
    rinha_storage, rinha_tracing,
};
use async_trait::async_trait;
use http::{Method, header};
use pingora::http::RequestHeader;
use pingora::lb::LoadBalancer;
use pingora::prelude::{HttpPeer, RoundRobin};
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use pingora::{
    connectors::{ConnectorOptions, http::Connector},
    server::configuration::ServerConf,
};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub struct RinhaWorker {
    receiver: Mutex<mpsc::Receiver<PaymentRequest>>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
    connector: Arc<Connector>,
}

impl RinhaWorker {
    fn new(
        receiver: mpsc::Receiver<PaymentRequest>,
        load_balancer: Arc<LoadBalancer<RoundRobin>>,
        connector: Connector,
    ) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            load_balancer: load_balancer,
            connector: Arc::new(connector),
        }
    }

    async fn process_payment(&self, payment_request: PaymentRequest) {
        let load_balancer = self.load_balancer.clone();

        let Some(backend) = load_balancer.select(b"", 8) else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "no backend found"
            );
            return;
        };
        let Some(target) = backend.ext.get::<Target>().cloned() else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to get Target backend ext"
            );
            return;
        };

        let peer = HttpPeer::new(backend.addr.to_string(), false, backend.addr.to_string());
        let connector = self.connector.clone();

        let Ok((mut http, _)) = connector.get_http_session(&peer).await else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to get http session"
            );
            return;
        };

        let Ok(payment_ser) = serde_json::ser::to_vec(&payment_request) else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to serialize payment struct"
            );
            return;
        };

        let Ok(mut request_header) = RequestHeader::build(Method::POST, b"/payments", None) else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to build request header"
            );
            return;
        };

        if request_header
            .append_header(header::HOST, RINHA_HOST.as_str())
            .and(request_header.append_header(header::CONTENT_LENGTH, payment_ser.len()))
            .and(request_header.append_header(header::CONTENT_TYPE, JSON_CONTENT_TYPE))
            .is_err()
        {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to write request headers"
            );
            return;
        };

        if http
            .write_request_header(Box::new(request_header))
            .await
            .and(http.write_request_body(payment_ser.into(), true).await)
            .and(http.finish_request_body().await)
            .is_err()
        {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to send request"
            );
            return;
        };

        if http.read_response_header().await.is_err() {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "failed to read header"
            );
            return;
        }

        let Some(response_header) = http.response_header() else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "fail while reading response header"
            );
            return;
        };

        if !response_header.status.is_success() {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "non-200 status code"
            );
            return;
        }

        let Ok(Some(_)) = pingora_runtime::current_handle()
            .spawn_blocking(move || -> Option<()> {
                let storage = rinha_storage::get_handle();

                // Create payment record with target information for database storage
                let payment = Payment {
                    correlation_id: payment_request.correlation_id,
                    amount: payment_request.amount,
                    requested_at: payment_request.requested_at,
                    target: target,
                };

                let (key, value): (fjall::Slice, fjall::Slice) =
                    match ((&payment.requested_at).try_into(), (&payment).try_into()) {
                        (Ok(key), Ok(value)) => (key, value),
                        _ => return None,
                    };

                if storage.insert(key, value).is_err() {
                    return None;
                };

                Some(())
            })
            .await
        else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::process_payment),
                "error while inserting to storage"
            );
            return;
        };
    }
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let mut receiver = self.receiver.lock().await;

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                Some(payment_request) = receiver.recv() => {
                    self.process_payment(payment_request).await
                }
            }
        }
    }
}

pub fn rinha_worker_service(
    receiver: mpsc::Receiver<PaymentRequest>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
    server_configuration: Arc<ServerConf>,
) -> GenBackgroundService<RinhaWorker> {
    let connector_options = ConnectorOptions::from_server_conf(&server_configuration);
    let connector = Connector::new(Some(connector_options));

    GenBackgroundService::new(
        "Rinha Worker Background Service".into(),
        Arc::new(RinhaWorker::new(receiver, load_balancer, connector)),
    )
}
