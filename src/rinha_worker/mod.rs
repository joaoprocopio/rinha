use std::sync::Arc;

use pingora::services::background::{BackgroundService, GenBackgroundService};
use tokio::sync::mpsc::Receiver;

use async_trait::async_trait;
use pingora::server::ShutdownWatch;

use crate::rinha_domain::Payment;

pub struct RinhaWorker {
    receiver: Receiver<Payment>,
}

impl RinhaWorker {
    fn new(receiver: Receiver<Payment>) -> Self {
        Self { receiver: receiver }
    }
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let mut receiver = self.receiver;

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                recv = receiver.recv() => {
                    if let Some(payment) = recv {
                        dbg!(payment);
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

pub fn rinha_worker_service(receiver: Receiver<Payment>) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Background Service".to_string(),
        Arc::new(RinhaWorker::new(receiver)),
    )
}
