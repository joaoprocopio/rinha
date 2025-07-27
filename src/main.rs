#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use async_trait::async_trait;
use http::Response;
use pingora::{
    apps::http_app::ServeHttp, prelude::*, protocols::http::ServerSession,
    services::listening::Service,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// const JSON_FORMAT: &'static str = "application/json";

#[derive(Serialize, Deserialize, Debug)]
struct Payment {
    #[serde(rename = "correlationId")]
    correlation_id: String,
    amount: f64,
}

struct Rinha {
    sender: mpsc::Sender<Payment>,
}

impl Rinha {
    fn new(sender: mpsc::Sender<Payment>) -> Self {
        Self { sender: sender }
    }

    async fn payments(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let body = http_session.read_request_body().await.unwrap().unwrap();
        let payment = serde_json::from_slice::<Payment>(&body).unwrap();
        self.sender.send(payment).await.unwrap();

        Response::builder().status(200).body(vec![]).unwrap()
    }
}

#[async_trait]
impl ServeHttp for Rinha {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let header = http_session.req_header();

        if header.method == "POST" && header.raw_path() == b"/payments" {
            self.payments(http_session).await
        } else {
            todo!()
        }
    }
}

fn rinha_service(sender: mpsc::Sender<Payment>) -> Service<Rinha> {
    Service::new("Rinha HTTP Service".to_string(), Rinha::new(sender))
}

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let (sender, receiver) = mpsc::channel::<Payment>(size_of::<Payment>() * 100);

    let mut rinha = rinha_service(sender);
    rinha.add_tcp("0.0.0.0:9999");

    // tokio::spawn(async move {
    //     while let Some(payment) = receiver.recv().await {
    //         dbg!(payment);
    //     }
    // });

    server.add_service(rinha);
    server.run_forever()
}
