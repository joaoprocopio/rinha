use std::str::FromStr;

use crate::{rinha_chan, rinha_conf, rinha_core::Result, rinha_domain::Payment};
use http_body_util::{BodyExt, Empty};
use hyper::{
    Method, Request, Uri,
    body::{Buf, Bytes},
    client::conn::http1,
    header,
};
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
        "http://{}/admin/payments-summary",
        *rinha_conf::RINHA_DEFAULT_BACKEND_ADDR
    );
    let uri = Uri::from_str(uri.as_str())?;
    dbg!(&uri);
    let authority = uri.authority().ok_or("Unable to get authority")?;
    dbg!(&authority);

    let req = Request::builder()
        .method(Method::GET)
        .header(header::HOST, authority.as_str())
        .uri(uri)
        .header("X-Rinha-Token", "123")
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(req).await?;

    dbg!(res.headers());

    let body = res.collect().await?;
    let body = body.aggregate();
    let body = body.chunk();
    let body = String::from_utf8_lossy(body);

    dbg!(&body);

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
