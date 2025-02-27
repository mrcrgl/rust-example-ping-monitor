use std::net::IpAddr;
use uuid::Uuid;

pub type TargetId = Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Target {
    pub id: TargetId,
    pub address: IpAddr,
}
