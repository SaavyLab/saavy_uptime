use serde::{Deserialize, Serialize};
use worker::console_error;

use crate::{
    auth::membership::MembershipError, bootstrap::types::BootstrapError,
    internal::types::MonitorKind,
};

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

impl From<crate::d1c::queries::GetMonitorByIdRow> for Monitor {
    fn from(row: crate::d1c::queries::GetMonitorByIdRow) -> Self {
        Monitor {
            id: row.id.unwrap_or_default(),
            org_id: row.org_id,
            name: row.name,
            kind: row.kind,
            url: row.url,
            interval_s: row.interval_s,
            timeout_ms: row.timeout_ms,
            follow_redirects: row.follow_redirects,
            verify_tls: row.verify_tls,
            expect_status_low: row.expect_status_low,
            expect_status_high: row.expect_status_high,
            expect_substring: row.expect_substring,
            headers_json: row.headers_json,
            tags_json: row.tags_json,
            enabled: row.enabled,
            last_checked_at_ts: row.last_checked_at_ts,
            next_run_at_ts: row.next_run_at_ts,
            current_status: row.current_status,
            last_ok: row.last_ok,
            consecutive_failures: row.consecutive_failures,
            current_incident_id: row.current_incident_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<crate::d1c::queries::GetMonitorsByOrgIdRow> for Monitor {
    fn from(row: crate::d1c::queries::GetMonitorsByOrgIdRow) -> Self {
        Monitor {
            id: row.id.unwrap_or_default(),
            org_id: row.org_id,
            name: row.name,
            kind: row.kind,
            url: row.url,
            interval_s: row.interval_s,
            timeout_ms: row.timeout_ms,
            follow_redirects: row.follow_redirects,
            verify_tls: row.verify_tls,
            expect_status_low: row.expect_status_low,
            expect_status_high: row.expect_status_high,
            expect_substring: row.expect_substring,
            headers_json: row.headers_json,
            tags_json: row.tags_json,
            enabled: row.enabled,
            last_checked_at_ts: row.last_checked_at_ts,
            next_run_at_ts: row.next_run_at_ts,
            current_status: row.current_status,
            last_ok: row.last_ok,
            consecutive_failures: row.consecutive_failures,
            current_incident_id: row.current_incident_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug)]
pub enum MonitorError {
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
