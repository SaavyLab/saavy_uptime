use crate::router::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Result,
    Json,
};
use cuid2::create_id;
use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, D1Database};

use crate::utils::wasm_types::js_number;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreateOrganization {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub created_at: i64,
}

fn get_d1(state: &AppState) -> Result<D1Database> {
    state.env.d1("DB")
}

pub async fn get_organization_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Organization>, StatusCode> {
    let d1 = get_d1(&state)?.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let statement = d1.prepare("SELECT * FROM organizations WHERE id = ?1");
    let query = statement
        .bind(&[id.into()])?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match query.first::<Organization>(None).await {
        Ok(Some(organization)) => Ok(Json(organization)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganization>,
) -> Result<Json<Organization>, StatusCode> {
    let d1 = get_d1(&state)?.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let statement = d1
        .prepare("INSERT INTO organizations (id, slug, name, created_at) VALUES (?1, ?2, ?3, ?4)");
    let id = create_id().to_string();
    let created_at = 1_762_845_925;

    let bind_values = vec![
        JsValue::from_str(&id),
        JsValue::from_str(&payload.slug),
        JsValue::from_str(&payload.name),
        js_number(created_at),
    ];

    let query = statement
        .bind(&bind_values)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    query
        .run()
        .await?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match query.first::<Organization>(None).await {
        Ok(Some(organization)) => Ok(Json(organization)),
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
