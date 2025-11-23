use crate::analytics::extractor::AppAeClient;
use crate::analytics::monitor_health::{recent_heartbeats, HeartbeatSample, TimeWindow};
use crate::auth::membership::load_membership;
use crate::auth::Role;
use crate::cloudflare::d1::AppDb;
use crate::cloudflare::durable_objects::ticker::AppTicker;
use crate::d1c::queries::monitors::{delete_monitor, get_monitor_by_id, get_monitors_by_org_id};
use crate::monitors::service::{create_monitor_for_org, update_monitor_for_org};
use crate::monitors::types::{CreateMonitor, Monitor, MonitorError, UpdateMonitor};
use crate::utils::date::now_ms;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Result,
    Json,
};
use hb_auth::User;
use serde::{Deserialize, Serialize};
use worker::console_error;

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.get_by_id",
    skip(d1),
    fields(monitor_id = %id)
)]
pub async fn get_monitor_by_id_handler(
    Path(id): Path<String>,
    AppDb(d1): AppDb,
    auth: User,
) -> Result<Json<Monitor>, StatusCode> {
    let org_id = load_membership(&d1, auth.sub()).await?.organization_id;

    match get_monitor_by_id(&d1, &id, &org_id).await {
        Ok(Some(row)) => Monitor::try_from(row).map(Json).map_err(StatusCode::from),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.list",
    skip(d1),
    fields(subject = %auth.sub())
)]
pub async fn get_monitors_handler(
    AppDb(d1): AppDb,
    auth: User,
) -> Result<Json<Vec<Monitor>>, StatusCode> {
    let org_id = load_membership(&d1, auth.sub()).await?.organization_id;
    match get_monitors_by_org_id(&d1, &org_id).await {
        Ok(rows) => rows
            .into_iter()
            .map(Monitor::try_from)
            .collect::<Result<Vec<_>, MonitorError>>()
            .map(Json)
            .map_err(StatusCode::from),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.create",
    skip(ticker, d1, monitor),
    fields(identity_id = %auth.sub())
)]
pub async fn create_monitor_handler(
    AppTicker(ticker): AppTicker,
    AppDb(d1): AppDb,
    auth: User,
    Json(monitor): Json<CreateMonitor>,
) -> Result<Json<Monitor>, StatusCode> {
    let org_id = load_membership(&d1, auth.sub()).await?.organization_id;
    match create_monitor_for_org(&ticker, &d1, &org_id, monitor).await {
        Ok(monitor) => Ok(Json(monitor)),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.update",
    skip(d1, monitor),
    fields(identity_id = %auth.sub())
)]
pub async fn update_monitor_handler(
    AppDb(d1): AppDb,
    auth: User,
    Path(id): Path<String>,
    Json(monitor): Json<UpdateMonitor>,
) -> Result<StatusCode, StatusCode> {
    let org_id = load_membership(&d1, auth.sub()).await?.organization_id;
    match update_monitor_for_org(&d1, &org_id, &id, monitor).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.delete",
    skip(d1),
    fields(identity_id = %auth.sub())
)]
pub async fn delete_monitor_handler(
    AppDb(d1): AppDb,
    auth: User<Role>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    if !auth.has_role(Role::Admin) {
        return Err(StatusCode::FORBIDDEN);
    }
    let org_id = load_membership(&d1, auth.sub()).await?.organization_id;
    match delete_monitor(&d1, &id, &org_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            console_error!("monitors.delete: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorHeartbeatsQuery {
    pub limit: Option<usize>,
    pub window_hours: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorHeartbeatsResponse {
    pub monitor_id: String,
    pub window: WindowSummary,
    pub items: Vec<HeartbeatSample>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowSummary {
    pub since_ms: i64,
    pub until_ms: i64,
    pub hours: i64,
}

#[worker::send]
#[tracing::instrument(
    name = "monitors.http.heartbeats",
    skip(d1, ae_client),
    fields(monitor_id = %id)
)]
pub async fn get_monitor_heartbeats_handler(
    Path(id): Path<String>,
    AppDb(d1): AppDb,
    AppAeClient(ae_client): AppAeClient,
    Query(params): Query<MonitorHeartbeatsQuery>,
    auth: User,
) -> Result<Json<MonitorHeartbeatsResponse>, StatusCode> {
    let membership = load_membership(&d1, auth.sub()).await?;
    let org_id = membership.organization_id;

    match get_monitor_by_id(&d1, &id, &org_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    let limit = params.limit.unwrap_or(100).clamp(1, 500);
    let hours = params.window_hours.unwrap_or(24).clamp(1, 720);
    let window = TimeWindow::last_hours(hours, now_ms());

    let items = recent_heartbeats(&ae_client, &id, &org_id, &window, limit)
        .await
        .map_err(|err| {
            console_error!("monitors.heartbeats.ae: {err:?}");
            StatusCode::BAD_GATEWAY
        })?;

    let response = MonitorHeartbeatsResponse {
        monitor_id: id,
        window: WindowSummary {
            since_ms: window.since_ms,
            until_ms: window.until_ms,
            hours,
        },
        items,
    };

    Ok(Json(response))
}
