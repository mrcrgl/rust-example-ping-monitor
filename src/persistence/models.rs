use std::collections::VecDeque;
use std::net::IpAddr;
use uuid::Uuid;

pub type TargetId = Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TargetTableEntry {
    pub target: Target,
    pub probe_results: VecDeque<TargetProbeResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Target {
    pub id: TargetId,
    pub address: IpAddr,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TargetProbeResult {
    pub probe_time: chrono::DateTime<chrono::Utc>,
    pub took: i64,
    #[serde(flatten)]
    pub result: ProbeStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProbeStatus {
    Ok { rtt: i64 },
    Timeout,
    Failure { reason: String },
}
