use anyhow::Ok;
use rinha::{error::Result, shared, worker};
use tokio::runtime::Handle;

fn main() {
    shared::tracing::init_tracing();
    let runtime = shared::tokio::new_runtime();

    runtime
        .block_on(run(runtime.handle().clone()))
        .unwrap_or_else(|err| {
            tracing::error!("worker boot failed with: {}", err);
            std::process::exit(1);
        });
}

async fn run(handle: Handle) -> Result<()> {
    let signal = shared::tokio::shutdown_signal().await?;
    handle.spawn(worker::run_worker()).await?;
    Ok(())
}
