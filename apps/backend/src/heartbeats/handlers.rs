use crate::auth::membership::load_membership;
use crate::cloudflare::d1::AppDb;
use crate::heartbeats::types::{GetHeartbeatsParams, Heartbeat};
use crate::utils::date::now_ms;
use crate::d1c::queries::heartbeats::get_heartbeats_by_monitor_id;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Result,
    Json,
};
use hb_auth::User;
use worker::console_error;

#[worker::send]
#[tracing::instrument(
    name = "heartbeats.http.get_by_monitor_id",
    skip(d1),
    fields(subject = %auth.sub())
)]
pub async fn get_heartbeats_by_monitor_id_handler(
    AppDb(d1): AppDb,
    Path(monitor_id): Path<String>,
    Query(params): Query<GetHeartbeatsParams>,
    auth: User,
) -> Result<Json<Vec<Heartbeat>>, StatusCode> {
    let org_id = load_membership(&d1, auth.sub()).await?.organization_id;
    let before = params.before.unwrap_or(now_ms());
    let limit = params.limit.unwrap_or(50);

    match get_heartbeats_by_monitor_id(&d1, &org_id, &monitor_id, before, limit).await {
        Ok(heartbeats) => Ok(Json(
            heartbeats
                .into_iter()
                .map(|heartbeat| heartbeat.into())
                .collect(),
        )),
        Err(err) => {
            console_error!("heartbeats.get_by_monitor_id: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
