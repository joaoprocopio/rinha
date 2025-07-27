use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use pingora::{
    http::ResponseHeader,
    lb::{Backend, health_check::HttpHealthCheck},
    server::ShutdownWatch,
    services::background::{BackgroundService, GenBackgroundService},
};
use tokio::time::interval;

pub struct RinhaAmbulance;

impl RinhaAmbulance {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BackgroundService for RinhaAmbulance {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let mut period = interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                tick = period.tick() => {
                    dbg!(tick);
                }
            }
        }
    }
}

fn validator(header: &ResponseHeader) -> Result<(), Box<pingora::Error>> {
    Ok(())
}

fn hc() {
    let mut hc = HttpHealthCheck::new("1.1.1.1", false);

    let v = Box::new(validator);

    hc.validator = Some(Box::new(&validator));

    let default_backend = Backend::new_with_weight("http://0.0.0.0:8001", 10).unwrap();
    let fallback_backend = Backend::new_with_weight("http://0.0.0.0:8002", 1).unwrap();
}

pub fn rinha_ambulance_service() -> GenBackgroundService<RinhaAmbulance> {
    GenBackgroundService::new(
        "Rinha Ambulance Background Service".to_string(),
        Arc::new(RinhaAmbulance::new()),
    )
}
