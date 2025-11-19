use serde::{Deserialize, Serialize};

use crate::{internal::types::MonitorKind, monitors::types::HttpMonitorConfig};

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
    pub kind: MonitorKind,
    pub config_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDispatchRow {
    pub id: String,
    pub kind: MonitorKind,
    pub config: HttpMonitorConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchPayload {
    pub dispatch_id: String,
    pub monitor_id: String,
    pub org_id: String,
    pub kind: MonitorKind,
    pub config: HttpMonitorConfig,
}

// impl From<(MonitorRow, i64)> for MonitorDispatch {
//     fn from((row, scheduled_for_ts): (MonitorRow, i64)) -> Self {
//         Self {
//             id: row.id,
//             kind: row.kind,
//             config: serde_json::from_str(&row.config_json).unwrap(),
//         }
//     }
// }

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
    UnsupportedMonitorKind(MonitorKind),
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
    pub fn unsupported_monitor_kind(context: &'static str, kind: MonitorKind) -> Self {
        TickerError::UnsupportedMonitorKind(kind)
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
            TickerError::UnsupportedMonitorKind(kind) => worker::Error::RustError(format!("unsupported monitor kind: {kind}")),
        }
    }
}
