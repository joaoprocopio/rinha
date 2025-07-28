use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    lb::{
        Backend,
        health_check::{HealthCheck, HttpHealthCheck},
    },
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
        let mut period = interval(Duration::from_millis(5000));

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                _ = period.tick() => {
                    health_check().await;
                }
            }
        }
    }
}

async fn health_check() {
    let default_backend = Backend::new_with_weight("0.0.0.0:8001", 10).unwrap();
    let fallback_backend = Backend::new_with_weight("0.0.0.0:8002", 1).unwrap();

    let mut hc = HttpHealthCheck::new("1.1.1.1", false);

    hc.req = RequestHeader::build("GET", b"/payments/service-health", None).unwrap();

    hc.validator = Some(Box::new(|header: &ResponseHeader| {
        if header.status == 200 {
            Ok(())
        } else {
            Err(pingora::Error::create(
                pingora::ErrorType::ConnectError,
                pingora::ErrorSource::Upstream,
                None,
                None,
            ))
        }
    }));

    let _ = tokio::join!(hc.check(&default_backend), hc.check(&fallback_backend));
}

pub fn rinha_ambulance_service() -> GenBackgroundService<RinhaAmbulance> {
    GenBackgroundService::new(
        "Rinha Ambulance Background Service".to_string(),
        Arc::new(RinhaAmbulance::new()),
    )
}
