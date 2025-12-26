use anyhow::Ok;
use axum::body::Bytes;
use rinha::{error::Result, shared, worker};
use tokio::{runtime::Handle, sync::mpsc};

fn main() {
    shared::tracing::init_tracing();
    let runtime = shared::tokio::new_runtime();

    runtime
        .block_on(run(runtime.handle().clone()))
        .unwrap_or_else(|err| {
            tracing::error!("server boot serverfailed with: {}", err);
            std::process::exit(1);
        });
}

async fn run(handle: Handle) -> Result<()> {
    let signal = shared::tokio::shutdown_signal().await?;
    let (sender, receiver) = mpsc::channel::<Bytes>(size_of::<u8>() << 20);

    handle.spawn(worker::run_worker()).await?;

    Ok(())
}
