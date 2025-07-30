#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod rinha_conf;
mod rinha_domain;
mod rinha_http;
mod rinha_load_balancer;
mod rinha_tracing;
mod rinha_worker;

use crate::{
    rinha_conf::RINHA_PROD, rinha_domain::Payment, rinha_http::rinha_http_service,
    rinha_load_balancer::rinha_load_balancer_service, rinha_worker::rinha_worker_service,
};
use pingora::{prelude::*, server::configuration::ServerConf};
use std::{num::NonZero, thread};
use tokio::sync::mpsc;

fn main() {
    let mut server_opt = Opt::default();
    server_opt.daemon = RINHA_PROD;

    let mut server_conf = ServerConf::default();
    server_conf.daemon = RINHA_PROD;
    server_conf.threads = thread::available_parallelism()
        .unwrap_or_else(|_| NonZero::new(1).unwrap())
        .into();

    let mut server = Server::new_with_opt_and_conf(server_opt, server_conf);

    server.bootstrap();

    let (sender, receiver) = mpsc::channel::<Payment>(size_of::<Payment>() * 512);

    let rinha_load_balancer = rinha_load_balancer_service();
    let rinha_http = rinha_http_service(sender);
    let rinha_worker = rinha_worker_service(
        receiver,
        rinha_load_balancer.task(),
        server.configuration.clone(),
    );

    server.add_service(rinha_http);
    server.add_service(rinha_load_balancer);
    server.add_service(rinha_worker);

    server.run_forever();
}
