use axum::http::StatusCode;
use worker::console_error;

#[derive(Debug)]
pub enum RelayError {
    Validation {
        field: &'static str,
        message: String,
    },
    Conflict(&'static str),
    Database {
        context: &'static str,
        source: worker::Error,
    },
    DurableObject {
        context: &'static str,
        source: worker::Error,
    },
    Serialization {
        context: &'static str,
        source: serde_json::Error,
    },
}

impl RelayError {
    pub fn validation(field: &'static str, message: impl Into<String>) -> Self {
        RelayError::Validation {
            field,
            message: message.into(),
        }
    }

    pub fn conflict(field: &'static str) -> Self {
        RelayError::Conflict(field)
    }

    pub fn database(context: &'static str, source: worker::Error) -> Self {
        RelayError::Database { context, source }
    }

    pub fn durable_object(context: &'static str, source: worker::Error) -> Self {
        RelayError::DurableObject { context, source }
    }

    pub fn serialization(context: &'static str, source: serde_json::Error) -> Self {
        RelayError::Serialization { context, source }
    }
}

impl From<RelayError> for StatusCode {
    fn from(err: RelayError) -> Self {
        match &err {
            RelayError::Validation { field, message } => {
                console_error!("relay.validation: field={} message={}", field, message);
                StatusCode::BAD_REQUEST
            }
            RelayError::Conflict(field) => {
                console_error!("relay.conflict: field={}", field);
                StatusCode::CONFLICT
            }
            RelayError::Database { context, source } => {
                console_error!("relay.db: {}: {:?}", context, source);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            RelayError::DurableObject { context, source } => {
                console_error!("relay.do: {}: {:?}", context, source);
                StatusCode::BAD_GATEWAY
            }
            RelayError::Serialization { context, source } => {
                console_error!("relay.serialization: {}: {:?}", context, source);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
