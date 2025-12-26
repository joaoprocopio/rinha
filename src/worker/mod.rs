use futures::FutureExt;
pub use state::Worker;
use tokio::{io::AsyncReadExt, net::UnixStream};

mod state;

pub async fn run_worker(signal: impl Future<Output = ()> + Send + Sync + 'static, worker: Worker) {
    let mut signal = signal.boxed();

    loop {
        let mut stream = tokio::select! {
            Ok(stream) = UnixStream::connect(&worker.env.uds_path) => {
                stream
            }
            _ = &mut signal => {
                tracing::info!("gracefully shutting down uds stream listener");
                break;
            }
        };

        let mut buf: [u8; _] = [0; 2048];

        tokio::select! {
            Ok(len) = stream.read(&mut buf) => {
                let data = unsafe { str::from_utf8_unchecked(&buf[..len]) };
                dbg!(data);
            }
            _ = &mut signal => {
                tracing::info!("gracefully shutting down channel receiver");
                break;
            }
        };
    }
}
