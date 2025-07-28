use crate::{rinha_domain::Payment, rinha_worker::TARGET_COUNTER};
use async_trait::async_trait;
use http::{Response, StatusCode, header};
use pingora::{
    apps::http_app::ServeHttp,
    listeners::TcpSocketOptions,
    protocols::{TcpKeepalive, http::ServerSession},
    services::listening::Service,
};
use std::{sync::Arc, time::Duration};
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
                let payment = serde_json::de::from_slice::<Payment>(&body).unwrap();

                sender.send(payment).await.unwrap();

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_LENGTH, empty_len)
                    .body(empty)
                    .unwrap()
            }
            ("GET", b"/payments-summary") => {
                let target_counter = TARGET_COUNTER.read().await;
                let target_count = serde_json::ser::to_vec(&*target_counter).unwrap();

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::CONTENT_LENGTH, target_count.len())
                    .body(target_count.into())
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
    let mut http_service = Service::new("Rinha HTTP Service".into(), RinhaHttp::new(sender));
    let mut socket_options = TcpSocketOptions::default();
    socket_options.tcp_fastopen = Some(10);
    socket_options.tcp_keepalive = Some(TcpKeepalive {
        idle: Duration::from_secs(60),
        interval: Duration::from_secs(5),
        count: 5,
        #[cfg(target_os = "linux")]
        user_timeout: Duration::from_secs(85),
    });

    http_service.add_tcp_with_settings("0.0.0.0:9999", socket_options);

    http_service
}
