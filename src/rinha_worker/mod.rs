use crate::rinha_domain::Payment;
use async_trait::async_trait;
use pingora::server::ShutdownWatch;
use pingora::services::background::{BackgroundService, GenBackgroundService};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;

pub struct RinhaWorker {
    receiver: Mutex<Receiver<Payment>>,
}

impl RinhaWorker {
    fn new(receiver: Receiver<Payment>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
        }
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
                    process_payment(payment);
                }
            }
        }
    }
}

fn process_payment(payment: Payment) {
    dbg!(payment);
}

pub fn rinha_worker_service(receiver: Receiver<Payment>) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Background Service".to_string(),
        Arc::new(RinhaWorker::new(receiver)),
    )
}
