use crate::rinha_domain::Payment;
use std::sync::{LazyLock, atomic};
use tokio::sync::{Mutex, mpsc};

pub type PaymentSendError = mpsc::error::SendError<Payment>;
pub type PaymentTrySendError = mpsc::error::TrySendError<Payment>;
pub type PaymentReceiver = mpsc::Receiver<Payment>;
pub type PaymentSender = mpsc::Sender<Payment>;

const CHANNEL_BUFFER: usize = 256 << 8;
const CHANNEL_COUNT: usize = 5;

static COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
static CHANNELS: LazyLock<[(PaymentSender, Mutex<PaymentReceiver>); CHANNEL_COUNT]> =
    LazyLock::new(|| {
        let channels: Vec<(PaymentSender, Mutex<PaymentReceiver>)> = (0..CHANNEL_COUNT)
            .map(|_| {
                let channel = mpsc::channel::<Payment>(CHANNEL_BUFFER);
                (channel.0, Mutex::new(channel.1))
            })
            .collect();

        channels.try_into().expect("failed to convert channels")
    });

pub fn get_sender() -> PaymentSender {
    let idx = COUNTER.fetch_add(1, atomic::Ordering::Relaxed) % CHANNEL_COUNT;
    CHANNELS[idx].0.clone()
}

pub fn get_channels<'a>() -> &'a [(PaymentSender, Mutex<PaymentReceiver>); CHANNEL_COUNT] {
    &CHANNELS
}

pub fn boostrap() {
    LazyLock::force(&CHANNELS);
}
