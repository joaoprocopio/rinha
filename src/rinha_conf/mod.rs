#[cfg(not(debug_assertions))]
pub const RINHA_PROD: bool = true;

#[cfg(debug_assertions)]
pub const RINHA_PROD: bool = false;

pub const RINHA_HOST: &str = "0.0.0.0";
pub const RINHA_ADDR: &str = "0.0.0.0:9999";

pub const RINHA_DEFAULT_BACKEND_ADDR: &str = "127.0.0.1:8001";
pub const RINHA_FALLBACK_BACKEND_ADDR: &str = "127.0.0.1:8002";
