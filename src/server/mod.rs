mod config;

use crate::error::Result;
use axum::{Router, routing, serve};
use config::Server;
use tokio::net::TcpListener;

pub async fn run_server(
    signal: impl Future<Output = ()> + Send + Sync + 'static,
    handle: tokio::runtime::Handle,
) -> Result<()> {
    let server = Server::new(handle).await?;
    let listener = TcpListener::bind(server.env.addr.as_str()).await?;
    let router = Router::new().route("/payments", routing::get(payments));

    tracing::info!("server listening on: http://{}", listener.local_addr()?);

    serve(listener, router)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}

async fn payments() -> &'static str {
    "Hello, World!"
}
