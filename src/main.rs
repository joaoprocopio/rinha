#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use tokio::net::TcpListener;

mod rinha_ambulance;
mod rinha_chan;
mod rinha_conf;
mod rinha_domain;
mod rinha_http;
mod rinha_net;
mod rinha_storage;
mod rinha_worker;

#[derive(thiserror::Error, Debug)]
enum MainError {
    #[error("join error")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("io")]
    IO(#[from] std::io::Error),

    #[error("accept loop")]
    AcceptLoop(#[from] rinha_net::AcceptLoopError),
    #[error("ambulance")]
    Ambulance(#[from] rinha_ambulance::BootstrapError),
    #[error("resolve socket")]
    ResolveSocket(#[from] rinha_net::ResolveSocketAddrError),
    #[error("create socket")]
    CreateSocket(#[from] rinha_net::CreateTCPSocketError),
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt().init();

    rinha_net::bootstrap();
    rinha_chan::boostrap();
    rinha_conf::bootstrap();
    rinha_storage::bootstrap();
    rinha_ambulance::bootstrap().await?;

    {
        let worker_task = rinha_worker::task();
        tokio::spawn(worker_task);
    }

    {
        let ambulance_task = rinha_ambulance::task();
        tokio::spawn(ambulance_task);
    }

    let addr = rinha_net::resolve_socket_addr(rinha_conf::RINHA_ADDR.as_str()).await?;
    let tcp_socket = rinha_net::create_tcp_socket(addr)?;
    let tcp_listener = TcpListener::from_std(tcp_socket.into())?;

    let accept_loop = rinha_net::accept_loop(tcp_listener);
    Ok(tokio::spawn(accept_loop).await??)
}
