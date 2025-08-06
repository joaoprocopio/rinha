use crate::{
    rinha_ambulance::UpstreamType,
    rinha_chan,
    rinha_domain::{Payment, TargetCounter},
    rinha_net::JSON_CONTENT_TYPE,
    rinha_storage,
};
use chrono::{DateTime, TimeZone, Utc};
use http_body_util::{BodyExt, Full};
use hyper::{
    Request, Response, StatusCode,
    body::{Buf, Bytes, Incoming},
    header,
};

#[derive(thiserror::Error, Debug)]
pub enum PaymentsError {
    #[error("hyper")]
    Hyper(#[from] hyper::Error),
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("http")]
    HTTP(#[from] http::Error),
    #[error("send")]
    Send(#[from] rinha_chan::PaymentSendError),
    #[error("spawn")]
    Spawn(#[from] tokio::task::JoinError),
}

pub async fn payments(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, PaymentsError> {
    let body = req.collect().await?.aggregate();
    let payment = serde_json::from_reader::<_, Payment>(body.reader())?;

    let _ = tokio::task::spawn_blocking(move || {
        let sender = rinha_chan::get_sender();
        sender.send(payment)
    })
    .await??;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::new()))?)
}

#[derive(thiserror::Error, Debug)]
pub enum PaymentsSummaryError {
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("http")]
    HTTP(#[from] http::Error),
    #[error("hyper")]
    Hyper(#[from] hyper::Error),

    #[error("infallible")]
    Infallible,
    #[error("storage failed")]
    StorageFailed,
}

pub async fn payments_summary(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, PaymentsSummaryError> {
    let mut from = Utc
        .timestamp_opt(0, 0)
        .single()
        .ok_or_else(|| PaymentsSummaryError::Infallible)?;
    let mut to = Utc::now();

    if let Some(query) = req.uri().query() {
        for param in query.split("&") {
            if let Some(dt) = param.strip_prefix("from=") {
                if let Ok(dt) = DateTime::parse_from_rfc3339(dt).map(|dt| dt.with_timezone(&Utc)) {
                    from = dt;
                }
            } else if let Some(dt) = param.strip_prefix("to=") {
                if let Ok(dt) = DateTime::parse_from_rfc3339(dt).map(|dt| dt.with_timezone(&Utc)) {
                    to = dt;
                }
            }
        }
    };

    let mut target_counter = TargetCounter::default();

    let storage = rinha_storage::get_storage();
    let storage = storage.read().await;

    let default_storage = storage
        .get(&UpstreamType::Default)
        .ok_or_else(|| PaymentsSummaryError::StorageFailed)?;
    let fallback_storage = storage
        .get(&UpstreamType::Fallback)
        .ok_or_else(|| PaymentsSummaryError::StorageFailed)?;

    for (_, amount) in default_storage.range(from..=to) {
        target_counter.default.requests += 1;
        target_counter.default.amount += amount;
    }

    for (_, amount) in fallback_storage.range(from..=to) {
        target_counter.fallback.requests += 1;
        target_counter.fallback.amount += amount;
    }

    let body = serde_json::to_vec(&target_counter)?;

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .status(StatusCode::OK)
        .body(Full::new(body.into()))?)
}

#[derive(thiserror::Error, Debug)]
pub enum NotFoundError {
    #[error("http")]
    HTTP(#[from] http::Error),
}

pub async fn not_found() -> Result<Response<Full<Bytes>>, NotFoundError> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::new()))?)
}
