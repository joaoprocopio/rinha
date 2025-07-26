#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use async_trait::async_trait;
use http::Response;
use pingora::{
    apps::http_app::ServeHttp, listeners::TcpSocketOptions, prelude::*,
    protocols::http::ServerSession, services::listening::Service,
};

struct RinhaHttpApp;

#[async_trait]
impl ServeHttp for RinhaHttpApp {
    async fn response(&self, _http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let text = "Hello, world!";
        let buf = text.as_bytes().to_vec();

        Response::builder()
            .status(200)
            .header(http::header::CONTENT_TYPE, "text/plain; version=0.0.4")
            .header(http::header::CONTENT_LENGTH, buf.len())
            .body(buf)
            .unwrap()
    }
}

fn rinha_http_service() -> Service<RinhaHttpApp> {
    Service::new("Rinha HTTP Service".to_string(), RinhaHttpApp)
}

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let mut rinha = rinha_http_service();

    rinha.add_tcp_with_settings("127.0.0.1:8000", TcpSocketOptions::default());

    server.add_service(rinha);
    server.run_forever();
}
