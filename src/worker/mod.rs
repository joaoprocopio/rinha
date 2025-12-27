use crate::shared::data::Payment;
use futures::FutureExt;
pub use state::Worker;
use tokio::{io::AsyncReadExt, net::UnixStream};

mod state;

const RECVBUF_SIZE: usize = 2048;

pub async fn run_worker(signal: impl Future<Output = ()> + Send + Sync + 'static, worker: Worker) {
    let mut signal = signal.boxed();

    loop {
        let mut stream = tokio::select! {
            Ok(stream) = UnixStream::connect(&worker.env.uds_path) => {
                stream
            }
            _ = &mut signal => {
                tracing::info!("gracefully shutting down uds stream");
                break;
            }
        };

        let mut buf: [u8; _] = [0; RECVBUF_SIZE];

        tokio::select! {
            Ok(len) = stream.read(&mut buf) => {
                process_payment(buf, len).await;
            }
            _ = &mut signal => {
                tracing::info!("gracefully shutting uds stream reader");
                break;
            }
        };
    }
}

async fn process_payment(buf: [u8; RECVBUF_SIZE], len: usize) {
    let payment = serde_json::from_slice::<Payment>(&buf[..len]);
    tracing::debug!("{:?}", payment);
}
