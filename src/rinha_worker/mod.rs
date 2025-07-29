use crate::rinha_domain::{HOST, Payment, Target, TargetCounter};
use crate::rinha_http::JSON_CONTENT_TYPE;
use async_trait::async_trait;
use http::{Method, header};
use pingora::connectors::http::Connector;
use pingora::http::RequestHeader;
use pingora::lb::LoadBalancer;
use pingora::prelude::{HttpPeer, RoundRobin};
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;
use tokio::sync::mpsc;

pub static TARGET_COUNTER: LazyLock<RwLock<TargetCounter>> =
    LazyLock::new(|| RwLock::new(TargetCounter::default()));

pub struct RinhaWorker {
    receiver: RwLock<mpsc::Receiver<Payment>>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
}

impl RinhaWorker {
    fn new(
        receiver: mpsc::Receiver<Payment>,
        load_balancer: Arc<LoadBalancer<RoundRobin>>,
    ) -> Self {
        Self {
            receiver: RwLock::new(receiver),
            load_balancer: load_balancer,
        }
    }
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let mut receiver = self.receiver.write().await;

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                Some(payment) = receiver.recv() => {
                    process_payment(payment, Arc::clone(&self.load_balancer)).await
                }
            }
        }
    }
}

async fn process_payment(payment: Payment, load_balancer: Arc<LoadBalancer<RoundRobin>>) {
    let Some(backend) = load_balancer.select(b"", 8) else {
        log::error!("process_payment: no backend found");
        return;
    };
    let Some(target) = backend.ext.get::<Target>() else {
        log::error!("process_payment: failed to get Target backend ext");
        return;
    };

    let peer = HttpPeer::new(backend.addr.clone(), false, backend.addr.to_string());
    let connector = Connector::new(None);

    let Ok((mut http, _)) = connector.get_http_session(&peer).await else {
        log::error!("process_payment: failed to get http session");
        return;
    };

    let Ok(payment_ser) = serde_json::ser::to_vec(&payment) else {
        log::error!("process_payment: failed to serialize payment struct");
        return;
    };

    let Ok(mut request_header) = RequestHeader::build(Method::POST, b"/payments", None) else {
        log::error!("process_payment: failed to build request header");
        return;
    };

    if let Err(_) = request_header
        .append_header(header::HOST, HOST.as_str())
        .and(request_header.append_header(header::CONTENT_LENGTH, payment_ser.len()))
        .and(request_header.append_header(header::CONTENT_TYPE, JSON_CONTENT_TYPE))
    {
        log::error!("process_payment: failed to write request headers");
        return;
    };

    if let Err(_) = http
        .write_request_header(Box::new(request_header))
        .await
        .and(http.write_request_body(payment_ser.into(), true).await)
        .and(http.finish_request_body().await)
    {
        log::error!("process_payment: failed to send request");
        return;
    };

    if let Err(_) = http.read_response_header().await {
        log::error!("process_payment: failed to read header");
        return;
    }

    let Some(response_header) = http.response_header() else {
        log::error!("process_payment: fail while reading response header");
        return;
    };

    if !response_header.status.is_success() {
        log::error!("process_payment: non-200 status code");
        return;
    }

    match target {
        Target::Default => {
            let mut counter = TARGET_COUNTER.write().await;
            counter.default.requests += 1;
            counter.default.amount += payment.amount;
        }
        Target::Fallback => {
            let mut counter = TARGET_COUNTER.write().await;
            counter.fallback.requests += 1;
            counter.fallback.amount += payment.amount;
        }
    }
}

pub fn rinha_worker_service(
    receiver: mpsc::Receiver<Payment>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Worker Background Service".into(),
        Arc::new(RinhaWorker::new(receiver, load_balancer)),
    )
}
