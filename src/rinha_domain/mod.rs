use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

pub type UTC = chrono::Utc;
pub type UTCDateTime = chrono::DateTime<UTC>;

#[derive(Debug)]
pub struct Timestamp(UTCDateTime);

impl Timestamp {
    pub fn new(dt: UTCDateTime) -> Self {
        Self(dt)
    }

    pub fn now() -> Self {
        Self(UTC::now())
    }
}

impl Deref for Timestamp {
    type Target = UTCDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Timestamp(UTCDateTime::deserialize(deserializer)?))
    }
}

#[derive(Debug, Clone)]
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
    #[serde(rename = "requestedAt", default = "Timestamp::now")]
    pub requested_at: Timestamp,
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
