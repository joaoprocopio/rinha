use std::sync::Arc;

use pingora::services::background::{BackgroundService, GenBackgroundService};
use tokio::sync::broadcast;

use async_trait::async_trait;
use pingora::server::ShutdownWatch;

use crate::rinha_domain::Payment;

struct RinhaWorker {
    receiver: broadcast::Receiver<Payment>,
}

impl RinhaWorker {
    fn new(receiver: broadcast::Receiver<Payment>) -> Self {
        Self { receiver: receiver }
    }
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let mut receiver = self.receiver.resubscribe();

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                recv = receiver.recv() => {
                    if let Ok(payment) = recv {
                        dbg!(payment);
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

pub fn rinha_worker_service(
    receiver: broadcast::Receiver<Payment>,
) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Background Service".to_string(),
        Arc::new(RinhaWorker::new(receiver)),
    )
}
