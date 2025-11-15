use crate::auth::current_user::CurrentUser;
use crate::monitors::service::{create_monitor, get_monitor_by_id, get_monitors};
use crate::monitors::types::{CreateMonitor, Monitor};
use crate::router::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Result,
    Json,
};

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.get_by_id",
    skip(state, _current_user),
    fields(monitor_id = %id)
)]
pub async fn get_monitor_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _current_user: CurrentUser,
) -> Result<Json<Monitor>, StatusCode> {
    match get_monitor_by_id(&state, id).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.list",
    skip(state),
    fields(identity_id = %subject)
)]
pub async fn get_monitors_handler(
    State(state): State<AppState>,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Vec<Monitor>>, StatusCode> {
    match get_monitors(&state, &subject).await {
        Ok(monitors) => Ok(Json(monitors)),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.create",
    skip(state, monitor),
    fields(identity_id = %subject)
)]
pub async fn create_monitor_handler(
    State(state): State<AppState>,
    CurrentUser { subject, .. }: CurrentUser,
    Json(monitor): Json<CreateMonitor>,
) -> Result<Json<Monitor>, StatusCode> {
    match create_monitor(&state, &subject, monitor).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(err) => Err(err.into()),
    }
}
