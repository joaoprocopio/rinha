use crate::rinha_domain::Backends;
use chrono::{DateTime, Utc};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

type Storage = HashMap<Backends, BTreeMap<DateTime<Utc>, f64>>;

static STORAGE: LazyLock<Arc<RwLock<Storage>>> = LazyLock::new(|| {
    let mut hash_map: Storage = HashMap::with_capacity(2);

    hash_map.insert(Backends::Default, BTreeMap::new());
    hash_map.insert(Backends::Fallback, BTreeMap::new());

    Arc::new(RwLock::new(hash_map))
});

pub fn bootstrap() {
    LazyLock::force(&STORAGE);
}

pub fn get_storage() -> Arc<RwLock<Storage>> {
    STORAGE.clone()
}
