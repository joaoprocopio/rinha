use std::{env, sync::LazyLock};

pub static RINHA_HOST: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_HOST").unwrap_or("0.0.0.0".into()));
pub static RINHA_PORT: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_PORT").unwrap_or("9999".into()));
pub static RINHA_ADDR: LazyLock<String> =
    LazyLock::new(|| format!("{}:{}", *RINHA_HOST, *RINHA_PORT));

pub static RINHA_DEFAULT_UPSTREAM_HOST: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_DEFAULT_UPSTREAM_HOST").unwrap_or("127.0.0.1".into()));
pub static RINHA_DEFAULT_UPSTREAM_PORT: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_DEFAULT_UPSTREAM_PORT").unwrap_or("8001".into()));
pub static RINHA_DEFAULT_UPSTREAM_ADDR: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}:{}",
        *RINHA_DEFAULT_UPSTREAM_HOST, *RINHA_DEFAULT_UPSTREAM_PORT
    )
});

pub static RINHA_FALLBACK_UPSTREAM_HOST: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_FALLBACK_UPSTREAM_HOST").unwrap_or("127.0.0.1".into()));
pub static RINHA_FALLBACK_UPSTREAM_PORT: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_FALLBACK_UPSTREAM_PORT").unwrap_or("8002".into()));
pub static RINHA_FALLBACK_UPSTREAM_ADDR: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}:{}",
        *RINHA_FALLBACK_UPSTREAM_HOST, *RINHA_FALLBACK_UPSTREAM_PORT
    )
});

pub fn bootstrap() {
    LazyLock::force(&RINHA_HOST);
    LazyLock::force(&RINHA_PORT);
    LazyLock::force(&RINHA_ADDR);
    LazyLock::force(&RINHA_DEFAULT_UPSTREAM_HOST);
    LazyLock::force(&RINHA_DEFAULT_UPSTREAM_PORT);
    LazyLock::force(&RINHA_DEFAULT_UPSTREAM_ADDR);
    LazyLock::force(&RINHA_FALLBACK_UPSTREAM_HOST);
    LazyLock::force(&RINHA_FALLBACK_UPSTREAM_PORT);
    LazyLock::force(&RINHA_FALLBACK_UPSTREAM_ADDR);
}
