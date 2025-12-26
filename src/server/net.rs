use crate::{error::Result, server::Server};
use axum::{Router, body::Bytes, extract::State, routing, serve};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, UnixStream},
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

pub async fn run_server_worker(
    signal: impl Future<Output = ()> + Send + Sync + 'static,
    mut receiver: Receiver<Bytes>,
) -> Result<()> {
    // TODO: receber o endereço do socket uds como variavel de ambiente
    let mut stream = UnixStream::connect("/run/rinha/rinha.sock").await?;

    tokio::pin!(signal);

    loop {
        tokio::select! {
            _ = &mut signal => {
                tracing::info!("shutting down server worker");
                break;
            }
            Some(bytes) = receiver.recv() => {
                stream.write_all(&bytes).await.unwrap_or_else(|err|{
                    tracing::error!("writing to uds socket failed with: {err}");
                });
            }
        }
    }

    stream.shutdown().await.unwrap_or_else(|err| {
        tracing::error!("uds socket shutdown failed with: {err}");
    });

    Ok(())
}

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

async fn payments(State(state): State<Server>, buf: Bytes) -> () {
    state.sender.send(buf).await.unwrap_or_else(|err| {
        tracing::error!("failed to send buffer: {err}");
    });
}
