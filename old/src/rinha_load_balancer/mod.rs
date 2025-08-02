use crate::{
    rinha_conf::{RINHA_DEFAULT_BACKEND_ADDR, RINHA_FALLBACK_BACKEND_ADDR, RINHA_HOST},
    rinha_domain::Target,
};
use http::{Extensions, Uri};
use pingora::{
    http::ResponseHeader,
    lb::{Backend, Backends, LoadBalancer, discovery, health_check::HttpHealthCheck},
    prelude::RoundRobin,
    protocols::l4::socket::SocketAddr,
    services::background::GenBackgroundService,
};
use std::net::ToSocketAddrs;
use std::{collections::BTreeSet, sync::Arc, time::Duration};

fn http_health_check() -> HttpHealthCheck {
    let mut health_checker = HttpHealthCheck::new(RINHA_HOST.as_str(), false);

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

fn resolve_socket_addr(addr: &str) -> SocketAddr {
    let socket_addrs: Vec<std::net::SocketAddr> = addr.to_socket_addrs().unwrap().collect();

    SocketAddr::Inet(socket_addrs.into_iter().next().unwrap())
}

pub fn rinha_load_balancer_service() -> GenBackgroundService<LoadBalancer<RoundRobin>> {
    let mut default_backend = Backend {
        addr: resolve_socket_addr(RINHA_DEFAULT_BACKEND_ADDR.as_str()),
        weight: 10,
        ext: Extensions::new(),
    };
    default_backend.ext.insert(Target::Default);

    let mut fallback_backend = Backend {
        addr: resolve_socket_addr(RINHA_FALLBACK_BACKEND_ADDR.as_str()),
        weight: 1,
        ext: Extensions::new(),
    };
    fallback_backend.ext.insert(Target::Fallback);

    let discovery = discovery::Static::new(BTreeSet::from([default_backend, fallback_backend]));
    let backends = Backends::new(discovery);

    let mut upstreams = LoadBalancer::<RoundRobin>::from_backends(backends);

    upstreams.set_health_check(Box::new(http_health_check()));
    upstreams.health_check_frequency = Some(Duration::from_secs(5));
    upstreams.parallel_health_check = true;
    upstreams.update_frequency = None;

    GenBackgroundService::new(
        "Rinha Worker Background Service".into(),
        Arc::new(upstreams),
    )
}
