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
use tokio::sync::mpsc;

pub struct RinhaHttp {
    sender: Arc<mpsc::Sender<Payment>>,
}

impl RinhaHttp {
    fn new(sender: mpsc::Sender<Payment>) -> Self {
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
                let body = match http_session.read_request_body().await {
                    Ok(Some(body)) => body,
                    Ok(None) => {
                        return Response::builder()
                            .status(StatusCode::NOT_ACCEPTABLE)
                            .header(header::CONTENT_LENGTH, empty_len)
                            .body(empty)
                            .unwrap_or_else(|_| Response::new("Internal Server Error".into()));
                    }
                    Err(_) => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_LENGTH, empty_len)
                            .body(empty)
                            .unwrap_or_else(|_| Response::new("Internal Server Error".into()));
                    }
                };

                let payment = match serde_json::de::from_slice::<Payment>(&body) {
                    Ok(payment) => payment,
                    Err(_) => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_LENGTH, empty_len)
                            .body(empty)
                            .unwrap_or_else(|_| Response::new("Internal Server Error".into()));
                    }
                };

                let sender = Arc::clone(&self.sender);

                if let Err(_) = sender.send(payment).await {
                    return Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .header(header::CONTENT_LENGTH, empty_len)
                        .body(empty)
                        .unwrap_or_else(|_| Response::new("Internal Server Error".into()));
                }

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_LENGTH, empty_len)
                    .body(empty)
                    .unwrap_or_else(|_| Response::new("Internal Server Error".into()))
            }
            ("GET", b"/payments-summary") => {
                let target_counter = TARGET_COUNTER.read().await;
                let target_count = match serde_json::ser::to_vec(&*target_counter) {
                    Ok(target_count) => target_count,
                    Err(_) => {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_LENGTH, empty_len)
                            .body(empty)
                            .unwrap_or_else(|_| Response::new("Internal Server Error".into()));
                    }
                };

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::CONTENT_LENGTH, target_count.len())
                    .body(target_count.into())
                    .unwrap_or_else(|_| Response::new("Internal Server Error".into()))
            }
            _ => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_LENGTH, empty_len)
                .body(empty)
                .unwrap_or_else(|_| Response::new("Internal Server Error".into())),
        }
    }
}

pub fn rinha_http_service(sender: mpsc::Sender<Payment>) -> Service<RinhaHttp> {
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
