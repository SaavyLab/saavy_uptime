use serde::{Deserialize, Serialize};
use worker::console_error;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GetHeartbeatsParams {
    pub limit: Option<i64>,
    pub before: Option<i64>, // timestamp in milliseconds
}

#[derive(Debug)]
pub enum HeartbeatError {
    DbInit(worker::Error),
    DbBind(worker::Error),
    DbRun(worker::Error),
}

impl From<worker::Error> for HeartbeatError {
    fn from(err: worker::Error) -> Self {
        HeartbeatError::DbRun(err)
    }
}

impl From<HeartbeatError> for axum::http::StatusCode {
    fn from(err: HeartbeatError) -> Self {
        match err {
            HeartbeatError::DbInit(err) => {
                console_error!("heartbeats.db.init: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            HeartbeatError::DbBind(err) => {
                console_error!("heartbeats.db.bind: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            HeartbeatError::DbRun(err) => {
                console_error!("heartbeats.db.run: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Heartbeat {
    pub monitor_id: String,
    pub org_id: String,
    pub ts: i64,
    pub ok: i64,
    pub code: Option<u16>,
    pub rtt_ms: Option<i64>,
    pub err: Option<String>,
    pub region: Option<String>,
    pub dispatch_id: String,
}

impl From<crate::d1c::queries::heartbeats::GetHeartbeatsByMonitorIdRow> for Heartbeat {
    fn from(row: crate::d1c::queries::heartbeats::GetHeartbeatsByMonitorIdRow) -> Self {
        Heartbeat {
            monitor_id: row.monitor_id,
            org_id: row.org_id,
            ts: row.ts,
            ok: row.ok,
            code: row.code.map(|r| r as u16),
            rtt_ms: row.rtt_ms,
            err: row.err,
            region: row.region,
            dispatch_id: row.dispatch_id.unwrap_or_default(),
        }
    }
}