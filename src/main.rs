use crate::rinha_core::Result;
use tokio::net::TcpListener;

mod rinha_conf;
mod rinha_core;
mod rinha_net;

#[tokio::main]
async fn main() -> Result<()> {
    rinha_conf::bootstrap().await;

    let addr = rinha_net::resolve_socket_addr(rinha_conf::RINHA_ADDR.as_str()).await?;
    let tcp_socket = rinha_net::create_tcp_socket(addr)?;
    let tcp_listener = TcpListener::from_std(tcp_socket.into())?;

    let accept_loop = rinha_net::accept_loop(tcp_listener);
    tokio::spawn(accept_loop).await?
}
