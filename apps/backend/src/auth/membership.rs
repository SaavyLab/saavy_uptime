use axum::http::StatusCode;
use serde::Deserialize;
use worker::console_error;

use crate::{cloudflare::d1::get_d1, router::AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct Membership {
    pub organization_id: String,
    pub role: String,
}

fn internal_error(ctx: &str, err: impl std::fmt::Debug) -> StatusCode {
    console_error!("{ctx}: {err:?}");
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn load_membership(
    state: &AppState,
    identity_id: &str,
) -> Result<Membership, StatusCode> {
    let d1 = get_d1(state).map_err(|err| internal_error("membership.d1", err))?;
    let statement = d1.prepare(
        "SELECT organization_id, role
         FROM organization_members
         WHERE identity_id = ?1
         ORDER BY created_at DESC
         LIMIT 1",
    );
    let query = statement
        .bind(&[identity_id.into()])
        .map_err(|err| internal_error("membership.bind", err))?;

    match query.first::<Membership>(None).await {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err(StatusCode::FORBIDDEN),
        Err(err) => Err(internal_error("membership.query", err)),
    }
}
