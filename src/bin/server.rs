use anyhow::Ok;
use axum::body::Bytes;
use futures::FutureExt;
use rinha::{error::Result, ext, server, shared::vars::MB};
use tokio::{runtime::Handle, sync::mpsc};

fn main() {
    ext::tracing::init_tracing();
    let runtime = ext::tokio::new_runtime();

    runtime
        .block_on(run(runtime.handle().clone()))
        .unwrap_or_else(|err| {
            tracing::error!("server boot failed with: {}", err);
            std::process::exit(1);
        });
}

async fn run(handle: Handle) -> Result<()> {
    let signal = ext::tokio::shutdown_signal().await?.shared();
    let (sender, receiver) = mpsc::channel::<Bytes>(MB);
    let server = server::Server::new(handle.clone(), sender).await?;

    let _ = tokio::try_join!(
        async {
            handle
                .spawn(server::http::run_http(signal.clone(), server.clone()))
                .await?
        },
        async {
            handle
                .spawn(server::task::run_task(
                    signal.clone(),
                    server.clone(),
                    receiver,
                ))
                .await?
        },
    )?;

    Ok(())
}
