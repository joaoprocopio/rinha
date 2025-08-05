use crate::{
    rinha_balancer::{self, Processor},
    rinha_chan,
    rinha_core::Result,
    rinha_domain::Payment,
    rinha_net::JSON_CONTENT_TYPE,
    rinha_storage,
};
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, Uri, body::Bytes, client::conn::http1, header};
use hyper_util::rt::TokioIo;
use std::str::FromStr;
use tokio::net::TcpStream;

async fn process_payment(payment: Payment) -> Result<()> {
    // TODO: aqui eu vou precisar do host que vai virar a uri
    // TODO: vou precisar saber se é o target ou o fallback
    let upstream = rinha_balancer::select()
        .await
        .ok_or_else(|| "Failed to get healhy upstream")?;
    let processor = upstream
        .ext
        .get::<Processor>()
        .ok_or_else(|| "No Processor enum field is found")?;

    let stream = TcpStream::connect(upstream.addr).await?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = http1::handshake(io).await?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!(?err, "Connection error");
        }
    });

    let uri = format!("http://{}/payments", upstream.addr);
    let uri = Uri::from_str(uri.as_str())?;
    let authority = uri.authority().ok_or_else(|| "Unable to get authority")?;

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
            .get_mut(&processor)
            .ok_or_else(|| "Unable to get mutable reference to storage")?;
        storage.insert(payment.requested_at, payment.amount);
    }

    if status.is_server_error() {
        // balancer.set_health(&upstream, false);
        // TODO: força um retry pra não ter drop de processamento
    }

    Ok(())
}

pub async fn task() {
    let receiver = rinha_chan::get_receiver();
    let mut receiver = receiver.lock().await;

    loop {
        tokio::select! {
            Some(payment) = receiver.recv() => {
                tokio::spawn(async move {
                    if let Err(err) = process_payment(payment).await {
                        tracing::error!(?err, "error while processing payment");
                    }
                })
            }
        };
    }
}
