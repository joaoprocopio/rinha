use crate::{rinha_chan, rinha_core::Result, rinha_domain::Payment};
use chrono::{DateTime, TimeZone, Utc};
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Request, Response, StatusCode,
    body::{Buf, Bytes, Incoming},
    header,
};
use std::convert::Infallible;

pub const JSON_CONTENT_TYPE: &str = "application/json";

pub async fn payments(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>> {
    let body = req.collect().await?.aggregate();
    let payment: Payment = serde_json::from_reader(body.reader())?;
    let sender = rinha_chan::get_sender();
    sender.send(payment)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .body(Empty::<Bytes>::new().boxed())
        .map_err(|err| err.into())
}

pub async fn payments_summary(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
    let mut from = Utc
        .timestamp_opt(0, 0)
        .single()
        .ok_or("Unable to parse from")?;
    let mut to = Utc::now();

    dbg!(&from);
    dbg!(&to);
    if let Some(query) = req.uri().query() {
        for param in query.split("&") {
            if let Some(str_dt) = param.strip_prefix("from=") {
                if let Ok(dt) =
                    DateTime::parse_from_rfc3339(str_dt).map(|dt| dt.with_timezone(&Utc))
                {
                    from = dt;
                }
            }

            if let Some(str_dt) = param.strip_prefix("to=") {
                if let Ok(dt) =
                    DateTime::parse_from_rfc3339(str_dt).map(|dt| dt.with_timezone(&Utc))
                {
                    to = dt;
                }
            }
        }
    };

    dbg!(&from);
    dbg!(&to);

    Response::builder()
        .status(StatusCode::OK)
        .body(Full::from("abc").boxed())
        .map_err(|err| err.into())
}

pub fn ping() -> Result<Response<BoxBody<Bytes, Infallible>>> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Full::from("pong").boxed())
        .map_err(|err| err.into())
}

pub fn not_found_error() -> Result<Response<BoxBody<Bytes, Infallible>>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Empty::<Bytes>::new().boxed())
        .map_err(|err| err.into())
}
