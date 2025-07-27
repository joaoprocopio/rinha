#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod rinha_ambulance;
mod rinha_domain;
mod rinha_http;
mod rinha_worker;

use crate::{
    rinha_ambulance::rinha_ambulance_service, rinha_domain::Payment,
    rinha_http::rinha_http_service, rinha_worker::rinha_worker_service,
};
use pingora::prelude::*;
use tokio::sync::mpsc::channel;

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let (sender, receiver) = channel::<Payment>(size_of::<Payment>() * 100);

    let mut rinha_http = rinha_http_service(sender);
    rinha_http.add_tcp("0.0.0.0:9999");

    let rinha_worker = rinha_worker_service(receiver);
    let rinha_ambulance = rinha_ambulance_service();

    server.add_service(rinha_http);
    server.add_service(rinha_worker);
    server.add_service(rinha_ambulance);

    server.run_forever();
}
