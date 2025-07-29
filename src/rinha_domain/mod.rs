use std::{env, sync::LazyLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub static HOST: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_HOST").unwrap_or("0.0.0.0".into()));
pub static PORT: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_PORT").unwrap_or("9999".into()));
pub static ADDR: LazyLock<String> = LazyLock::new(|| format!("{}:{}", *HOST, *PORT));

pub static DEFAULT_BACKEND_ADDR: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_DEFAULT_BACKEND_ADDR").unwrap_or("0.0.0.0:8001".into()));
pub static FALLBACK_BACKEND_ADDR: LazyLock<String> =
    LazyLock::new(|| env::var("RINHA_FALLBACK_BACKEND_ADDR").unwrap_or("0.0.0.0:8002".into()));

#[derive(Clone, Debug)]
pub enum Target {
    Default,
    Fallback,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    #[serde(rename = "amount")]
    pub amount: f32,
    #[serde(rename = "requestedAt", default = "Utc::now")]
    pub requested_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct TargetCounter {
    #[serde(rename = "default")]
    pub default: Count,
    #[serde(rename = "fallback")]
    pub fallback: Count,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct Count {
    #[serde(rename = "totalRequests")]
    pub requests: u32,
    #[serde(rename = "totalAmount")]
    pub amount: f32,
}
