#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use crate::rinha_core::Result;
use tokio::net::TcpListener;

mod rinha_balancer;
mod rinha_chan;
mod rinha_conf;
mod rinha_core;
mod rinha_domain;
mod rinha_http;
mod rinha_net;
mod rinha_storage;
mod rinha_worker;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_thread_ids(true).init();

    rinha_chan::boostrap();
    rinha_conf::bootstrap();
    rinha_storage::bootstrap();
    rinha_balancer::bootstrap().await?;

    {
        let worker_task = rinha_worker::task();
        tokio::spawn(worker_task);
    }

    {
        let balancer_task = rinha_balancer::task();
        tokio::spawn(balancer_task);
    }

    let addr = rinha_net::resolve_socket_addr(rinha_conf::RINHA_ADDR.as_str()).await?;
    let tcp_socket = rinha_net::create_tcp_socket(addr)?;
    let tcp_listener = TcpListener::from_std(tcp_socket.into())?;

    let accept_loop = rinha_net::accept_loop(tcp_listener);
    tokio::spawn(accept_loop).await?
}
