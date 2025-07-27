#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::sync::Arc;

use async_trait::async_trait;
use http::Response;
use pingora::{
    apps::http_app::ServeHttp,
    prelude::*,
    protocols::http::ServerSession,
    server::ShutdownWatch,
    services::{
        background::{BackgroundService, GenBackgroundService},
        listening::Service,
    },
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug)]
struct Payment {
    #[serde(rename = "correlationId")]
    correlation_id: String,
    amount: f64,
}

struct Rinha {
    sender: Arc<mpsc::Sender<Payment>>,
}

impl Rinha {
    fn new(sender: mpsc::Sender<Payment>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

struct RinhaWorker {
    receiver: mpsc::Receiver<Payment>,
}

impl RinhaWorker {
    fn new(receiver: mpsc::Receiver<Payment>) -> Self {
        Self { receiver: receiver }
    }
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                recv = self.receiver.recv() => {
                    if let Some(payment) = recv {
                        dbg!(payment);
                    }
                }
            }
        }
    }
}

impl Rinha {
    async fn payments(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let sender = self.sender.clone();
        let body = http_session.read_request_body().await.unwrap().unwrap();
        let payment = serde_json::from_slice::<Payment>(&body).unwrap();

        sender.send(payment).await.unwrap();

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

fn rinha_worker_service(receiver: mpsc::Receiver<Payment>) -> GenBackgroundService<RinhaWorker> {
    GenBackgroundService::new(
        "Rinha Background Service".to_string(),
        Arc::new(RinhaWorker::new(receiver)),
    )
}

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let (sender, receiver) = mpsc::channel::<Payment>(size_of::<Payment>() * 100);

    let mut rinha = rinha_service(sender);
    rinha.add_tcp("0.0.0.0:9999");

    let rinha_worker = rinha_worker_service(receiver);

    server.add_service(rinha);
    server.add_service(rinha_worker);

    server.run_forever()
}
