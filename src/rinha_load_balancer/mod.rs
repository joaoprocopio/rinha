use http::Uri;
use pingora::{
    http::ResponseHeader,
    lb::{Backend, Backends, LoadBalancer, discovery, health_check::HttpHealthCheck},
    prelude::RoundRobin,
    services::background::GenBackgroundService,
};
use std::{collections::BTreeSet, sync::Arc, time::Duration};

#[derive(Clone, Debug)]
pub enum Target {
    Default,
    Fallback,
}

fn http_health_check() -> HttpHealthCheck {
    let mut health_checker = HttpHealthCheck::new("0.0.0.0", false);

    health_checker
        .req
        .set_uri(Uri::from_static("/payments/service-health"));

    health_checker.validator = Some(Box::new(|header: &ResponseHeader| {
        match header.status.is_success() {
            true => Ok(()),
            false => Err(pingora::Error::create(
                pingora::ErrorType::ConnectError,
                pingora::ErrorSource::Upstream,
                None,
                None,
            )),
        }
    }));

    health_checker
}

pub fn rinha_load_balancer_service() -> GenBackgroundService<LoadBalancer<RoundRobin>> {
    let mut default_backend = Backend::new_with_weight("0.0.0.0:8001", 10).unwrap();
    default_backend.ext.insert(Target::Default);

    let mut fallback_backend = Backend::new_with_weight("0.0.0.0:8002", 1).unwrap();
    fallback_backend.ext.insert(Target::Fallback);

    let discovery = discovery::Static::new(BTreeSet::from([default_backend, fallback_backend]));
    let backends = Backends::new(discovery);

    let mut upstreams = LoadBalancer::<RoundRobin>::from_backends(backends);

    upstreams.set_health_check(Box::new(http_health_check()));
    upstreams.health_check_frequency = Some(Duration::from_secs(5));
    upstreams.parallel_health_check = true;
    upstreams.update_frequency = None;

    GenBackgroundService::new(
        "Rinha Worker Background Service".to_string(),
        Arc::new(upstreams),
    )
}
