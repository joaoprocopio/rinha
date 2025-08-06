use crate::rinha_ambulance::UpstreamType;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

type Storage = HashMap<UpstreamType, BTreeMap<i64, f64>>;

static STORAGE: LazyLock<Arc<RwLock<Storage>>> = LazyLock::new(|| {
    let upstreams = [UpstreamType::Default, UpstreamType::Fallback];
    let mut hash_map: Storage = HashMap::with_capacity(upstreams.len());

    for upstream in upstreams {
        hash_map.insert(upstream, BTreeMap::new());
    }

    Arc::new(RwLock::new(hash_map))
});

pub fn bootstrap() {
    LazyLock::force(&STORAGE);
}

pub fn get_storage() -> Arc<RwLock<Storage>> {
    STORAGE.clone()
}
