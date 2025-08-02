use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use std::error::Error;
use tokio::net::TcpListener;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;
type BoxError = Box<dyn Error + Send + Sync>;
type Result<T, E = BoxError> = std::result::Result<T, E>;

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
    let listener = TcpListener::bind("0.0.0.0:9999").await?;

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
