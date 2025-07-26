#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use async_trait::async_trait;
use http::Response;
use pingora::{
    apps::http_app::ServeHttp, prelude::*, protocols::http::ServerSession,
    services::listening::Service,
};

struct Rinha;

#[async_trait]
impl ServeHttp for Rinha {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let text = "Hello, world!";
        println!("{:?}", http_session.req_header());
        let buf = text.as_bytes().to_vec();

        Response::builder()
            .status(200)
            .header(http::header::CONTENT_TYPE, "text/plain")
            .header(http::header::CONTENT_LENGTH, buf.len())
            .body(buf)
            .unwrap()
    }
}

fn rinha_service() -> Service<Rinha> {
    Service::new("Rinha HTTP Service".to_string(), Rinha)
}

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let mut rinha = rinha_service();
    rinha.add_tcp("0.0.0.0:9999");

    server.add_service(rinha);
    server.run_forever();
}
