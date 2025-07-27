use std::sync::Arc;

use tokio::sync::broadcast;

use async_trait::async_trait;
use http::{Response, header};
use pingora::{
    apps::http_app::ServeHttp, protocols::http::ServerSession, services::listening::Service,
};

use crate::rinha_domain::Payment;

struct Rinha {
    sender: Arc<broadcast::Sender<Payment>>,
}

impl Rinha {
    fn new(sender: broadcast::Sender<Payment>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

#[async_trait]
impl ServeHttp for Rinha {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let header = http_session.req_header();

        let empty: Vec<u8> = vec![];

        if header.method == "POST" && header.raw_path() == b"/payments" {
            let sender = self.sender.clone();
            let body = http_session.read_request_body().await.unwrap().unwrap();

            sender
                .send(serde_json::from_slice::<Payment>(&body).unwrap())
                .unwrap();

            return Response::builder()
                .status(200)
                .header(header::CONTENT_LENGTH, empty.len())
                .body(empty)
                .unwrap();
        }

        return Response::builder()
            .status(404)
            .header(header::CONTENT_LENGTH, empty.len())
            .body(empty)
            .unwrap();
    }
}

pub fn rinha_service(sender: broadcast::Sender<Payment>) -> Service<Rinha> {
    Service::new("Rinha HTTP Service".to_string(), Rinha::new(sender))
}
