use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub id: Uuid,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEnvelope<T> {
    pub id: Uuid,
    pub collected_at: OffsetDateTime,
    pub capability: String,
    pub payload: T,
}

impl<T> EvidenceEnvelope<T> {
    pub fn new(capability: impl Into<String>, payload: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            collected_at: OffsetDateTime::now_utc(),
            capability: capability.into(),
            payload,
        }
    }
}

pub trait ReadOnlyCollector {
    type Output;

    fn capability(&self) -> &'static str;
    fn collect(&self) -> anyhow::Result<Self::Output>;
}

