use http::{StatusCode, Uri};
use pingora::{
    http::ResponseHeader,
    lb::{Backend, LoadBalancer, health_check::HttpHealthCheck, selection::Consistent},
    services::background::GenBackgroundService,
};
use std::{str::FromStr, sync::Arc, time::Duration};

fn http_health_check() -> HttpHealthCheck {
    let mut health_checker = HttpHealthCheck::new("0.0.0.0", false);

    health_checker
        .req
        .set_uri(Uri::from_str("/payments/service-health").unwrap());

    health_checker.validator = Some(Box::new(|header: &ResponseHeader| match header.status {
        StatusCode::OK => Ok(()),
        _ => Err(pingora::Error::create(
            pingora::ErrorType::ConnectError,
            pingora::ErrorSource::Upstream,
            None,
            None,
        )),
    }));

    health_checker
}

pub fn rinha_load_balancer_service() -> GenBackgroundService<LoadBalancer<Consistent>> {
    let default_backend = Backend::new_with_weight("0.0.0.0:8001", 10).unwrap();
    let fallback_backend = Backend::new_with_weight("0.0.0.0:8002", 1).unwrap();
    let mut load_balancer =
        LoadBalancer::<Consistent>::try_from_iter([default_backend, fallback_backend]).unwrap();

    load_balancer.set_health_check(Box::new(http_health_check()));
    load_balancer.update_frequency = None;
    load_balancer.health_check_frequency = Some(Duration::from_secs(5));
    load_balancer.parallel_health_check = true;

    GenBackgroundService::new(
        "Rinha Worker Background Service".to_string(),
        Arc::new(load_balancer),
    )
}
