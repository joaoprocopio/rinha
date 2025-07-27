#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use async_trait::async_trait;
use http::Response;
use pingora::{
    apps::http_app::ServeHttp, prelude::*, protocols::http::ServerSession,
    services::listening::Service,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Payment<'a> {
    #[serde(rename = "correlationId")]
    correlation_id: &'a str,
    amount: f64,
}

struct Rinha;

#[async_trait]
impl ServeHttp for Rinha {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let body: &[u8] = &http_session.read_request_body().await.unwrap().unwrap()[..];
        let payment = serde_json::from_slice::<Payment>(body).unwrap();
        let response = serde_json::to_vec(&payment).unwrap();

        Response::builder()
            .status(200)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::CONTENT_LENGTH, response.len())
            .body(response)
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
