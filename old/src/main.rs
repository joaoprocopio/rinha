#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod rinha_conf;
mod rinha_domain;
mod rinha_http;
mod rinha_load_balancer;
mod rinha_storage;
mod rinha_tracing;
mod rinha_worker;

use crate::{
    rinha_domain::Payment, rinha_http::rinha_http_service,
    rinha_load_balancer::rinha_load_balancer_service, rinha_worker::rinha_worker_service,
};
use pingora::{prelude::*, server::configuration::ServerConf};
use tokio::sync::mpsc;

fn main() {
    rinha_storage::bootstrap();
    rinha_conf::bootstrap();

    let server_opt = Opt::default();
    let server_conf = ServerConf::default();
    let mut server = Server::new_with_opt_and_conf(server_opt, server_conf);

    server.bootstrap();

    let (sender, receiver) = mpsc::unbounded_channel::<Payment>();

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
