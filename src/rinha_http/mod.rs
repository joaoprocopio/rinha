use crate::{rinha_core::Result, rinha_net::BoxBody};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{Response, StatusCode, body::Bytes};
use std::convert::Infallible;

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
