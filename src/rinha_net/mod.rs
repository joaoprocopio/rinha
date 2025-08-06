use crate::rinha_http;
use http_body_util::Full;
use hyper::{
    Method, Request, Response,
    body::{Bytes, Incoming},
    server, service,
};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::{TokioExecutor, TokioIo, TokioTimer},
};
use socket2::{Domain, Protocol, SockAddr, Socket, TcpKeepalive, Type};
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::{
    net::{TcpListener, ToSocketAddrs, lookup_host},
    time::Duration,
};

static CLIENT: LazyLock<Arc<Client<HttpConnector, Full<Bytes>>>> = LazyLock::new(|| {
    let mut client = Client::builder(TokioExecutor::new());
    client.pool_timer(TokioTimer::new());
    client.pool_idle_timeout(KEEPALIVE_TIME);

    let mut conn = HttpConnector::new();
    conn.set_keepalive(Some(KEEPALIVE_TIME));
    conn.set_keepalive_interval(Some(KEEPALIVE_INTERVAL));
    conn.set_tcp_user_timeout(Some(USER_TIMEOUT));
    conn.set_nodelay(NODELAY);
    conn.set_reuse_address(true);

    Arc::new(client.build(conn))
});

pub const JSON_CONTENT_TYPE: &'static str = "application/json";

const TTL: u32 = 128;
const NODELAY: bool = true;
const IPTOS_LOWDELAY: u32 = (0u8 | 0x10) as u32;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
const KEEPALIVE_TIME: Duration = Duration::from_secs(90);
const BACKLOCK_BUFFER_SIZE: i32 = 8 * 1024;
const SEND_BUFFER_SIZE: usize = 64 * 1024;
const RECV_BUFFER_SIZE: usize = 64 * 1024;
const USER_TIMEOUT: Duration = Duration::from_secs(1);
const LINGER: Duration = Duration::ZERO;

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
    keepalive = keepalive.with_time(KEEPALIVE_TIME);
    keepalive = keepalive.with_interval(KEEPALIVE_INTERVAL);

    socket.set_tcp_keepalive(&keepalive)?;
    socket.set_tcp_quickack(true)?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_tcp_nodelay(NODELAY)?;
    socket.set_nonblocking(true)?;
    socket.set_ttl_v4(TTL)?;
    socket.set_tos_v4(IPTOS_LOWDELAY)?;
    socket.set_send_buffer_size(SEND_BUFFER_SIZE)?;
    socket.set_recv_buffer_size(RECV_BUFFER_SIZE)?;
    socket.set_tcp_user_timeout(Some(USER_TIMEOUT))?;
    socket.set_linger(Some(LINGER))?;

    Ok(())
}

pub fn get_client() -> Arc<Client<HttpConnector, Full<Bytes>>> {
    CLIENT.clone()
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
    http.keep_alive(true);

    let service = service::service_fn(router);

    loop {
        let (stream, _) = tcp_listener.accept().await?;
        let http = http.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            if let Err(err) = http.serve_connection(io, service).await {
                tracing::error!(?err, "accept loop");
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

pub async fn router(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, RouterError> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/payments") => Ok(rinha_http::payments(req).await?),
        (&Method::GET, "/payments-summary") => Ok(rinha_http::payments_summary(req).await?),
        _ => Ok(rinha_http::not_found().await?),
    }
}

pub fn bootstrap() {
    LazyLock::force(&CLIENT);
}
