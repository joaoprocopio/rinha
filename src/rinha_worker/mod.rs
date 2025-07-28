use crate::rinha_domain::Payment;
use async_trait::async_trait;
use pingora::lb::LoadBalancer;
use pingora::lb::selection::Consistent;
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;

pub struct RinhaWorker {
    receiver: Mutex<Receiver<Payment>>,
    load_balancer: Arc<LoadBalancer<Consistent>>,
}

impl RinhaWorker {
    fn new(receiver: Receiver<Payment>, load_balancer: Arc<LoadBalancer<Consistent>>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
            load_balancer: load_balancer,
        }
    }
}

impl RinhaWorker {
    fn process_payment(&self, payment: Payment) {
        dbg!(payment);
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
                    self.process_payment(payment);
                }
            }
        }
    }
}

pub fn rinha_worker_service(
    receiver: Receiver<Payment>,
    load_balancer: Arc<LoadBalancer<Consistent>>,
) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Worker Background Service".to_string(),
        Arc::new(RinhaWorker::new(receiver, load_balancer)),
    )
}
