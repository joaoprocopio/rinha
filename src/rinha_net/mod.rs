use crate::rinha_http;
use http_body_util::combinators::BoxBody;
use hyper::{
    Method, Request, Response,
    body::{Body, Bytes, Incoming},
    client, server, service,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use socket2::{Domain, Protocol, SockAddr, Socket, TcpKeepalive, Type};
use std::{convert::Infallible, error::Error as StdError, net::SocketAddr};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs, lookup_host},
    time::Duration,
};

pub const JSON_CONTENT_TYPE: &'static str = "application/json";

const IPTOS_LOWDELAY: u32 = (0u8 | 0x10) as u32;
const TTL: u32 = 128;
const BACKLOCK_BUFFER_SIZE: i32 = 8 * 1024;
const SEND_BUFFER_SIZE: usize = 64 * 1024;
const RECV_BUFFER_SIZE: usize = 64 * 1024;
const TCP_USER_TIMEOUT: Duration = Duration::from_millis(100);
const TCP_LINGER: Duration = Duration::ZERO;

#[derive(thiserror::Error, Debug)]
pub enum ResolveSocketAddrError {
    #[error("io")]
    IO(#[from] std::io::Error),
    #[error("unmatched")]
    Unmatched,
}

pub async fn resolve_socket_addr<T: ToSocketAddrs>(
    addr: T,
) -> Result<SocketAddr, ResolveSocketAddrError> {
    let mut addrs = lookup_host(addr).await?;
    let addr = addrs
        .next()
        .ok_or_else(|| ResolveSocketAddrError::Unmatched)?;

    Ok(addr)
}

#[derive(thiserror::Error, Debug)]
pub enum CreateTCPSocketError {
    #[error("io")]
    IO(#[from] std::io::Error),
}

pub fn create_tcp_socket(addr: SocketAddr) -> Result<Socket, CreateTCPSocketError> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    set_sock_opt_conf(&socket)?;

    let addr = SockAddr::from(addr);
    socket.bind(&addr)?;
    socket.listen(BACKLOCK_BUFFER_SIZE)?;

    Ok(socket)
}

fn set_sock_opt_conf(socket: &Socket) -> Result<(), std::io::Error> {
    let mut keepalive = TcpKeepalive::new();
    keepalive = keepalive.with_time(Duration::from_secs(90));
    keepalive = keepalive.with_interval(Duration::from_secs(30));

    socket.set_tcp_keepalive(&keepalive)?;
    socket.set_tcp_quickack(true)?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_tcp_nodelay(true)?;
    socket.set_nonblocking(true)?;
    socket.set_ttl_v4(TTL)?;
    socket.set_tos_v4(IPTOS_LOWDELAY)?;
    socket.set_send_buffer_size(SEND_BUFFER_SIZE)?;
    socket.set_recv_buffer_size(RECV_BUFFER_SIZE)?;
    socket.set_tcp_user_timeout(Some(TCP_USER_TIMEOUT))?;
    socket.set_linger(Some(TCP_LINGER))?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum CreateTCPSenderError {
    #[error("io")]
    IO(#[from] std::io::Error),
    #[error("hyper")]
    Hyper(#[from] hyper::Error),
}

pub async fn create_tcp_socket_sender<B>(
    addr: SocketAddr,
) -> Result<client::conn::http1::SendRequest<B>, CreateTCPSenderError>
where
    B: Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    let stream = TcpStream::connect(addr).await?;
    let socket = socket2::SockRef::from(&stream);
    set_sock_opt_conf(&socket)?;

    let io = TokioIo::new(stream);
    let (sender, conn) = client::conn::http1::handshake::<TokioIo<TcpStream>, B>(io).await?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!(?err);
        }
    });

    Ok(sender)
}

#[derive(thiserror::Error, Debug)]
pub enum AcceptLoopError {
    #[error("io")]
    IO(#[from] std::io::Error),
}

pub async fn accept_loop(tcp_listener: TcpListener) -> Result<(), AcceptLoopError> {
    let mut http = server::conn::http1::Builder::new();

    http.writev(true);
    http.timer(TokioTimer::new());
    http.pipeline_flush(true);
    http.half_close(false);

    let service = service::service_fn(router);

    loop {
        let (stream, _) = tcp_listener.accept().await?;
        let http = http.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            if let Err(err) = http.serve_connection(io, service).await {
                tracing::error!(?err);
            };
        });
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RouterError {
    #[error("payments")]
    Payments(#[from] rinha_http::PaymentsError),
    #[error("payments summary")]
    PaymentsSummary(#[from] rinha_http::PaymentsSummaryError),
    #[error("not found")]
    NotFound(#[from] rinha_http::NotFoundError),
}

pub async fn router(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, RouterError> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/payments") => Ok(rinha_http::payments(req).await?),
        (&Method::GET, "/payments-summary") => Ok(rinha_http::payments_summary(req).await?),
        _ => Ok(rinha_http::not_found().await?),
    }
}
