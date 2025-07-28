use crate::rinha_domain::{Payment, Target, TargetCounter};
use async_trait::async_trait;
use http::{Method, header};
use parking_lot::RwLock;
use pingora::connectors::http::Connector;
use pingora::http::RequestHeader;
use pingora::lb::LoadBalancer;
use pingora::prelude::{HttpPeer, RoundRobin};
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use std::sync::{Arc, LazyLock};
use tokio::sync::mpsc::Receiver;

pub static TARGET_COUNTER: LazyLock<RwLock<TargetCounter>> =
    LazyLock::new(|| RwLock::new(TargetCounter::default()));

pub struct RinhaWorker {
    receiver: RwLock<Receiver<Payment>>,
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
}

impl RinhaWorker {
    fn new(receiver: Receiver<Payment>, load_balancer: Arc<LoadBalancer<RoundRobin>>) -> Self {
        Self {
            receiver: RwLock::new(receiver),
            load_balancer: load_balancer,
        }
    }
}

impl RinhaWorker {
    async fn process_payment(&self, payment: Payment) {
        let load_balancer = Arc::clone(&self.load_balancer);
        let backend = load_balancer.select(b"", 8).unwrap();
        let target = backend.ext.get::<Target>().unwrap();

        let peer = HttpPeer::new(backend.addr.clone(), false, backend.addr.to_string());
        let connector = Connector::new(None);
        let (mut http, _) = connector.get_http_session(&peer).await.unwrap();

        let payment_ser = serde_json::ser::to_vec(&payment).unwrap();

        let mut request_header = RequestHeader::build(Method::POST, b"/payments", None).unwrap();

        request_header
            .append_header(header::HOST, "0.0.0.0")
            .unwrap();
        request_header
            .append_header(header::CONTENT_LENGTH, payment_ser.len())
            .unwrap();
        request_header
            .append_header(header::CONTENT_TYPE, "application/json")
            .unwrap();

        http.write_request_header(Box::new(request_header))
            .await
            .unwrap();
        http.write_request_body(payment_ser.into(), true)
            .await
            .unwrap();
        http.finish_request_body().await.unwrap();
        http.read_response_header().await.unwrap();

        let response_header = http.response_header().unwrap();

        match (target, response_header.status.is_success()) {
            (Target::Default, true) => {
                let mut counter = TARGET_COUNTER.write();
                counter.default.requests += 1;
                counter.default.amount += payment.amount;
            }
            (Target::Fallback, true) => {
                let mut counter = TARGET_COUNTER.write();
                counter.fallback.requests += 1;
                counter.fallback.amount += payment.amount;
            }
            _ => (),
        }
    }
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
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
        "Rinha Worker Background Service".into(),
        Arc::new(RinhaWorker::new(receiver, load_balancer)),
    )
}
