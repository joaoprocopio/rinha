use anyhow::Ok;
use axum::body::Bytes;
use rinha::{error::Result, server, shared};
use tokio::{runtime::Handle, sync::mpsc};

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

async fn run(handle: Handle) -> Result<()> {
    let signal = shared::tokio::shutdown_signal().await?;
    // TODO: calcular um tamanho melhor pra esse ara
    let (sender, _) = mpsc::channel::<Bytes>(1024 * 1000);
    server::run_server(signal, handle.clone(), sender).await?;
    Ok(())
}
