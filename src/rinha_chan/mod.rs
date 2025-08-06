use crate::rinha_domain::Payment;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, mpsc};

pub type PaymentSendError = mpsc::error::SendError<Payment>;
pub type PaymentReceiver = mpsc::Receiver<Payment>;
pub type PaymentSender = mpsc::Sender<Payment>;

const CHANNEL_BUFFER: usize = ((size_of::<Payment>() as f64) * 1024.0 * 3.125) as usize; // 128 kB

static CHANNEL: LazyLock<(Arc<PaymentSender>, Arc<Mutex<PaymentReceiver>>)> = LazyLock::new(|| {
    let channel = mpsc::channel::<Payment>(CHANNEL_BUFFER);

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
