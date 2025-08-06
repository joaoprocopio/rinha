use crate::rinha_domain::Payment;
use crossbeam::channel;
use std::sync::LazyLock;

pub type PaymentSendError = channel::SendError<Payment>;
pub type PaymentReceiver = channel::Receiver<Payment>;
pub type PaymentSender = channel::Sender<Payment>;

const CHANNEL_BUFFER: usize = size_of::<Payment>() * (8 << 9) as usize;

static CHANNEL: LazyLock<(PaymentSender, PaymentReceiver)> = LazyLock::new(|| {
    let channel = channel::bounded::<Payment>(CHANNEL_BUFFER);

    (channel.0, channel.1)
});

pub fn get_sender() -> PaymentSender {
    CHANNEL.0.clone()
}

pub fn get_receiver() -> PaymentReceiver {
    CHANNEL.1.clone()
}

pub fn boostrap() {
    LazyLock::force(&CHANNEL);
}
