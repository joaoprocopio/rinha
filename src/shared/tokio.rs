use crate::error::Result;
use tokio::signal::unix::{SignalKind, signal};

pub fn new_runtime() -> tokio::runtime::Runtime {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            tracing::error!("tokio runtime boot failed: {err}");
            std::process::exit(1);
        }
    }
}

pub async fn shutdown_signal() -> Result<impl Future<Output = ()>> {
    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;

    let signal_watcher = async move {
        tokio::select! {
            _ = terminate.recv() => {
                tracing::info!("SIGTERM received");
            },
            _ = interrupt.recv() => {
                tracing::info!("SIGINT received");
            }
        }
    };

    Ok(signal_watcher)
}
