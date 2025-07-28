use crate::rinha_domain::Payment;
use async_trait::async_trait;
use http::{Method, header};
use pingora::connectors::http::Connector;
use pingora::http::RequestHeader;
use pingora::lb::LoadBalancer;
use pingora::prelude::{HttpPeer, RoundRobin};
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;

pub struct RinhaWorker {
    receiver: Mutex<Receiver<Payment>>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
}

impl RinhaWorker {
    fn new(receiver: Receiver<Payment>, load_balancer: Arc<LoadBalancer<RoundRobin>>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            load_balancer: load_balancer,
        }
    }
}

impl RinhaWorker {
    async fn process_payment(&self, payment: Payment) {
        let load_balancer = Arc::clone(&self.load_balancer);
        let backend = load_balancer.select(b"", 8).unwrap();

        let peer = HttpPeer::new(backend.addr.clone(), false, backend.addr.to_string());
        let connector = Connector::new(None);
        let (mut http, _) = connector.get_http_session(&peer).await.unwrap();

        let payment = serde_json::ser::to_vec(&payment).unwrap();

        let mut request_header = RequestHeader::build(Method::POST, b"/payments", None).unwrap();

        request_header
            .append_header(header::HOST, "0.0.0.0:9999")
            .unwrap();
        request_header
            .append_header(header::CONTENT_LENGTH, payment.len())
            .unwrap();
        request_header
            .append_header(header::CONTENT_TYPE, "application/json")
            .unwrap();

        http.write_request_header(Box::new(request_header))
            .await
            .unwrap();
        http.write_request_body(payment.into(), true).await.unwrap();
        http.finish_request_body().await.unwrap();

        http.read_response_header().await.unwrap();

        dbg!(http.response_header());
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
                Some(payment) = receiver.recv() => {
                    self.process_payment(payment).await;
                }
            }
        }
    }
}

pub fn rinha_worker_service(
    receiver: Receiver<Payment>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Worker Background Service".to_string(),
        Arc::new(RinhaWorker::new(receiver, load_balancer)),
    )
}
