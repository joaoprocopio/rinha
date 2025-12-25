use anyhow::Ok;
use rinha::{error::Result, server, shared};
use tokio::runtime::Handle;

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
    server::run_server(signal, handle.clone()).await?;
    Ok(())
}
