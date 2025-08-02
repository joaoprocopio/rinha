use crate::{rinha_chan, rinha_core::Result, rinha_domain::Payment};

async fn process_payment(payment: Payment) -> Result<()> {
    // https://github.com/hyperium/hyper/blob/master/examples/client_json.rs
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
                        tracing::error!(?err);
                    }
                })
            }
        };
    }
}
