use crate::rinha_domain::{Payment, Target, TargetCounter};
use crate::rinha_http::JSON_CONTENT_TYPE;
use async_trait::async_trait;
use http::{Method, header};
use pingora::connectors::http::Connector;
use pingora::http::RequestHeader;
use pingora::lb::LoadBalancer;
use pingora::prelude::{HttpPeer, RoundRobin};
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use std::result;
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
    let backend = match load_balancer.select(b"", 8) {
        Some(backend) => backend,
        _ => return,
    };
    let target = match backend.ext.get::<Target>() {
        Some(target) => target,
        _ => return,
    };

    let peer = HttpPeer::new(backend.addr.clone(), false, backend.addr.to_string());
    let connector = Connector::new(None);

    let (mut http, _) = match connector.get_http_session(&peer).await {
        Ok(session) => session,
        _ => return,
    };

    let payment_ser = match serde_json::ser::to_vec(&payment) {
        Ok(payment_ser) => payment_ser,
        _ => return,
    };

    let mut request_header = match RequestHeader::build(Method::POST, b"/payments", None) {
        Ok(request_header) => request_header,
        _ => return,
    };

    if let Err(_) = request_header
        .append_header(header::HOST, "0.0.0.0")
        .and(request_header.append_header(header::CONTENT_LENGTH, payment_ser.len()))
        .and(request_header.append_header(header::CONTENT_TYPE, JSON_CONTENT_TYPE))
    {
        return;
    };

    if let Err(_) = http
        .write_request_header(Box::new(request_header))
        .await
        .and(http.write_request_body(payment_ser.into(), true).await)
        .and(http.finish_request_body().await)
        .and(http.read_response_header().await)
    {
        return;
    };

    let Some(response_header) = http.response_header() else {
        return;
    };

    match (target, response_header.status.is_success()) {
        (Target::Default, true) => {
            let mut counter = TARGET_COUNTER.write().await;
            counter.default.requests += 1;
            counter.default.amount += payment.amount;
        }
        (Target::Fallback, true) => {
            let mut counter = TARGET_COUNTER.write().await;
            counter.fallback.requests += 1;
            counter.fallback.amount += payment.amount;
        }
        _ => (),
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
