use std::{str::FromStr, sync::Arc, time::Duration};

use async_trait::async_trait;
use http::{StatusCode, Uri, Version};
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

    let mut checker = HttpHealthCheck::new("0.0.0.0", false);

    checker
        .req
        .set_uri(Uri::from_str("/payments/service-health").unwrap());

    checker.validator = Some(Box::new(|header: &ResponseHeader| match header.status {
        StatusCode::OK => Ok(()),
        _ => Err(pingora::Error::create(
            pingora::ErrorType::ConnectError,
            pingora::ErrorSource::Upstream,
            None,
            None,
        )),
    }));

    let res = tokio::join!(
        checker.check(&default_backend),
        checker.check(&fallback_backend)
    );
    dbg!(res);
}

pub fn rinha_ambulance_service() -> GenBackgroundService<RinhaAmbulance> {
    GenBackgroundService::new(
        "Rinha Ambulance Background Service".to_string(),
        Arc::new(RinhaAmbulance::new()),
    )
}
