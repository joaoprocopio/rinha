use crate::rinha_domain::Payment;
use async_trait::async_trait;
use http::header;
use pingora::connectors::http::v1::Connector;
use pingora::http::RequestHeader;
use pingora::lb::LoadBalancer;
use pingora::prelude::{HttpPeer, RoundRobin};
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use pingora::upstreams::peer::{PeerOptions, Scheme};
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
        let peer = HttpPeer {
            _address: backend.addr.clone(),
            sni: backend.addr.to_string(),
            scheme: Scheme::HTTP,
            proxy: None,
            client_cert_key: None,
            group_key: 0,
            options: PeerOptions::new(),
        };
        let connector = Connector::new(None);
        let (mut http, _) = connector.get_http_session(&peer).await.unwrap();
        let mut request_header = RequestHeader::build("POST", b"/payments", None).unwrap();
        let payment_serialized = &serde_json::ser::to_vec(&payment).unwrap()[..];
        request_header
            .append_header(header::HOST, backend.addr.to_string())
            .unwrap();
        request_header
            .append_header(header::CONTENT_LENGTH, payment_serialized.len())
            .unwrap();
        request_header
            .append_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        http.write_request_header(Box::new(request_header))
            .await
            .unwrap();
        http.write_body(payment_serialized).await.unwrap();

        let data = http.read_body_bytes().await.unwrap().unwrap();

        dbg!(data);
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
