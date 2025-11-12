use crate::auth::current_user::CurrentUser;
use crate::cloudflare::d1::get_d1;
use crate::router::AppState;
use crate::utils::date::now_ms;
use axum::{extract::State, http::StatusCode, response::Result, Json};
use cuid2::create_id;
use worker::{console_error, wasm_bindgen::JsValue};

use crate::utils::wasm_types::js_number;

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
pub async fn status(
    State(state): State<AppState>,
    CurrentUser {
        email,
        subject: _,
        claims: _,
    }: CurrentUser,
) -> Result<Json<BootstrapStatus>, StatusCode> {
    let team_name = state.access_config().team_name();

    let d1 = get_d1(&state).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let statement = d1.prepare("SELECT COUNT(*) as count FROM organizations");
    let query = statement
        .bind(&[])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match query.first::<CountResult>(None).await {
        Ok(Some(CountResult { count })) => Ok(Json(BootstrapStatus {
            is_bootstrapped: count > 0,
            suggested_slug: team_name,
            email,
        })),
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializePayload {
    name: String,
    slug: String,
}

#[worker::send]
pub async fn initialize(
    State(state): State<AppState>,
    CurrentUser {
        email,
        subject,
        claims: _,
    }: CurrentUser,
    Json(payload): Json<InitializePayload>,
) -> Result<Json<BootstrapStatus>, StatusCode> {
    let d1 = get_d1(&state).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let org_statement = d1.prepare(
        "INSERT INTO organizations (id, slug, name, owner_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    );
    let member_statement = d1.prepare(
        "INSERT INTO members (identity_id, email, is_workspace_admin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(identity_id) DO UPDATE SET email=excluded.email, updated_at=excluded.updated_at",
    );
    let organization_member_statement = d1.prepare(
        "INSERT INTO organization_members (organization_id, identity_id, role, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    );
    let org_id = create_id().to_string();
    let now = now_ms();

    let org_bind_values = vec![
        JsValue::from_str(&org_id),
        JsValue::from_str(&payload.slug),
        JsValue::from_str(&payload.name),
        JsValue::from_str(&subject),
        js_number(now),
    ];
    let member_bind_values = vec![
        JsValue::from_str(&subject),
        JsValue::from_str(&email),
        js_number(0),
        js_number(now),
        js_number(now),
    ];
    let organization_member_bind_values = vec![
        JsValue::from_str(&org_id),
        JsValue::from_str(&subject),
        JsValue::from_str("admin"),
        js_number(now),
        js_number(now),
    ];

    let statements = vec![
        member_statement.bind(&member_bind_values).map_err(|err| {
            console_error!("bootstrap.initialize: member bind failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
        org_statement.bind(&org_bind_values).map_err(|err| {
            console_error!("bootstrap.initialize: org bind failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
        organization_member_statement
            .bind(&organization_member_bind_values)
            .map_err(|err| {
                console_error!("bootstrap.initialize: org-member bind failed: {err:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
    ];

    let batch_results = d1.batch(statements).await.map_err(|err| {
        console_error!("bootstrap.initialize: batch execution failed: {err:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(err) = batch_results.iter().find_map(|result| result.error()) {
        console_error!("bootstrap.initialize: statement failed: {err}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(BootstrapStatus {
        is_bootstrapped: true,
        suggested_slug: payload.slug,
        email,
    }))
}
