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
use std::{net::SocketAddr, sync::LazyLock};
use tokio::{
    net::{TcpListener, ToSocketAddrs, lookup_host},
    time::Duration,
};

pub const JSON_CONTENT_TYPE: &str = "application/json";

static CLIENT: LazyLock<Client<HttpConnector, Full<Bytes>>> = LazyLock::new(|| {
    let mut client = Client::builder(TokioExecutor::new());
    client.pool_timer(TokioTimer::new());
    client.pool_idle_timeout(Duration::from_secs(30));
    client.pool_max_idle_per_host(8);

    let mut conn = HttpConnector::new();
    conn.set_keepalive(Some(Duration::from_secs(30)));
    conn.set_keepalive_interval(Some(Duration::from_secs(10)));
    conn.set_tcp_user_timeout(Some(Duration::from_secs(3)));
    conn.set_nodelay(true);
    conn.set_reuse_address(true);
    conn.set_connect_timeout(Some(Duration::from_millis(500)));
    conn.set_happy_eyeballs_timeout(Some(Duration::from_millis(100)));

    client.build(conn)
});

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
    socket.listen(8 * 1024)?;

    Ok(socket)
}

fn set_sock_opt_conf(socket: &Socket) -> Result<(), std::io::Error> {
    let mut keepalive = TcpKeepalive::new();
    keepalive = keepalive.with_time(Duration::from_secs(30));
    keepalive = keepalive.with_interval(Duration::from_secs(10));

    socket.set_tcp_keepalive(&keepalive)?;
    socket.set_tcp_quickack(true)?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.set_tcp_nodelay(true)?;
    socket.set_ttl_v4(64)?;

    socket.set_tos_v4(0x10)?;

    socket.set_send_buffer_size(32 * 1024)?;
    socket.set_recv_buffer_size(32 * 1024)?;

    socket.set_tcp_user_timeout(Some(Duration::from_secs(3)))?;
    socket.set_linger(Some(Duration::ZERO))?;

    Ok(())
}

pub fn get_client() -> Client<HttpConnector, Full<Bytes>> {
    CLIENT.clone()
}

#[derive(thiserror::Error, Debug)]
pub enum AcceptLoopError {
    #[error("io")]
    IO(#[from] std::io::Error),
}

pub async fn accept_loop(tcp_listener: TcpListener) -> Result<(), AcceptLoopError> {
    let mut http = server::conn::http1::Builder::new();

    http.writev(false);
    http.timer(TokioTimer::new());
    http.pipeline_flush(false);
    http.half_close(true);
    http.keep_alive(false);
    http.header_read_timeout(Duration::from_millis(100));
    http.max_buf_size(16 * 1024);

    let service = service::service_fn(router);

    loop {
        let http = http.clone();
        let (stream, _) = tcp_listener.accept().await?;
        let socket = socket2::SockRef::from(&stream);
        let _ = socket.set_tcp_nodelay(true);
        let _ = socket.set_tcp_quickack(true);

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
