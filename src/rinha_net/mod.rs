use crate::{rinha_core::Result, rinha_http};
use http_body_util::combinators::BoxBody;
use hyper::{
    Method, Request, Response,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use socket2::{Domain, Protocol, SockAddr, Socket, TcpKeepalive, Type};
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, ToSocketAddrs, lookup_host};

pub const JSON_CONTENT_TYPE: &'static str = "application/json";

pub async fn resolve_socket_addr<T: ToSocketAddrs>(addr: T) -> Result<SocketAddr> {
    let mut addrs = lookup_host(addr).await?;
    let addr = addrs.next().ok_or_else(|| "Couldn't match an address")?;

    Ok(addr)
}

pub fn create_tcp_socket(addr: SocketAddr) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };
    let addr = SockAddr::from(addr);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    let backlog = 4096;

    let keepalive = TcpKeepalive::new().with_time(Duration::from_secs(75));

    socket.set_tcp_keepalive(&keepalive)?;
    socket.set_tcp_quickack(true)?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_tcp_nodelay(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr)?;
    socket.listen(backlog)?;

    Ok(socket)
}

pub async fn accept_loop(tcp_listener: TcpListener) -> Result<()> {
    let mut http = http1::Builder::new();

    http.writev(true);
    http.timer(TokioTimer::new());
    http.pipeline_flush(true);
    http.half_close(false);

    let service = service_fn(router);

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

pub async fn router(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/payments") => rinha_http::payments(req).await,
        (&Method::GET, "/payments-summary") => rinha_http::payments_summary(req).await,
        _ => rinha_http::not_found_error(),
    }
}
