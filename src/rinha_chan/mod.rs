use crate::rinha_domain::Payment;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, mpsc};

pub type PaymentSendError = mpsc::error::SendError<Payment>;
pub type PaymentReceiver = mpsc::UnboundedReceiver<Payment>;
pub type PaymentSender = mpsc::UnboundedSender<Payment>;

static CHANNEL: LazyLock<(Arc<PaymentSender>, Arc<Mutex<PaymentReceiver>>)> = LazyLock::new(|| {
    let channel = mpsc::unbounded_channel::<Payment>();

    (Arc::new(channel.0), Arc::new(Mutex::new(channel.1)))
});

pub fn get_sender() -> Arc<PaymentSender> {
    CHANNEL.0.clone()
}

pub fn get_receiver() -> Arc<Mutex<PaymentReceiver>> {
    CHANNEL.1.clone()
}

pub fn boostrap() {
    LazyLock::force(&CHANNEL);
}
