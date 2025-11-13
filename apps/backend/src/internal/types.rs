use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ReconcileResponse {
    pub organizations: usize,
    pub bootstrapped: usize,
    pub failed: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRequest {
    pub dispatch_id: String,
    pub monitor_id: String,
    pub scheduled_for_ts: i64,
}
