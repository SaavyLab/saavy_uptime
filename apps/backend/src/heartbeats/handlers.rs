use crate::{auth::current_user::CurrentUser, heartbeats::service::get_heartbeats_by_monitor_id};
use crate::auth::membership::load_membership;
use crate::cloudflare::d1::AppDb;
use crate::heartbeats::types::{GetHeartbeatsParams, Heartbeat};
use axum::{extract::{Path, Query}, http::StatusCode, response::Result, Json};

#[worker::send]
#[tracing::instrument(
    name = "heartbeats.http.get_by_monitor_id",
    skip(d1),
    fields(subject = %subject)
)]
pub async fn get_heartbeats_by_monitor_id_handler(
    AppDb(d1): AppDb,
    Path(monitor_id): Path<String>,
    Query(params): Query<GetHeartbeatsParams>,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Vec<Heartbeat>>, StatusCode> {
    let org_id = load_membership(&d1, &subject).await?.organization_id;
    match get_heartbeats_by_monitor_id(&d1, &org_id, &monitor_id, params).await {
        Ok(heartbeats) => Ok(Json(heartbeats)),
        Err(err) => Err(err.into()),
    }
  }