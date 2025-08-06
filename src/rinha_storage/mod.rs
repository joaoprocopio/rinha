use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

type Storage = BTreeMap<i64, f64>;

static DEFAULT_STORAGE: LazyLock<Arc<RwLock<Storage>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Storage::new())));
static FALLBACK_STORAGE: LazyLock<Arc<RwLock<Storage>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Storage::new())));

pub fn bootstrap() {
    LazyLock::force(&DEFAULT_STORAGE);
    LazyLock::force(&FALLBACK_STORAGE);
}

pub fn get_default_storage() -> Arc<RwLock<Storage>> {
    DEFAULT_STORAGE.clone()
}

pub fn get_fallback_storage() -> Arc<RwLock<Storage>> {
    FALLBACK_STORAGE.clone()
}
