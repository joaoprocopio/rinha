use crate::{error::Result, shared::app::App};
use futures::FutureExt;
use tokio::{fs, io::AsyncWriteExt, net::UnixListener};

pub async fn run_task(
    signal: impl Future<Output = ()> + Send + Sync + 'static,
    server: App,
) -> Result<()> {
    let mut signal = signal.boxed();
    let mut receiver = server.receiver.lock().await;
    if let Some(uds_dirname) = server.env.uds_path.parent() {
        fs::remove_dir_all(uds_dirname).await?;
        fs::create_dir_all(uds_dirname).await?;
    }
    let listener = UnixListener::bind(&server.env.uds_path)?;

    loop {
        let mut stream = tokio::select! {
            Ok(conn) = listener.accept() => {
                conn.0
            }
            _ = &mut signal => {
                tracing::info!("gracefully shutting down uds stream listener");
                break;
            }
        };

        let _ = tokio::select! {
            Some(bytes) = receiver.recv() => {
                stream.write_all(&bytes).await.unwrap_or_else(|err|{
                    tracing::error!("writing to uds socket failed with: {err}");
                });
            }
            _ = &mut signal => {
                tracing::info!("gracefully shutting down channel received");
                break;
            }
        };
    }

    Ok(())
}
