use serde::{Deserialize, Serialize};
use worker::console_error;

use crate::monitors::types::MonitorError;

#[derive(Serialize)]
pub struct ReconcileResponse {
    pub organizations: usize,
    pub bootstrapped: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorKind {
    Http,
    Tcp,
    Udp,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRequest {
    pub dispatch_id: String,
    pub monitor_id: String,
    pub monitor_url: String,
    pub kind: MonitorKind,
    pub scheduled_for_ts: i64,
    pub timeout_ms: i64,
    pub follow_redirects: bool,
    pub verify_tls: bool,
}

#[derive(Debug)]
pub struct CheckResult {
    pub ok: bool,
    pub status_code: Option<u16>,
    pub rtt_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub error_msg: Option<String>,
    pub colo: String,
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum DispatchError {
    Database { context: &'static str, source: worker::Error },
    CheckFailed(CheckResult),
    Heartbeat(worker::Error),
    Monitor(MonitorError),
}

impl DispatchError {
    pub fn database(context: &'static str, source: worker::Error) -> Self {
        DispatchError::Database { context, source }
    }
}

impl From<worker::Error> for DispatchError {
    fn from(err: worker::Error) -> Self {
        DispatchError::database("dispatch.unknown", err)
    }
}

impl From<DispatchError> for axum::http::StatusCode {
    fn from(err: DispatchError) -> axum::http::StatusCode {
        match err {
            DispatchError::Database { context, source } => {
                console_error!("{context}: {source:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            DispatchError::CheckFailed(result) => {
                console_error!(
                    "dispatch.check.failed: status={} error={:?}",
                    result.status_code.unwrap_or(0),
                    result.error_msg
                );
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            DispatchError::Heartbeat(err) => {
                console_error!("dispatch.heartbeat: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            DispatchError::Monitor(err) => err.into(),
        }
    }
}
