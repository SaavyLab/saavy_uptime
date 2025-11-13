use axum::http::StatusCode;
use worker::console_error;

pub fn internal_error(context: &str, err: impl std::fmt::Debug) -> StatusCode {
    console_error!("{context}: {err:?}");
    StatusCode::INTERNAL_SERVER_ERROR
}

pub fn rust_error(context: &str, err: impl std::fmt::Debug) -> worker::Error {
    console_error!("{context}: {err:?}");
    worker::Error::RustError(format!("{context}: {err:?}"))
}