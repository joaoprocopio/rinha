use crate::rinha_domain::Payment;
use std::sync::LazyLock;
use tokio::sync::{Mutex, mpsc};

pub type PaymentSendError = mpsc::error::SendError<Payment>;
pub type PaymentTrySendError = mpsc::error::TrySendError<Payment>;
pub type PaymentReceiver = mpsc::Receiver<Payment>;
pub type PaymentSender = mpsc::Sender<Payment>;

const CHANNEL_BUFFER: usize = 256 << 8;

static CHANNEL: LazyLock<(PaymentSender, Mutex<PaymentReceiver>)> = LazyLock::new(|| {
    let channel = mpsc::channel::<Payment>(CHANNEL_BUFFER);

    (channel.0, Mutex::new(channel.1))
});

pub fn get_sender() -> PaymentSender {
    CHANNEL.0.clone()
}

pub fn get_receiver() -> &'static Mutex<PaymentReceiver> {
    &CHANNEL.1
}

pub fn boostrap() {
    LazyLock::force(&CHANNEL);
}
