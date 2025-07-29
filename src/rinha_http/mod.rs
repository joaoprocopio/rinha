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
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;

pub const JSON_CONTENT_TYPE: &'static str = "application/json";

const EMPTY_BODY: Vec<u8> = vec![];
const EMPTY_BODY_LEN: usize = 0;

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
        match http_session.read_request().await {
            Ok(true) => (),
            Ok(false) => return Response::new(EMPTY_BODY),
            _ => return bad_request(),
        }

        let header = http_session.req_header();
        let response = match (header.method.as_str(), header.raw_path()) {
            ("POST", b"/payments") => payments(http_session, Arc::clone(&self.sender)).await,
            ("GET", b"/payments-summary") => payments_summary(http_session).await,
            _ => not_found(),
        };

        if let Err(_) = http_session.drain_request_body().await {
            return Response::new(EMPTY_BODY);
        }

        response
    }
}

fn internal_server_error() -> Response<Vec<u8>> {
    Response::new(b"Internal Server Error".into())
}
fn bad_request() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
        .body(EMPTY_BODY)
        .unwrap_or_else(|_| internal_server_error())
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
        .body(EMPTY_BODY)
        .unwrap_or_else(|_| internal_server_error())
}

async fn payments_summary(_http_session: &mut ServerSession) -> Response<Vec<u8>> {
    let target_counter = TARGET_COUNTER.read().await;
    let target_count = match serde_json::ser::to_vec(&*target_counter) {
        Ok(target_count) => target_count,
        _ => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
                .body(EMPTY_BODY)
                .unwrap_or_else(|_| internal_server_error());
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .header(header::CONTENT_LENGTH, target_count.len())
        .body(target_count.into())
        .unwrap_or_else(|_| internal_server_error())
}

async fn payments(
    http_session: &mut ServerSession,
    sender: Arc<mpsc::Sender<Payment>>,
) -> Response<Vec<u8>> {
    let body = match http_session.read_request_body().await {
        Ok(Some(body)) => body,
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_ACCEPTABLE)
                .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
                .body(EMPTY_BODY)
                .unwrap_or_else(|_| internal_server_error());
        }
    };

    let payment = match serde_json::de::from_slice::<Payment>(&body) {
        Ok(payment) => payment,
        _ => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
                .body(EMPTY_BODY)
                .unwrap_or_else(|_| internal_server_error());
        }
    };

    if let Err(_) = sender.send(payment).await {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
            .body(EMPTY_BODY)
            .unwrap_or_else(|_| internal_server_error());
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
        .body(EMPTY_BODY)
        .unwrap_or_else(|_| internal_server_error())
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
