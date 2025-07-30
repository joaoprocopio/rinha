use crate::rinha_domain::{DateTime, Payment};
use fjall as storage;
use rand::Rng;

impl From<&Payment> for fjall::Slice {
    fn from(value: &Payment) -> Self {
        serde_json::ser::to_vec(value).unwrap().into()
    }
}

impl From<&fjall::Slice> for Payment {
    fn from(value: &fjall::Slice) -> Self {
        serde_json::de::from_slice(value).unwrap()
    }
}

impl From<&DateTime> for fjall::Slice {
    fn from(value: &DateTime) -> Self {
        value.as_ref().timestamp().to_be_bytes().into()
    }
}

impl From<&fjall::Slice> for DateTime {
    fn from(value: &fjall::Slice) -> Self {
        let value = value.as_ref();
        let value = i64::from_be_bytes(value.try_into().unwrap());
        let value = chrono::DateTime::from_timestamp(value, 0).unwrap();

        DateTime::wrap(value)
    }
}

fn random_utc_datetime() -> chrono::DateTime<chrono::Utc> {
    let mut rng = rand::rng();
    let now = chrono::Utc::now();
    let five_years_ago = now - chrono::Duration::days(365 * 5);
    let offset = rng.random_range(five_years_ago.timestamp()..now.timestamp());
    let rand_date = chrono::DateTime::from_timestamp(offset, 0).unwrap();

    rand_date
}

pub fn setup() {
    let keyspace = storage::Config::new(".storage").open().unwrap();
    let items = keyspace
        .open_partition("payments", storage::PartitionCreateOptions::default())
        .unwrap();

    for _ in 1..=1000 {
        let payment = Payment {
            correlation_id: uuid::Uuid::new_v4(),
            requested_at: DateTime::wrap(random_utc_datetime()),
            amount: 19.90,
        };

        items.insert(&payment.requested_at, &payment).unwrap();
    }

    let start_ts = DateTime::wrap(chrono::Utc::now() - chrono::Duration::days(30));
    let end_ts = DateTime::wrap(chrono::Utc::now());

    let start_key: fjall::Slice = (&start_ts).into();
    let end_key: fjall::Slice = (&end_ts).into();

    for kv in items.range(start_key..=end_key) {
        let kv = kv.unwrap();
        let requested_at: DateTime = (&kv.0).into();
        let payment: Payment = (&kv.1).into();
        dbg!(requested_at, payment);
    }
}
