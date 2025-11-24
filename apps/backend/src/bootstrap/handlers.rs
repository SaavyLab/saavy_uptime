use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::cloudflare::d1::AppDb;
use crate::cloudflare::durable_objects::ticker::AppTicker;
use crate::d1c::queries::bootstrap::{create_member_stmt, create_organization_member_stmt};
use crate::d1c::queries::organizations::{check_if_bootstrapped, create_organization_stmt};
use crate::router::AppState;
use crate::utils::date::now_ms;
use axum::{extract::State, http::StatusCode, response::Result, Json};
use cuid2::create_id;
use hb_auth::{HasAuthConfig, User};
use worker::console_error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct BootstrapStatus {
    pub is_bootstrapped: bool,
    pub suggested_slug: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountResult {
    count: i64,
}

#[worker::send]
#[tracing::instrument(
    name = "bootstrap.status",
    skip(state, d1, auth),
    fields(identity_id = %auth.sub())
)]
pub async fn status(
    State(state): State<AppState>,
    AppDb(d1): AppDb,
    auth: User,
) -> Result<Json<BootstrapStatus>, StatusCode> {
    let team_name = state.auth_config().team_name();

    match check_if_bootstrapped(&d1).await {
        Ok(count) => Ok(Json(BootstrapStatus {
            is_bootstrapped: count.unwrap_or(0) > 0,
            suggested_slug: team_name,
            email: auth.email().to_string(),
        })),
        Err(err) => {
            console_error!("bootstrap.status.query: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializePayload {
    name: String,
    slug: String,
}

#[worker::send]
#[tracing::instrument(
    name = "bootstrap.initialize",
    skip(ticker, d1, auth, payload),
    fields(identity_id = %auth.sub())
)]
pub async fn initialize(
    AppTicker(ticker): AppTicker,
    AppDb(d1): AppDb,
    auth: User,
    Json(payload): Json<InitializePayload>,
) -> Result<Json<BootstrapStatus>, StatusCode> {
    let org_id = create_id().to_string();
    let now = now_ms();

    let org_statement =
        create_organization_stmt(&d1, &org_id, &payload.slug, &payload.name, auth.sub(), now)
            .map_err(|err| {
                console_error!(
                    "bootstrap.initialize: create organization statement failed: {err:?}"
                );
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    let member_statement =
        create_member_stmt(&d1, auth.sub(), auth.email(), 0, now, now).map_err(|err| {
            console_error!("bootstrap.initialize: create member statement failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let organization_member_statement =
        create_organization_member_stmt(&d1, &org_id, auth.sub(), "admin", now, now).map_err(
            |err| {
                console_error!(
                    "bootstrap.initialize: create organization member statement failed: {err:?}"
                );
                StatusCode::INTERNAL_SERVER_ERROR
            },
        )?;

    let batch_results = d1
        .batch(vec![
            member_statement,
            org_statement,
            organization_member_statement,
        ])
        .await
        .map_err(|err| {
            console_error!("bootstrap.initialize: batch execution failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(err) = batch_results.iter().find_map(|result| result.error()) {
        console_error!("bootstrap.initialize: statement failed: {err}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Err(err) = ensure_ticker_bootstrapped(&ticker, &org_id).await {
        console_error!("bootstrap.initialize: ticker bootstrap failed: {err:?}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(BootstrapStatus {
        is_bootstrapped: true,
        suggested_slug: payload.slug,
        email: auth.email().to_string(),
    }))
}
