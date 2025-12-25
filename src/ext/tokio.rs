use crate::error::Result;
use tokio::signal::unix::{SignalKind, signal};

pub fn new_runtime() -> tokio::runtime::Runtime {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            tracing::error!(?err);
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
                tracing::debug!("recv terminate signal");
            },
            _ = interrupt.recv() => {
                tracing::debug!("recv interrupt signal");
            }
        }
    };

    Ok(signal_watcher)
}
