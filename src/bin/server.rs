use anyhow::Ok;
use axum::body::Bytes;
use rinha::{error::Result, prelude::*, server, shared};
use tokio::{runtime::Handle, sync::mpsc};

fn main() {
    shared::tracing::init_tracing();
    let runtime = shared::tokio::new_runtime();

    runtime
        .block_on(run(runtime.handle().clone()))
        .unwrap_or_else(|err| {
            tracing::error!("server boot failed with: {}", err);
            std::process::exit(1);
        });
}

async fn run(handle: Handle) -> Result<()> {
    let signal = shared::tokio::shutdown_signal().await?.shared();

    let (sender, receiver) = mpsc::channel::<Bytes>(
        /* TODO: calcular um tamanho melhor pra esse cara */ 1024 * 1000,
    );

    let _ = tokio::try_join!(
        async {
            handle
                .spawn(server::run_server(signal.clone(), handle.clone(), sender))
                .await?
        },
        async {
            handle
                .spawn(server::run_server_worker(signal.clone(), receiver))
                .await?
        }
    )?;

    Ok(())
}
