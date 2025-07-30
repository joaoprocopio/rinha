use fjall as storage;

use crate::rinha_domain::Payment;

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

pub fn setup() {
    let keyspace = storage::Config::new(".storage").open().unwrap();
    let items = keyspace
        .open_partition("payments", storage::PartitionCreateOptions::default())
        .unwrap();

    let payment = serde_json::de::from_str::<Payment>(
        "{\"correlationId\":\"0ef201ce-2995-4ec4-bf8b-3d03f8bfbff1\",\"amount\":19.90}",
    )
    .unwrap();

    items.insert("abc", &payment).unwrap();

    let res = items.get("abc").unwrap().unwrap();
    dbg!(&res);

    let res: Payment = (&res).into();

    dbg!(&res);
}
