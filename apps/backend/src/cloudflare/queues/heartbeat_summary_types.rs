use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatSummary {
    pub monitor_id: String,
    pub org_id: String,
    pub ts: i64,
    pub ok: i64,
    pub code: Option<u16>,
    pub rtt_ms: Option<i64>,
    pub region: Option<String>,
}
