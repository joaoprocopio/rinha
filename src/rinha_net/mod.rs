use crate::rinha_core::Result;
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::SocketAddr;
use tokio::net::{TcpListener, ToSocketAddrs, lookup_host};

pub type BoxBody = http_body_util::combinators::BoxBody<hyper::body::Bytes, hyper::Error>;

// fn resolve_socket_addr(addr: &str) -> SocketAddr {
//     let socket_addrs: Vec<std::net::SocketAddr> = addr.to_socket_addrs().unwrap().collect();

//     SocketAddr::Inet(socket_addrs.into_iter().next().unwrap())
// }

pub async fn resolve_socket_addr<T: ToSocketAddrs>(addr: T) -> Result<SocketAddr> {
    let mut addrs = lookup_host(addr).await?;
    let addr = addrs.next().ok_or("Couldn't match an address")?;

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

    socket.set_tcp_nodelay(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr)?;
    socket.listen(backlog)?;

    Ok(socket)
}

pub async fn router(req: Request<Incoming>) -> Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let body = Full::new(Bytes::from("hello, world!"))
                .map_err(|never| match never {})
                .boxed();

            Ok(Response::new(body))
        }
        _ => {
            let body = Empty::new().map_err(|never| match never {}).boxed();

            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body)
                .unwrap())
        }
    }
}

pub async fn accept_loop(tcp_listener: TcpListener) -> Result<()> {
    let mut http = http1::Builder::new();
    http.pipeline_flush(true);

    let service = service_fn(router);

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let http = http.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(tcp_stream);
            if let Err(_) = http.serve_connection(io, service).await {
                //
            };
        });
    }
}
