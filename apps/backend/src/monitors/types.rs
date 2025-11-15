use serde::{Deserialize, Serialize};
use worker::console_error;

use crate::{auth::membership::MembershipError, bootstrap::types::BootstrapError};

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
        }
    }
}
