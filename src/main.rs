use crate::{rinha_core::Result, rinha_net::BoxBody};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod rinha_conf;
mod rinha_core;
mod rinha_net;

async fn router(req: Request<Incoming>) -> Result<Response<BoxBody>> {
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

async fn accept_loop(tcp_listener: TcpListener) -> Result<()> {
    let mut http = http1::Builder::new();
    http.pipeline_flush(true);

    let service = service_fn(router);

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let http = http.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(tcp_stream);
            if let Err(_) = http.serve_connection(io, service).await {};
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    rinha_conf::bootstrap().await;

    let addr: SocketAddr = rinha_conf::RINHA_ADDR.parse()?;
    let tcp_socket = rinha_net::create_tcp_socket(addr)?;
    let tcp_listener = TcpListener::from_std(tcp_socket.into())?;

    let accept_loop = accept_loop(tcp_listener);
    tokio::spawn(accept_loop).await?
}
