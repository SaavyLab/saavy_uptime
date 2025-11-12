use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreateMonitor {
    pub name: String,
    pub url: String,
    pub interval: i64,
    pub timeout: i64,
    pub verify_tls: bool,
    pub follow_redirects: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Monitor {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub interval_s: i64,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
    pub expect_status_low: Option<i64>,
    pub expect_status_high: Option<i64>,
    pub expect_substring: Option<String>,
    pub headers_json: Option<String>,
    pub tags_json: Option<String>,
    pub enabled: i64,
    pub last_checked_at_ts: Option<i64>,
    pub next_run_at_ts: Option<i64>,
    pub current_status: String,
    pub last_ok: i64,
    pub consecutive_failures: i64,
    pub current_incident_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
