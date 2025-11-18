use crate::auth::current_user::CurrentUser;
use crate::auth::membership::load_membership;
use crate::cloudflare::d1::AppDb;
use crate::cloudflare::durable_objects::ticker::AppTicker;
use crate::d1c::queries::{delete_monitor, get_monitor_by_id, get_monitors_by_org_id};
use crate::monitors::service::{
    create_monitor_for_org,
    update_monitor_for_org,
};
use crate::monitors::types::{CreateMonitor, Monitor, UpdateMonitor};
use axum::{extract::Path, http::StatusCode, response::Result, Json};

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.get_by_id",
    skip(d1),
    fields(monitor_id = %id)
)]
pub async fn get_monitor_by_id_handler(
    Path(id): Path<String>,
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Monitor>, StatusCode> {
    let org_id = load_membership(&d1, &subject).await?.organization_id;

    match get_monitor_by_id(&d1, &org_id, &id).await {
        Ok(Some(monitor)) => Ok(Json(monitor.into())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.list",
    skip(d1),
    fields(subject = %subject)
)]
pub async fn get_monitors_handler(
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Vec<Monitor>>, StatusCode> {
    let org_id = load_membership(&d1, &subject).await?.organization_id;
    match get_monitors_by_org_id(&d1, &org_id).await {
        Ok(monitors) => Ok(Json(monitors.into_iter().map(|monitor| monitor.into()).collect())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
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
    let org_id = load_membership(&d1, &subject).await?.organization_id;
    match create_monitor_for_org(&ticker, &d1, &org_id, monitor).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.update",
    skip(d1, monitor),
    fields(identity_id = %subject)
)]
pub async fn update_monitor_handler(
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
    Path(id): Path<String>,
    Json(monitor): Json<UpdateMonitor>,
) -> Result<StatusCode, StatusCode> {
    let org_id = load_membership(&d1, &subject).await?.organization_id;
    match update_monitor_for_org(&d1, &org_id, &id, monitor).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.delete",
    skip(d1),
    fields(identity_id = %subject)
)]
pub async fn delete_monitor_handler(
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = load_membership(&d1, &subject).await?.organization_id;
    match delete_monitor(&d1, &id, &org_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
