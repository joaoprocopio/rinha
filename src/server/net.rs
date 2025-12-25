use crate::{error::Result, server::Server};
use axum::{Router, body::Bytes, extract::State, routing, serve};
use tokio::{net::TcpListener, runtime::Handle, sync::mpsc::Sender};

pub async fn run_server(
    signal: impl Future<Output = ()> + Send + Sync + 'static,
    handle: Handle,
    sender: Sender<Bytes>,
) -> Result<()> {
    let server = Server::new(handle, sender).await?;
    let listener = TcpListener::bind(server.env.addr.as_str()).await?;
    let router = Router::new()
        .route("/payments", routing::post(payments))
        .with_state(server);

    tracing::info!("server listening on: http://{}", listener.local_addr()?);

    serve(listener, router)
        .with_graceful_shutdown(signal)
        .await?;

    Ok(())
}

// pub async fn run_uds_receiver() -> Result<()> {
//     let stream = UnixStream::connect("/run/rinha/rinha.sock").await?;

//     Ok(())
// }

async fn payments(State(state): State<Server>, buf: Bytes) -> () {
    state.sender.send(buf).await.unwrap_or_else(|err| {
        tracing::error!("failed to send buffer: {err}",);
    });
}
