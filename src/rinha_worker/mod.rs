use crate::rinha_domain::Payment;
use async_trait::async_trait;
use pingora::lb::LoadBalancer;
use pingora::prelude::RoundRobin;
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

        dbg!(payment, load_balancer.select(b"", 256));
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
