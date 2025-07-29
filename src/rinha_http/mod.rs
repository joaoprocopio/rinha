use crate::{
    rinha_domain::{ADDR, Payment},
    rinha_worker::TARGET_COUNTER,
};
use async_trait::async_trait;
use http::{Response, StatusCode, header};
use pingora::{
    apps::http_app::ServeHttp,
    listeners::TcpSocketOptions,
    protocols::{TcpKeepalive, http::ServerSession},
    services::listening::Service,
};
use std::{sync::Arc, time::Duration, vec};
use tokio::sync::mpsc;

pub const JSON_CONTENT_TYPE: &'static str = "application/json";

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

        let response = match (header.method.as_str(), header.raw_path()) {
            ("POST", b"/payments") => payments(http_session, Arc::clone(&self.sender)).await,
            ("GET", b"/payments-summary") => payments_summary(http_session).await,
            _ => empty_response_with_status_code(StatusCode::NOT_FOUND),
        };

        if let Err(_) = http_session.drain_request_body().await {
            return empty_response_with_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }

        response
    }
}

fn internal_server_error_response() -> Response<Vec<u8>> {
    Response::new("Internal Server Error".into())
}

fn empty_response_with_status_code<T>(status_code: T) -> Response<Vec<u8>>
where
    T: TryInto<StatusCode>,
    <T as TryInto<StatusCode>>::Error: Into<http::Error>,
{
    Response::builder()
        .status(status_code)
        .header(header::CONTENT_LENGTH, 0)
        .body(vec![])
        .unwrap_or_else(|_| internal_server_error_response())
}

async fn payments_summary(_http_session: &mut ServerSession) -> Response<Vec<u8>> {
    let target_counter = TARGET_COUNTER.read().await;
    let Ok(target_count) = serde_json::ser::to_vec(&*target_counter) else {
        return empty_response_with_status_code(StatusCode::BAD_REQUEST);
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .header(header::CONTENT_LENGTH, target_count.len())
        .body(target_count.into())
        .unwrap_or_else(|_| internal_server_error_response())
}

async fn payments(
    http_session: &mut ServerSession,
    sender: Arc<mpsc::Sender<Payment>>,
) -> Response<Vec<u8>> {
    let Ok(Some(body)) = http_session.read_request_body().await else {
        return empty_response_with_status_code(StatusCode::NOT_ACCEPTABLE);
    };

    let Ok(payment) = serde_json::de::from_slice::<Payment>(&body) else {
        return empty_response_with_status_code(StatusCode::BAD_REQUEST);
    };

    if let Err(_) = sender.send(payment).await {
        return empty_response_with_status_code(StatusCode::SERVICE_UNAVAILABLE);
    }

    empty_response_with_status_code(StatusCode::OK)
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

    http_service.add_tcp_with_settings(ADDR.as_str(), socket_options);

    http_service
}
