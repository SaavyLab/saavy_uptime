use axum::{routing::post, Router};

mod dispatch;
mod handlers;
pub mod types;

use crate::{
    internal::handlers::{dispatch_handler, reconcile_tickers_handler},
    router::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ticker/reconcile", post(reconcile_tickers_handler))
        .route("/dispatch/run", post(dispatch_handler))
}
