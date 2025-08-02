use crate::{rinha_chan, rinha_core::Result, rinha_domain::Payment};
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
