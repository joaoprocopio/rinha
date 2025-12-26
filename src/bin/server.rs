use anyhow::Ok;
use axum::body::Bytes;
use futures::FutureExt;
use rinha::{error::Result, ext, server, shared};
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
    let (sender, receiver) = mpsc::channel::<Bytes>(size_of::<u8>() << 20);
    let app = shared::app::App::new(handle.clone(), sender).await?;

    let _ = tokio::try_join!(
        async {
            handle
                .spawn(server::http::run_http(signal.clone(), app.clone()))
                .await?
        },
        async {
            handle
                .spawn(server::task::run_task(
                    signal.clone(),
                    app.clone(),
                    receiver,
                ))
                .await?
        },
    )?;

    Ok(())
}
