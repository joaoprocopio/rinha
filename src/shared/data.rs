use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    #[serde(rename = "amount")]
    pub amount: f64,
    #[serde(rename = "requestedAt", default = "Utc::now")]
    pub requested_at: DateTime<Utc>,
}
