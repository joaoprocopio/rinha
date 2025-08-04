use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    #[serde(rename = "amount")]
    pub amount: f64,
    #[serde(rename = "requestedAt", default = "Utc::now")]
    pub requested_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TargetCounter {
    #[serde(rename = "default")]
    pub default: Count,
    #[serde(rename = "fallback")]
    pub fallback: Count,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Count {
    #[serde(rename = "totalRequests")]
    pub requests: u64,
    #[serde(rename = "totalAmount")]
    pub amount: f64,
}
