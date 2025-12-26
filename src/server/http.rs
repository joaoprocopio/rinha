use crate::{error::Result, shared::app::App};
use axum::{Router, body::Bytes, extract::State, routing, serve};
use tokio::net::TcpListener;

pub async fn run_http(
    signal: impl Future<Output = ()> + Send + Sync + 'static,
    app: App,
) -> Result<()> {
    let listener = TcpListener::bind(app.env.addr.as_str()).await?;
    let router = Router::new()
        .route("/payments", routing::post(payments))
        .with_state(app);

    tracing::info!("server listening on: http://{}", listener.local_addr()?);

    serve(listener, router)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}

async fn payments(State(app): State<App>, buf: Bytes) -> () {
    app.sender.send(buf).await.unwrap_or_else(|err| {
        tracing::error!("failed to send buffer: {err}");
    });
}
