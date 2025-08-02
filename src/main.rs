use crate::{rinha_core::Result, rinha_domain::Payment};
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

mod rinha_chan;
mod rinha_conf;
mod rinha_core;
mod rinha_domain;
mod rinha_http;
mod rinha_net;
mod rinha_storage;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_thread_ids(true).init();
    rinha_core::bootstrap();

    let addr = rinha_net::resolve_socket_addr(rinha_conf::RINHA_ADDR.as_str()).await?;
    let tcp_socket = rinha_net::create_tcp_socket(addr)?;
    let tcp_listener = TcpListener::from_std(tcp_socket.into())?;

    tokio::spawn(async move {
        let receiver = rinha_chan::get_receiver();
        let mut receiver = receiver.lock().await;

        loop {
            let payment = receiver.recv().await;
            tracing::debug!(?payment);
        }
    });

    let accept_loop = rinha_net::accept_loop(tcp_listener);
    tokio::spawn(accept_loop).await?
}
