use anyhow::Ok;
use axum::{self, routing};
use rinha::{error::Result, shared};
use tokio::{net::TcpListener, runtime as rt};

fn main() {
    shared::tracing::init_tracing();
    let runtime = shared::tokio::new_runtime();

    runtime
        .block_on(run(runtime.handle().clone()))
        .unwrap_or_else(|err| {
            tracing::error!(?err);
            std::process::exit(1);
        });
}

async fn run(handle: rt::Handle) -> Result<()> {
    let signal = shared::tokio::shutdown_signal().await?;
    serve(signal, handle.clone()).await?;
    Ok(())
}

async fn serve(
    signal: impl Future<Output = ()> + Send + Sync + 'static,
    handle: rt::Handle,
) -> Result<()> {
    let router = axum::Router::new().route("/payments", routing::get(root));
    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    tracing::info!("www listening on: http://{}", listener.local_addr()?);

    axum::serve(listener, router)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
