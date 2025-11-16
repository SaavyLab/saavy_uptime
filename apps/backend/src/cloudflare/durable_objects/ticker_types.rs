use serde::{Deserialize, Serialize};

use crate::internal::types::MonitorKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerConfig {
    pub org_id: String,
    pub tick_interval_ms: u64,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TickerState {
    pub config: Option<TickerConfig>,
    pub last_tick_ts: i64,
    pub consecutive_errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRow {
    pub id: String,
    pub interval_s: i64,
    pub url: String,
    pub kind: MonitorKind,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDispatch {
    pub id: String,
    pub url: String,
    pub kind: MonitorKind,
    pub scheduled_for_ts: i64,
    pub timeout_ms: i64,
    pub follow_redirects: bool,
    pub verify_tls: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchPayload {
    pub dispatch_id: String,
    pub monitor_id: String,
    pub monitor_url: String,
    pub kind: MonitorKind,
    pub scheduled_for_ts: i64,
    pub timeout_ms: i64,
    pub follow_redirects: bool,
    pub verify_tls: bool,
}

impl From<(MonitorRow, i64)> for MonitorDispatch {
    fn from((row, scheduled_for_ts): (MonitorRow, i64)) -> Self {
        Self {
            id: row.id,
            url: row.url,
            kind: row.kind,
            scheduled_for_ts,
            timeout_ms: row.timeout_ms,
            follow_redirects: row.follow_redirects != 0,
            verify_tls: row.verify_tls != 0,
        }
    }
}

#[derive(Debug)]
pub enum TickerError {
    Database {
        context: &'static str,
        source: worker::Error,
    },
    Alarm(worker::Error),
    Unknown(String),
    MissingVar(String),
    Request {
        context: &'static str,
        source: worker::Error,
    },
    Response {
        context: &'static str,
        source: worker::Error,
    },
    ResponseStatus {
        context: &'static str,
        status: u16,
    },
}

impl TickerError {
    pub fn database(context: &'static str, source: worker::Error) -> Self {
        TickerError::Database { context, source }
    }
    pub fn save_state(_context: &'static str, source: worker::Error) -> Self {
        TickerError::Alarm(source)
    }
    // this feels odd
    pub fn arm_alarm(_context: &'static str, source: worker::Error) -> Self {
        TickerError::Alarm(source)
    }
    pub fn unknown(context: &'static str, source: String) -> Self {
        TickerError::Unknown(format!("{context}: {source}"))
    }
    pub fn missing_var(context: &'static str, var: &str) -> Self {
        TickerError::MissingVar(format!("{context}: missing {var}"))
    }
    pub fn request(context: &'static str, source: worker::Error) -> Self {
        TickerError::Request { context, source }
    }
    pub fn response(context: &'static str, source: worker::Error) -> Self {
        TickerError::Response { context, source }
    }
    pub fn response_status(context: &'static str, status: u16) -> Self {
        TickerError::ResponseStatus { context, status }
    }
}

impl From<worker::Error> for TickerError {
    fn from(err: worker::Error) -> Self {
        TickerError::Unknown(err.to_string())
    }
}

impl From<TickerError> for worker::Error {
    fn from(err: TickerError) -> worker::Error {
        match err {
            TickerError::Unknown(message) => worker::Error::RustError(message),
            TickerError::MissingVar(message) => worker::Error::RustError(message),
            TickerError::Request { context, source } => {
                worker::Error::RustError(format!("{context}: {source:?}"))
            }
            TickerError::Response { context, source } => {
                worker::Error::RustError(format!("{context}: {source:?}"))
            }
            TickerError::ResponseStatus { context, status } => {
                worker::Error::RustError(format!("{context}: {status}"))
            }
            TickerError::Database { context, source } => {
                worker::Error::RustError(format!("{context}: {source:?}"))
            }
            TickerError::Alarm(source) => worker::Error::RustError(format!("alarm: {source:?}")),
        }
    }
}
