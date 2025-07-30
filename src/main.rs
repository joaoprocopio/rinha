#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod rinha_conf;
mod rinha_domain;
mod rinha_http;
mod rinha_load_balancer;
mod rinha_tracing;
mod rinha_worker;

use crate::{
    rinha_domain::Payment, rinha_http::rinha_http_service,
    rinha_load_balancer::rinha_load_balancer_service, rinha_worker::rinha_worker_service,
};
use pingora::{prelude::*, server::configuration::ServerConf};
use std::sync::Arc;
use tokio::sync::mpsc;

fn main() {
    let mut server = Server::new_with_opt_and_conf(Opt::default(), ServerConf::default());

    server.bootstrap();

    let (sender, receiver) = mpsc::channel::<Payment>(size_of::<Payment>() * 512);

    let rinha_load_balancer = rinha_load_balancer_service();
    let rinha_http = rinha_http_service(sender);
    let rinha_worker = rinha_worker_service(
        receiver,
        rinha_load_balancer.task(),
        Arc::clone(&server.configuration),
    );

    server.add_service(rinha_http);
    server.add_service(rinha_load_balancer);
    server.add_service(rinha_worker);

    server.run_forever();
}
