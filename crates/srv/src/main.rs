use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const OK_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
const NOT_FOUND_RESPONSE: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";

const SEPARATOR: &[u8] = b"\r\n";
const BODY_SEPARATOR: &[u8] = b"\r\n\r\n";

const SEPARATOR_LEN: usize = SEPARATOR.len();
const BODY_SEPARATOR_LEN: usize = BODY_SEPARATOR.len();

fn split_http_request(request: &[u8]) -> Option<(&[u8], &[u8])> {
    let body_breakpoint = request
        .windows(BODY_SEPARATOR_LEN)
        .position(|window| window == BODY_SEPARATOR)?;

    Some(request.split_at(body_breakpoint + BODY_SEPARATOR_LEN))
}

fn get_http_request_line(header: &[u8]) -> Option<(&[u8], &[u8])> {
    let header_break_pos = header
        .windows(SEPARATOR_LEN)
        .position(|window| window == SEPARATOR)?;

    let request_line = header.split_at(header_break_pos).0;
    let mut request_line = request_line.split(|c| *c == b' ');

    Some((request_line.next()?, request_line.next()?))
}

async fn payments(stream: &mut TcpStream) -> Result<()> {
    stream.write_all(OK_RESPONSE).await?;
    Ok(())
}

async fn not_found(stream: &mut TcpStream) -> Result<()> {
    stream.write_all(NOT_FOUND_RESPONSE).await?;
    Ok(())
}

async fn serve_connection(mut stream: TcpStream) -> Result<()> {
    let mut recv_buffer = [0u8; 2 << 9];
    let read_bytes = stream.read(&mut recv_buffer).await?;

    if let Some((header, _body)) = split_http_request(&recv_buffer[..read_bytes]) {
        if let Some((method, path)) = get_http_request_line(header) {
            if method == b"POST" && path.starts_with(b"/payments") {
                payments(&mut stream).await?
            } else {
                not_found(&mut stream).await?
            }
        }
    }

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
