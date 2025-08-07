use crate::{
    rinha_ambulance::{self, Upstream, UpstreamType},
    rinha_chan,
    rinha_domain::{Payment, dt_to_i64},
    rinha_net::{self, JSON_CONTENT_TYPE},
    rinha_storage,
};
use http_body_util::Full;
use hyper::{Method, Request, body::Bytes, header};
use tokio::time::{Duration, sleep};

#[derive(thiserror::Error, Debug)]
pub enum PaymentError {
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("http")]
    HTTP(#[from] http::Error),
    #[error("uri")]
    URI(#[from] http::uri::InvalidUri),
    #[error("client")]
    Client(#[from] hyper_util::client::legacy::Error),

    #[error("no upstream type ext")]
    NoUpstreamTypeExt,
    #[error("request failed")]
    ServerFailed,
}

async fn try_process_payment(payment: &Payment, upstream: &Upstream) -> Result<(), PaymentError> {
    let upstream_type = upstream
        .ext
        .get::<rinha_ambulance::UpstreamType>()
        .ok_or_else(|| PaymentError::NoUpstreamTypeExt)?;

    let client = rinha_net::get_client();
    let uri = format!("http://{}/payments", upstream.addr);
    let payment_ser = serde_json::to_string(&payment)?;
    let req = Request::builder()
        .method(Method::POST)
        .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
        .uri(uri)
        .body(Full::<Bytes>::from(payment_ser))?;
    let res = client.request(req).await?;
    let status = res.status();

    if status.is_success() {
        let storage = match upstream_type {
            UpstreamType::Default => rinha_storage::get_default_storage(),
            UpstreamType::Fallback => rinha_storage::get_fallback_storage(),
        };
        let mut storage = storage.write().await;
        storage.insert(dt_to_i64(payment.requested_at), payment.amount);

        return Ok(());
    }

    if status.is_server_error() {
        return Err(PaymentError::ServerFailed);
    }

    Ok(())
}

async fn process_payment(payment: &Payment) {
    let mut upstream_attempt: u32 = 0;
    let mut payment_attempt: u32 = 0;

    loop {
        if let Some(upstream) = rinha_ambulance::select().await {
            if let Err(err) = try_process_payment(&payment, &upstream).await {
                if let PaymentError::ServerFailed = err {
                    let health_map = rinha_ambulance::get_health_map();
                    let mut health_map = health_map.write().await;
                    health_map.insert(upstream.hash_addr(), false);

                    sleep(backoff(
                        Duration::from_millis(10),
                        payment_attempt,
                        Duration::from_secs(5),
                    ))
                    .await;
                    payment_attempt += 1;
                    continue;
                }
            } else {
                break;
            }
        } else {
            sleep(backoff(
                Duration::from_millis(50),
                upstream_attempt,
                Duration::from_secs(10),
            ))
            .await;
            upstream_attempt += 1;
            continue;
        }
    }
}

#[inline]
fn backoff(base: Duration, attempt: u32, max: Duration) -> Duration {
    let exp = 1 << attempt;
    let delay = base * exp;
    std::cmp::min(delay, max)
}

async fn workers() {
    let channels = rinha_chan::get_channels();

    for (_, receiver) in channels {
        tokio::spawn({
            let receiver = receiver;

            async move {
                let mut receiver = receiver.lock().await;

                loop {
                    if let Some(payment) = receiver.recv().await {
                        process_payment(&payment).await;
                    }
                }
            }
        });
    }
}

pub async fn task() {
    workers().await;
}
