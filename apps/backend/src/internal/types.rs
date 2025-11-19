use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use worker::console_error;

use crate::monitors::types::{HeartbeatResult, MonitorError};

#[derive(Serialize)]
pub struct ReconcileResponse {
    pub organizations: usize,
    pub bootstrapped: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorKind {
    Http,
    Tcp,
    Udp,
}

impl Display for MonitorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorKind::Http => write!(f, "http"),
            MonitorKind::Tcp => write!(f, "tcp"),
            MonitorKind::Udp => write!(f, "udp"),
        }
    }
}

impl FromStr for MonitorKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "http" => Ok(MonitorKind::Http),
            "tcp" => Ok(MonitorKind::Tcp),
            "udp" => Ok(MonitorKind::Udp),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRequest {
    pub dispatch_id: String,
    pub monitor_id: String,
    pub org_id: String,
    pub monitor_url: String,
    pub kind: MonitorKind,
    pub scheduled_for_ts: i64,
    pub timeout_ms: i64,
    pub follow_redirects: bool,
    pub verify_tls: bool,
    pub status: Option<String>,
    pub first_checked_at: Option<i64>,
    pub last_failed_at: Option<i64>,
}

#[derive(Debug)]
pub enum DispatchError {
    Database {
        context: &'static str,
        source: worker::Error,
    },
    CheckFailed(HeartbeatResult),
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
                    result.status.to_string(),
                    result.error,
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

impl From<DispatchError> for String {
    fn from(err: DispatchError) -> Self {
        match err {
            DispatchError::Database { context, source } => format!("{context}: {source:?}"),
            DispatchError::CheckFailed(result) => format!(
                "dispatch.check.failed: status={} error={:?}",
                result.status, result.error
            ),
            DispatchError::Heartbeat(err) => format!("dispatch.heartbeat: {err:?}"),
            DispatchError::Monitor(err) => err.into(),
        }
    }
}
