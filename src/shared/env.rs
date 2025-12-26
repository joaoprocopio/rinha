use crate::error::Result;
use std::{env, ffi::OsStr, path::PathBuf, str::FromStr};

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

#[derive(Debug)]
pub struct Env {
    pub addr: String,
    pub uds_path: PathBuf,
}

impl Env {
    pub fn env_or_default() -> Result<Self> {
        Ok(Self {
            addr: env_or("RINHA_SERVER_ADDR", "0.0.0.0:8000".into()),
            uds_path: env_or("RINHA_UDS_PATH", "/tmp/rinha/rinha.sock".into()),
        })
    }
}
