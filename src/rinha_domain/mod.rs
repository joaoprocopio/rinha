use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Target {
    Default,
    Fallback,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    #[serde(rename = "amount")]
    pub amount: f32,
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
    pub requests: u32,
    #[serde(rename = "totalAmount")]
    pub amount: f32,
}
