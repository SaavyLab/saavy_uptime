
use strum::{Display, EnumString};
use serde::{Deserialize, Serialize};
use worker::{BlobType, console_error};

use crate::{
    auth::membership::MembershipError, bootstrap::types::BootstrapError,
    internal::types::MonitorKind,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpMonitorConfig {
    pub url: String,
    pub interval: i64,
    pub timeout: i64,
    pub verify_tls: bool,
    pub follow_redirects: bool,
}

impl HttpMonitorConfig {
    pub fn new(url: &str, interval: i64, timeout: i64, verify_tls: bool, follow_redirects: bool) -> Self {
        Self {
            url: url.to_string(),
            interval,
            timeout,
            verify_tls,
            follow_redirects,
        }
    }
}

impl Into<String> for HttpMonitorConfig {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreateMonitor {
    pub name: String,
    pub config: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Display, PartialEq, PartialOrd, Eq, EnumString)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum MonitorStatus {
    Up,
    Down,
    Degraded,
    Pending,
}

impl MonitorStatus {
    pub fn is_down(&self) -> bool {
        matches!(self, MonitorStatus::Down | MonitorStatus::Degraded)
    }
}

impl Into<BlobType> for MonitorStatus {
    fn into(self) -> BlobType {
        BlobType::String(self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Monitor {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub enabled: i64,
    pub config: String,
    pub status: String,
    pub last_checked_at: Option<i64>,
    pub last_failed_at: Option<i64>,
    pub first_checked_at: Option<i64>,
    pub rt_ms: Option<i64>,
    pub region: Option<String>,
    pub last_error: Option<String>,
    pub next_run_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<crate::d1c::queries::monitors::GetMonitorByIdRow> for Monitor {
    fn from(row: crate::d1c::queries::monitors::GetMonitorByIdRow) -> Self {
        Monitor {
            id: row.id.unwrap_or_default(),
            org_id: row.org_id,
            name: row.name,
            kind: row.kind,
            enabled: row.enabled,
            config: row.config_json,
            status: row.status,
            last_checked_at: row.last_checked_at,
            last_failed_at: row.last_failed_at,
            first_checked_at: row.first_checked_at,
            rt_ms: row.rt_ms,
            region: row.region,
            last_error: row.last_error,
            next_run_at: row.next_run_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<crate::d1c::queries::monitors::GetMonitorsByOrgIdRow> for Monitor {
    fn from(row: crate::d1c::queries::monitors::GetMonitorsByOrgIdRow) -> Self {
        Monitor {
            id: row.id.unwrap_or_default(),
            org_id: row.org_id,
            name: row.name,
            kind: row.kind,
            enabled: row.enabled,
            config: row.config_json,
            status: row.status,
            last_checked_at: row.last_checked_at,
            last_failed_at: row.last_failed_at,
            first_checked_at: row.first_checked_at,
            rt_ms: row.rt_ms,
            region: row.region,
            last_error: row.last_error,
            next_run_at: row.next_run_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug)]
pub enum MonitorError {
    InvalidStatus(String),
    // pub status_code: u16,
    DbInit(worker::Error),
    DbBind(worker::Error),
    DbRun(worker::Error),
    NotFound,
    Forbidden,
    Bootstrap(BootstrapError),
    Membership(MembershipError),
    NoFieldsToUpdate,
    // pub colo: String,
    // pub extra: Option<serde_json::Value>,
}

impl From<worker::Error> for MonitorError {
    fn from(err: worker::Error) -> Self {
        MonitorError::DbRun(err)
    }
}

impl From<MonitorError> for axum::http::StatusCode {
    fn from(err: MonitorError) -> axum::http::StatusCode {
        match err {
            MonitorError::DbInit(err) => {
                console_error!("monitors.db.init: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            MonitorError::DbBind(err) => {
                console_error!("monitors.db.bind: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            MonitorError::DbRun(err) => {
                console_error!("monitors.db.run: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            MonitorError::NotFound => {
                console_error!("monitors.not.found");
                axum::http::StatusCode::NOT_FOUND
            }
            MonitorError::Forbidden => {
                console_error!("monitors.forbidden");
                axum::http::StatusCode::FORBIDDEN
            }
            MonitorError::Bootstrap(err) => {
                console_error!("monitors.bootstrap: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            MonitorError::Membership(err) => {
                console_error!("monitors.membership: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            MonitorError::NoFieldsToUpdate => {
                console_error!("monitors.no.fields.to.update");
                axum::http::StatusCode::BAD_REQUEST
            }
            MonitorError::InvalidStatus(status) => {
                console_error!("monitors.invalid.status: {status}");
                axum::http::StatusCode::BAD_REQUEST
            }
        }
    }
}

impl Into<String> for MonitorError {
    fn into(self) -> String {
        match self {
            MonitorError::InvalidStatus(status) => format!("monitors.invalid.status: {status}"),
            MonitorError::DbInit(err) => format!("monitors.db.init: {err:?}"),
            MonitorError::DbBind(err) => format!("monitors.db.bind: {err:?}"),
            MonitorError::DbRun(err) => format!("monitors.db.run: {err:?}"),
            MonitorError::NotFound => format!("monitors.not.found"),
            MonitorError::Forbidden => format!("monitors.forbidden"),
            MonitorError::Bootstrap(err) => format!("monitors.bootstrap: {err:?}"),
            MonitorError::Membership(err) => format!("monitors.membership: {err:?}"),
            MonitorError::NoFieldsToUpdate => format!("monitors.no.fields.to.update"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct UpdateMonitor {
    pub name: Option<String>,
    pub kind: Option<MonitorKind>,
    pub url: Option<String>,
    pub interval: Option<i64>,
    pub timeout: Option<i64>,
    pub follow_redirects: Option<bool>,
    pub verify_tls: Option<bool>,
    pub expect_status_low: Option<i64>,
    pub expect_status_high: Option<i64>,
    pub expect_substring: Option<String>,
    pub headers_json: Option<String>,
    pub tags_json: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResult {
    // Identity
    pub monitor_id: String,
    pub org_id: String,

    // Result
    pub timestamp: i64,
    pub status: MonitorStatus,

    // Metrics 
    pub latency_ms: i64,
    pub region: String,
    pub colo: String,

    pub error: Option<String>,

    pub code: Option<u16>,
}