use crate::rinha_domain::Payment;
use async_trait::async_trait;
use http::{Response, StatusCode, header};
use pingora::{
    apps::http_app::ServeHttp, protocols::http::ServerSession, services::listening::Service,
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct RinhaHttp {
    sender: Arc<Sender<Payment>>,
}

impl RinhaHttp {
    fn new(sender: Sender<Payment>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

#[async_trait]
impl ServeHttp for RinhaHttp {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let header = http_session.req_header();

        let empty: Vec<u8> = vec![];
        let empty_len: usize = empty.len();

        match (header.method.as_str(), header.raw_path()) {
            ("POST", b"/payments") => {
                let sender = Arc::clone(&self.sender);
                let body = http_session.read_request_body().await.unwrap().unwrap();
                let payment = serde_json::from_slice::<Payment>(&body).unwrap();

                sender.send(payment).await.unwrap();

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_LENGTH, empty_len)
                    .body(empty)
                    .unwrap()
            }
            _ => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_LENGTH, empty_len)
                .body(empty)
                .unwrap(),
        }
    }
}

pub fn rinha_http_service(sender: Sender<Payment>) -> Service<RinhaHttp> {
    Service::new("Rinha HTTP Service".to_string(), RinhaHttp::new(sender))
}
