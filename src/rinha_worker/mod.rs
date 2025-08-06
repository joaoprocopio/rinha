use crate::{
    rinha_ambulance, rinha_chan,
    rinha_core::Result,
    rinha_domain::Payment,
    rinha_net::{self, JSON_CONTENT_TYPE},
    rinha_storage,
};
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, Uri, body::Bytes, header};
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum PaymentError {
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("http")]
    HTTP(#[from] http::Error),
    #[error("uri")]
    URI(#[from] http::uri::InvalidUri),
    #[error("hyper")]
    Hyper(#[from] hyper::Error),
    #[error("tcp")]
    Sender(#[from] rinha_net::CreateTCPSenderError),

    #[error("no healthy upstream")]
    NoHealthyUpstream,
    #[error("no upstream type ext")]
    NoUpstreamTypeExt,
    #[error("unstored upstream type")]
    UnstoredUpstreamType,
    #[error("request failed")]
    RequestFailed,
    #[error("invalid authority")]
    InvalidAuthority,
}

async fn try_process_payment(payment: Payment) -> Result<(), PaymentError> {
    let upstream = rinha_ambulance::select()
        .await
        .ok_or_else(|| PaymentError::NoHealthyUpstream)?;
    let upstream_type = upstream
        .ext
        .get::<rinha_ambulance::UpstreamType>()
        .ok_or_else(|| PaymentError::NoUpstreamTypeExt)?;

    let mut sender = rinha_net::create_tcp_socket_sender(upstream.addr).await?;
    let uri = format!("http://{}/payments", upstream.addr);
    let uri = Uri::from_str(uri.as_str())?;
    let authority = uri
        .authority()
        .ok_or_else(|| PaymentError::InvalidAuthority)?;

    let payment_ser = serde_json::to_string(&payment)?;
    let req = Request::builder()
        .method(Method::POST)
        .header(header::HOST, authority.as_str())
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .uri(uri)
        .body(Full::<Bytes>::from(payment_ser).boxed())?;

    let res = sender.send_request(req).await?;
    let status = res.status();

    if status.is_success() {
        let storage = rinha_storage::get_storage();
        let mut storage = storage.write().await;
        let storage = storage
            .get_mut(&upstream_type)
            .ok_or_else(|| PaymentError::UnstoredUpstreamType)?;
        storage.insert(payment.requested_at, payment.amount);
    }

    if status.is_server_error() {
        return Err(PaymentError::RequestFailed);
    }

    Ok(())
}

async fn process_payment(payment: Payment) {
    if let Err(err) = try_process_payment(payment).await {
        tracing::error!(?err);
    }
}

pub async fn task() {
    let receiver = rinha_chan::get_receiver();
    let mut receiver = receiver.lock().await;

    loop {
        tokio::select! {
            Some(payment) = receiver.recv() => {
                tokio::spawn(async move {
                    process_payment(payment).await;
                })
            }
        };
    }
}
