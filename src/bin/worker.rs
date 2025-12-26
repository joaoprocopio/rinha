fn main() {}

/*
let task_path = server.env.uds_path.clone();
let mut task_signal = signal.clone();

server.handle.spawn(async move {
    loop {
        let mut stream = tokio::select! {
            Ok(conn) = UnixStream::connect(&task_path) => {
                conn
            }
            _ = &mut task_signal => {
                tracing::info!("gracefully shutting down uds stream listener");
                break;
            }
        };

        let mut buf = [0; 1024];

        let _ = tokio::select! {
            Ok(len) = stream.read(&mut buf) => {
                let data = unsafe { str::from_utf8_unchecked(&buf[..len]) };
                tracing::debug!("{data}");
            }
        };
    }
});
*/
