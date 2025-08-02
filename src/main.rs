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

async fn muxer(req: Request<Incoming>) -> Result<Response<BoxBody>> {
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

#[tokio::main]
async fn main() -> Result<()> {
    rinha_conf::bootstrap().await;

    let addr: SocketAddr = rinha_conf::RINHA_ADDR.parse()?;
    let socket = rinha_net::create_tcp_socket(addr)?;
    let listener = TcpListener::from_std(socket.into())?;

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(muxer))
                .await
            {
                println!("error serving connection {:?}", err)
            }
        });
    }
}
