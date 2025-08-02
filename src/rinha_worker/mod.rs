use std::str::FromStr;

use crate::{
    rinha_chan, rinha_conf,
    rinha_core::Result,
    rinha_domain::{Backends, Payment},
    rinha_storage,
};
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, Uri, body::Bytes, client::conn::http1, header};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

async fn process_payment(payment: Payment) -> Result<()> {
    let stream = TcpStream::connect(rinha_conf::RINHA_DEFAULT_BACKEND_ADDR.as_str()).await?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = http1::handshake(io).await?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!(?err, "connection error");
        }
    });

    let uri = format!(
        "http://{}/payments",
        *rinha_conf::RINHA_DEFAULT_BACKEND_ADDR
    );
    let uri = Uri::from_str(uri.as_str())?;
    let authority = uri.authority().ok_or("Unable to get authority")?;

    let payment_ser = serde_json::to_string(&payment)?;

    let req = Request::builder()
        .method(Method::POST)
        .header(header::HOST, authority.as_str())
        .uri(uri)
        .body(Full::<Bytes>::from(payment_ser).boxed())?;

    let res = sender.send_request(req).await?;

    dbg!(res.headers());

    let storage = rinha_storage::get_storage();
    let mut storage = storage.write().await;
    let storage = storage
        .get_mut(&Backends::Default)
        .ok_or("Unable to get mutable reference to storage")?;
    storage.insert(payment.requested_at, payment.amount);

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
