use crate::{rinha_core::Result, rinha_domain::Payment, rinha_net::BoxBody};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    Request, Response, StatusCode,
    body::{Buf, Bytes, Incoming},
    header,
};
use std::convert::Infallible;

pub const JSON_CONTENT_TYPE: &str = "application/json";

pub async fn payments(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>> {
    let body = req.collect().await?.aggregate();
    let mut payment: Payment = serde_json::from_reader(body.reader())?;

    payment.amount += 10.0;

    let payment = serde_json::to_string(&payment)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .body(Full::new(payment.into()).boxed())
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
