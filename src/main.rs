#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod rinha_domain;
mod rinha_http;
mod rinha_load_balancer;
mod rinha_worker;

use crate::{
    rinha_domain::Payment, rinha_http::rinha_http_service,
    rinha_load_balancer::rinha_load_balancer_service, rinha_worker::rinha_worker_service,
};
use pingora::{prelude::*, server::configuration::ServerConf};
use tokio::sync::mpsc::channel;

fn main() -> Result<()> {
    let opt = Opt::default();
    let conf = ServerConf::default();
    let mut server = Server::new_with_opt_and_conf(opt, conf);
    server.bootstrap();

    let (sender, receiver) = channel::<Payment>(size_of::<Payment>() * 256);

    let rinha_load_balancer = rinha_load_balancer_service();
    let rinha_http = rinha_http_service(sender);
    let rinha_worker = rinha_worker_service(receiver, rinha_load_balancer.task());

    server.add_service(rinha_http);
    server.add_service(rinha_worker);
    server.add_service(rinha_load_balancer);

    server.run_forever();
}
