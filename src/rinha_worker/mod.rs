use crate::{
    rinha_ambulance::{self, Upstream, UpstreamType},
    rinha_chan,
    rinha_domain::{Payment, dt_to_i64},
    rinha_net::{self, JSON_CONTENT_TYPE},
    rinha_storage,
};
use http_body_util::Full;
use hyper::{Method, Request, Uri, body::Bytes, header};
use std::str::FromStr;
use tokio::time::{Duration, sleep};

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

    #[error("no upstream type ext")]
    NoUpstreamTypeExt,
    #[error("request failed")]
    ServerFailed,
    #[error("invalid authority")]
    InvalidAuthority,
}

async fn try_process_payment(payment: &Payment, upstream: &Upstream) -> Result<(), PaymentError> {
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
        .body(Full::<Bytes>::from(payment_ser))?;

    let res = sender.send_request(req).await?;
    let status = res.status();

    if status.is_success() {
        let storage = match upstream_type {
            UpstreamType::Default => rinha_storage::get_default_storage(),
            UpstreamType::Fallback => rinha_storage::get_fallback_storage(),
        };

        let mut storage = storage.write().await;
        storage.insert(dt_to_i64(payment.requested_at), payment.amount);
    }

    if status.is_server_error() {
        return Err(PaymentError::ServerFailed);
    }

    Ok(())
}

async fn process_payment(payment: &Payment) {
    let mut upstream_retry: u32 = 0;
    let mut payment_retry: u32 = 0;

    loop {
        if let Some(upstream) = rinha_ambulance::select().await {
            if let Err(err) = try_process_payment(&payment, &upstream).await {
                if let PaymentError::ServerFailed = err {
                    let health_map = rinha_ambulance::get_health_map();
                    let mut health_map = health_map.write().await;
                    health_map.insert(upstream.hash_addr(), false);

                    let time = std::cmp::min(
                        Duration::from_millis(5) * (1 << payment_retry),
                        Duration::from_secs(10),
                    );
                    sleep(time).await;
                    payment_retry += 1;
                    continue;
                }
            } else {
                break;
            }
        } else {
            let time = std::cmp::min(
                Duration::from_millis(5) * (1 << upstream_retry),
                Duration::from_secs(10),
            );
            sleep(time).await;
            upstream_retry += 1;
            continue;
        }
    }
}

pub async fn task() {
    loop {
        let receiver = rinha_chan::get_receiver();

        if let Ok(Ok(payment)) = tokio::task::spawn_blocking(move || receiver.recv()).await {
            tokio::spawn(async move {
                process_payment(&payment).await;
            });
        } else {
            tracing::error!("unexpected error occurred while recv");
        };
    }
}
