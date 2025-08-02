use crate::rinha_domain::Target;
use chrono::{DateTime, Utc};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

type Storage = HashMap<Target, BTreeMap<DateTime<Utc>, f64>>;

static STORAGE: LazyLock<Arc<RwLock<Storage>>> = LazyLock::new(|| {
    let mut hash_map: Storage = HashMap::with_capacity(2);
    hash_map.insert(Target::Default, BTreeMap::new());
    hash_map.insert(Target::Fallback, BTreeMap::new());

    Arc::new(RwLock::new(hash_map))
});

pub fn bootstrap() {
    LazyLock::force(&STORAGE);
}

pub fn get_storage() -> Arc<RwLock<Storage>> {
    STORAGE.clone()
}
