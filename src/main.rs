use async_trait::async_trait;
use http::Response;
use pingora::{
    apps::http_app::{HttpServer, ServeHttp},
    modules::http::compression::ResponseCompressionBuilder,
    prelude::*,
    protocols::{Stream, http::ServerSession},
    services::listening::Service,
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

struct RinhaServer(HttpServer<RinhaHttpApp>);

impl RinhaServer {
    fn new() -> Self {
        let mut server = HttpServer::new_app(RinhaHttpApp);
        server.add_module(ResponseCompressionBuilder::enable(7));
        Self(server)
    }
}

struct RinhaService(Service<RinhaServer>);

impl RinhaService {
    fn rinha_http_service() -> Self {
        Self(Service::new(
            "Rinha HTTP Service".to_string(),
            RinhaServer::new(),
        ))
    }
}

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let mut rinha_http_service = RinhaService::rinha_http_service().0;
    rinha_http_service.add_tcp("127.0.0.1:8080");
    server.add_service(rinha_http_service);

    server.run_forever();
}
