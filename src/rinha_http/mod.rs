use crate::{rinha_conf::RINHA_ADDR, rinha_domain::Payment, rinha_worker::TARGET_COUNTER};
use async_trait::async_trait;
use http::{Response, StatusCode, Uri, header};
use pingora::{
    apps::http_app::ServeHttp,
    listeners::TcpSocketOptions,
    protocols::{TcpKeepalive, http::ServerSession},
    services::listening::Service,
};
use std::{str::FromStr, sync::Arc, time::Duration, vec};
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

    fn empty_response_with_status_code<T>(&self, status_code: T) -> Response<Vec<u8>>
    where
        T: TryInto<StatusCode>,
        <T as TryInto<StatusCode>>::Error: Into<http::Error>,
    {
        Response::builder()
            .status(status_code)
            .header(header::CONTENT_LENGTH, 0)
            .body(vec![])
            .unwrap_or_else(|_| self.internal_server_error_response())
    }

    fn internal_server_error_response(&self) -> Response<Vec<u8>> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_LENGTH, 0)
            .body(vec![])
            .unwrap()
    }

    async fn payments(&self, http_session: &mut ServerSession, _uri: Uri) -> Response<Vec<u8>> {
        let sender = Arc::clone(&self.sender);

        let Ok(Some(body)) = http_session.read_request_body().await else {
            debug_assert!(
                false,
                "RinhaHttp::payments: failed while reading request body"
            );
            return self.empty_response_with_status_code(StatusCode::NOT_ACCEPTABLE);
        };

        let Ok(payment) = serde_json::de::from_slice::<Payment>(&body) else {
            debug_assert!(
                false,
                "RinhaHttp::payments: fail while deserializing request body"
            );
            return self.empty_response_with_status_code(StatusCode::BAD_REQUEST);
        };

        if let Err(_) = sender.send(payment).await {
            debug_assert!(false, "RinhaHttp::payments: channel send failed");
            return self.empty_response_with_status_code(StatusCode::SERVICE_UNAVAILABLE);
        }

        self.empty_response_with_status_code(StatusCode::OK)
    }

    async fn payments_summary(
        &self,
        _http_session: &mut ServerSession,
        _uri: Uri,
    ) -> Response<Vec<u8>> {
        let target_counter = TARGET_COUNTER.read().await;
        let Ok(target_counter) = serde_json::ser::to_vec(&*target_counter) else {
            debug_assert!(
                false,
                "RinhaHttp::payments_summary: failed serializing payment"
            );
            return self.empty_response_with_status_code(StatusCode::BAD_REQUEST);
        };

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
            .header(header::CONTENT_LENGTH, target_counter.len())
            .body(target_counter.into())
            .unwrap_or_else(|_| self.internal_server_error_response())
    }
}

#[async_trait]
impl ServeHttp for RinhaHttp {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let header = http_session.req_header();

        let Ok(path) = String::from_utf8(header.raw_path().to_vec()) else {
            debug_assert!(false, "RinhaHttp::response: path is not a valid utf-8");
            return self.empty_response_with_status_code(StatusCode::BAD_REQUEST);
        };
        let Ok(uri) = Uri::from_str(&path) else {
            debug_assert!(false, "RinhaHttp::response: path is not a valid uri");
            return self.empty_response_with_status_code(StatusCode::BAD_REQUEST);
        };

        let response = match (header.method.as_str(), uri.path()) {
            ("POST", "/payments") => self.payments(http_session, uri).await,
            ("GET", "/payments-summary") => self.payments_summary(http_session, uri).await,
            _ => self.empty_response_with_status_code(StatusCode::NOT_FOUND),
        };

        if let Err(_) = http_session.drain_request_body().await {
            debug_assert!(false, "RinhaHttp::response: failed draining request body");
            return self.empty_response_with_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }

        response
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

    http_service.add_tcp_with_settings(RINHA_ADDR.as_str(), socket_options);

    http_service
}
