use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    correlation_id: String,
    amount: f64,
}
