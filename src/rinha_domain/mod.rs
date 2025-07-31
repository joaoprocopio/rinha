use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn wrap(date_time: chrono::DateTime<chrono::Utc>) -> Self {
        Self(date_time)
    }
}

impl AsRef<chrono::DateTime<chrono::Utc>> for DateTime {
    fn as_ref(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DateTime(chrono::DateTime::<chrono::Utc>::deserialize(
            deserializer,
        )?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Default,
    Fallback,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaymentRequest {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    #[serde(rename = "amount")]
    pub amount: f64,
    #[serde(rename = "requestedAt", default = "default_requested_at")]
    pub requested_at: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    #[serde(rename = "amount")]
    pub amount: f64,
    #[serde(rename = "requestedAt", default = "default_requested_at")]
    pub requested_at: DateTime,
    #[serde(rename = "target")]
    pub target: Target,
}

fn default_requested_at() -> DateTime {
    DateTime::wrap(chrono::Utc::now())
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
