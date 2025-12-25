use std::{env, ffi::OsStr, str::FromStr};

pub fn env_or<K, T>(key: K, default: T) -> T
where
    K: AsRef<OsStr>,
    T: FromStr,
{
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| default)
}
