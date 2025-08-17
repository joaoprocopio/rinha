use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

const EMPTY_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n";

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn serve_connection(mut stream: TcpStream) -> Result<()> {
    stream.write_all(EMPTY_RESPONSE).await?;
    stream.shutdown().await?;

    Ok(())
}

async fn accept_loop(listener: TcpListener) -> Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(err) = serve_connection(stream).await {
                eprintln!("connection error: {err}");
            }
        });
    }
}

async fn serve() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9999").await?;
    let accept_loop = accept_loop(listener);

    tokio::spawn(accept_loop).await?
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("connection error {err}");
        std::process::exit(1);
    }
}
