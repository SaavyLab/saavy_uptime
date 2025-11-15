use crate::auth::current_user::CurrentUser;
use crate::cloudflare::d1::AppDb;
use crate::cloudflare::ticker::AppTicker;
use crate::monitors::service::{create_monitor, get_monitor_by_id, get_monitors};
use crate::monitors::types::{CreateMonitor, Monitor};
use axum::{extract::Path, http::StatusCode, response::Result, Json};

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.get_by_id",
    skip(d1, _current_user),
    fields(monitor_id = %id)
)]
pub async fn get_monitor_by_id_handler(
    Path(id): Path<String>,
    AppDb(d1): AppDb,
    _current_user: CurrentUser,
) -> Result<Json<Monitor>, StatusCode> {
    match get_monitor_by_id(&d1, id).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.list",
    skip(d1),
    fields(identity_id = %subject)
)]
pub async fn get_monitors_handler(
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Vec<Monitor>>, StatusCode> {
    match get_monitors(&d1, &subject).await {
        Ok(monitors) => Ok(Json(monitors)),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.create",
    skip(ticker, d1, monitor),
    fields(identity_id = %subject)
)]
pub async fn create_monitor_handler(
    AppTicker(ticker): AppTicker,
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
    Json(monitor): Json<CreateMonitor>,
) -> Result<Json<Monitor>, StatusCode> {
    match create_monitor(&ticker, &d1, &subject, monitor).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(err) => Err(err.into()),
    }
}
