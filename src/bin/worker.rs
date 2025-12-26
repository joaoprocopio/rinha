use anyhow::Ok;
use rinha::{error::Result, ext, worker};
use tokio::runtime::Handle;

fn main() {
    ext::tracing::init_tracing();
    let runtime = ext::tokio::new_runtime();

    runtime
        .block_on(run(runtime.handle().clone()))
        .unwrap_or_else(|err| {
            tracing::error!("worker boot failed with: {}", err);
            std::process::exit(1);
        });
}

async fn run(handle: Handle) -> Result<()> {
    let signal = ext::tokio::shutdown_signal().await?;
    let worker = worker::Worker::new(handle.clone()).await?;

    handle.spawn(worker::run_worker(signal, worker)).await?;

    Ok(())
}
