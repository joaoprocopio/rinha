use crate::rinha_domain::Payment;
use fjall as storage;
use rand::Rng;

impl From<&Payment> for storage::UserValue {
    fn from(value: &Payment) -> Self {
        serde_json::ser::to_vec(&value).unwrap().into()
    }
}

impl From<&storage::UserValue> for Payment {
    fn from(value: &storage::UserValue) -> Self {
        serde_json::de::from_slice(&value.to_vec()).unwrap()
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
            requested_at: random_utc_datetime(),
            amount: 19.90,
        };

        items
            .insert(payment.requested_at.timestamp().to_be_bytes(), &payment)
            .unwrap();
    }

    for kv in items.range(
        (chrono::Utc::now() - chrono::Duration::days(30))
            .timestamp()
            .to_be_bytes()..=chrono::Utc::now().timestamp().to_be_bytes(),
    ) {
        let kv = kv.unwrap();
        let requested_at = &*kv.0;
        let requested_at = i64::from_be_bytes(requested_at.try_into().unwrap());
        let requested_at = chrono::DateTime::from_timestamp(requested_at, 0);

        let payment: Payment = (&kv.1).into();
        dbg!(requested_at, payment);
    }
}
