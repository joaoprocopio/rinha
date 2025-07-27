#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::{sync::Arc, time::Duration};

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
use tokio::{sync::mpsc, time::interval};

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

struct RinhaWorker {
    receiver: mpsc::Receiver<Payment>,
}

#[async_trait]
impl BackgroundService for RinhaWorker {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        let mut period = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    break;
                }
                _ = period.tick() => {
                    println!("passo 1s")
                }
            }
        }
    }
}

impl Rinha {
    fn new(sender: mpsc::Sender<Payment>) -> Self {
        Self { sender: sender }
    }

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
        Arc::new(RinhaWorker { receiver: receiver }),
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
