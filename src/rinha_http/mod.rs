use crate::{
    rinha_conf::RINHA_ADDR,
    rinha_domain::{Payment, Target, TargetCounter},
    rinha_storage, rinha_tracing,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::{Method, Response, StatusCode, header};
use pingora::{
    apps::http_app::{HttpServer, ServeHttp},
    listeners::TcpSocketOptions,
    modules::http::compression::ResponseCompressionBuilder,
    protocols::{TcpKeepalive, http::ServerSession},
    services::listening::Service,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use url::form_urlencoded;

pub const JSON_CONTENT_TYPE: &str = "application/json";
const EMPTY_BODY: Vec<u8> = vec![];
const EMPTY_BODY_LEN: usize = 0;

pub struct RinhaHttpApp {
    sender: Arc<mpsc::UnboundedSender<Payment>>,
}

impl RinhaHttpApp {
    fn new(sender: mpsc::UnboundedSender<Payment>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }
}

trait Handlers {
    async fn payments(&self, http_session: &mut ServerSession) -> Response<Vec<u8>>;
    async fn payments_summary(&self, http_session: &mut ServerSession) -> Response<Vec<u8>>;
}

impl Handlers for RinhaHttpApp {
    async fn payments(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let sender = self.sender.clone();

        let Ok(Some(body)) = http_session.read_request_body().await else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::payments),
                "failed while reading request body"
            );
            return empty_response_with_status_code(StatusCode::NOT_ACCEPTABLE);
        };

        let Ok(payment_request) = serde_json::de::from_slice::<Payment>(&body) else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::payments),
                "fail while deserializing request body"
            );
            return empty_response_with_status_code(StatusCode::BAD_REQUEST);
        };

        let handle = pingora_runtime::current_handle();

        handle.spawn(async move { sender.send(payment_request) });

        empty_response_with_status_code(StatusCode::OK)
    }

    async fn payments_summary(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let mut from: Option<DateTime<Utc>> = None;
        let mut to: Option<DateTime<Utc>> = None;

        if let Some(query) = http_session.req_header().uri.query() {
            let query = form_urlencoded::parse(query.as_bytes());

            for (key, value) in query {
                match &*key {
                    "from" => {
                        from = chrono::DateTime::parse_from_rfc3339(&value)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .ok()
                    }
                    "to" => {
                        to = chrono::DateTime::parse_from_rfc3339(&value)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .ok()
                    }
                    _ => (),
                }
            }
        }

        let mut target_counter = TargetCounter::default();
        let storage = rinha_storage::get_storage();
        let storage = storage.read().await;
        let default_storage = storage.get(&Target::Default).unwrap();
        let fallback_storage = storage.get(&Target::Fallback).unwrap();

        if let (Some(from), Some(to)) = (from, to) {
            for (_, amount) in default_storage.range(&from..=&to) {
                target_counter.default.requests += 1;
                target_counter.default.amount += amount;
            }

            for (_, amount) in fallback_storage.range(&from..=&to) {
                target_counter.fallback.requests += 1;
                target_counter.fallback.amount += amount;
            }
        } else {
            for amount in default_storage.values() {
                target_counter.default.requests += 1;
                target_counter.default.amount += amount;
            }

            for amount in fallback_storage.values() {
                target_counter.fallback.requests += 1;
                target_counter.fallback.amount += amount;
            }
        }

        let Ok(target_counter) = serde_json::ser::to_vec(&target_counter) else {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::payments_summary),
                "failed serializing payment"
            );
            return empty_response_with_status_code(StatusCode::BAD_REQUEST);
        };

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
            .header(header::CONTENT_LENGTH, target_counter.len())
            .body(target_counter)
            .unwrap_or_else(|_| empty_response_with_status_code(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

#[async_trait]
impl ServeHttp for RinhaHttpApp {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let header = http_session.req_header();

        let response = match (header.method.clone(), header.uri.path()) {
            (Method::POST, "/payments") => self.payments(http_session).await,
            (Method::GET, "/payments-summary") => self.payments_summary(http_session).await,
            _ => empty_response_with_status_code(StatusCode::NOT_FOUND),
        };

        if http_session.drain_request_body().await.is_err() {
            rinha_tracing::debug!(
                rinha_tracing::type_name_of_val!(&Self::response),
                "failed draining request body"
            );
            return empty_response_with_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }

        response
    }
}

fn empty_response_with_status_code<T>(status_code: T) -> Response<Vec<u8>>
where
    T: TryInto<StatusCode>,
    <T as TryInto<StatusCode>>::Error: Into<http::Error>,
{
    Response::builder()
        .status(status_code)
        .header(header::CONTENT_LENGTH, EMPTY_BODY_LEN)
        .body(EMPTY_BODY)
        .unwrap()
}

pub fn rinha_http_service(
    sender: mpsc::UnboundedSender<Payment>,
) -> Service<HttpServer<RinhaHttpApp>> {
    let mut server = HttpServer::new_app(RinhaHttpApp::new(sender));
    server.add_module(ResponseCompressionBuilder::enable(7));

    let mut service = Service::new("Rinha HTTP Service".into(), server);

    let mut socket_options = TcpSocketOptions::default();
    socket_options.tcp_fastopen = Some(10);
    socket_options.tcp_keepalive = Some(TcpKeepalive {
        idle: Duration::from_secs(60),
        interval: Duration::from_secs(5),
        count: 5,
        #[cfg(target_os = "linux")]
        user_timeout: Duration::from_secs(85),
    });

    service.add_tcp_with_settings(RINHA_ADDR.as_str(), socket_options);

    service
}
