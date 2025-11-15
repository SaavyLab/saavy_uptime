use worker::console_error;

#[derive(Debug)]
pub enum BootstrapError {
    DbInit(worker::Error),
    DbBind(worker::Error),
    DbRun(worker::Error),
    NotFound,
}

impl From<worker::Error> for BootstrapError {
    fn from(err: worker::Error) -> Self {
        BootstrapError::DbRun(err)
    }
}

impl From<BootstrapError> for axum::http::StatusCode {
    fn from(err: BootstrapError) -> axum::http::StatusCode {
        match err {
            BootstrapError::DbInit(err) => {
                console_error!("bootstrap.db.init: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            BootstrapError::DbBind(err) => {
                console_error!("bootstrap.db.bind: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            BootstrapError::DbRun(err) => {
                console_error!("bootstrap.db.run: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            BootstrapError::NotFound => {
                console_error!("bootstrap.not.found");
                axum::http::StatusCode::NOT_FOUND
            }
        }
    }
}
