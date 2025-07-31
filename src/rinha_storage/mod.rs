use crate::rinha_domain::{DateTime, Payment};
use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle, Slice};
use std::sync::{Arc, LazyLock};

static KEYSPACE: LazyLock<Keyspace> = LazyLock::new(|| Config::new(".storage").open().unwrap());
static HANDLE: LazyLock<Arc<PartitionHandle>> = LazyLock::new(|| {
    Arc::new(
        KEYSPACE
            .open_partition("payments", PartitionCreateOptions::default())
            .unwrap(),
    )
});

impl TryFrom<&Payment> for Slice {
    type Error = serde_json::Error;

    fn try_from(value: &Payment) -> Result<Self, Self::Error> {
        Ok(serde_json::ser::to_vec(&value)?.into())
    }
}

impl TryFrom<&Slice> for Payment {
    type Error = serde_json::Error;

    fn try_from(value: &Slice) -> Result<Self, Self::Error> {
        Ok(serde_json::de::from_slice(value)?)
    }
}

impl TryFrom<&DateTime> for Slice {
    type Error = ();

    fn try_from(value: &DateTime) -> Result<Self, Self::Error> {
        Ok(value.as_ref().timestamp().to_be_bytes().into())
    }
}

impl TryFrom<&Slice> for DateTime {
    type Error = ();

    fn try_from(value: &Slice) -> Result<Self, Self::Error> {
        let Ok(value) = value.as_ref().try_into() else {
            return Err(());
        };
        let value = i64::from_be_bytes(value);
        let Some(value) = chrono::DateTime::from_timestamp(value, 0) else {
            return Err(());
        };

        Ok(DateTime::wrap(value))
    }
}

pub fn bootstrap() {
    LazyLock::force(&KEYSPACE);
    LazyLock::force(&HANDLE);
}

pub fn get_handle() -> Arc<PartitionHandle> {
    HANDLE.clone()
}
