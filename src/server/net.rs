use crate::{error::Result, server::Server};
use axum::{Router, body::Bytes, extract::State, routing, serve};
use futures::FutureExt;
use tokio::{
    fs,
    io::AsyncWriteExt,
    net::{TcpListener, UnixListener},
};

pub async fn run_worker(
    signal: impl Future<Output = ()> + Send + 'static,
    server: Server,
) -> Result<()> {
    let mut signal = signal.boxed();
    let mut receiver = server.receiver.lock().await;

    if let Some(uds_dirname) = server.env.uds_path.parent() {
        fs::create_dir_all(uds_dirname).await?;
    }

    let listener = UnixListener::bind(&server.env.uds_path)?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::select! {
            _ = &mut signal => {
                tracing::info!("shutting down server worker");
                stream.shutdown().await.unwrap_or_else(|err| {
                    tracing::error!("uds socket shutdown failed with: {err}");
                });
                break;
            }
            Some(bytes) = receiver.recv() => {
                stream.write_all(&bytes).await.unwrap_or_else(|err|{
                    tracing::error!("writing to uds socket failed with: {err}");
                });
            }
        }
    }

    Ok(())
}

pub async fn run_server(
    signal: impl Future<Output = ()> + Send + 'static,
    server: Server,
) -> Result<()> {
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

async fn payments(State(state): State<Server>, buf: Bytes) -> () {
    state.sender.send(buf).await.unwrap_or_else(|err| {
        tracing::error!("failed to send buffer: {err}");
    });
}
