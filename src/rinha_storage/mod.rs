use crate::rinha_balancer::Processor;
use chrono::{DateTime, Utc};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

type Storage = HashMap<Processor, BTreeMap<DateTime<Utc>, f64>>;

static STORAGE: LazyLock<Arc<RwLock<Storage>>> = LazyLock::new(|| {
    let processors = [Processor::Default, Processor::Fallback];
    let mut hash_map: Storage = HashMap::with_capacity(processors.len());

    for processor in processors {
        hash_map.insert(processor, BTreeMap::new());
    }

    Arc::new(RwLock::new(hash_map))
});

pub fn bootstrap() {
    LazyLock::force(&STORAGE);
}

pub fn get_storage() -> Arc<RwLock<Storage>> {
    STORAGE.clone()
}
