use crate::router::AppState;
use crate::utils::wasm_types::js_number;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Result,
    Json,
};
use cuid2::create_id;
use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, D1Database};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreateMonitor {
    pub org_id: String,
    pub name: String,
    pub url: String,
    pub interval: i64,
    pub timeout: i64,
    pub verify_tls: bool,
    pub follow_redirects: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub interval_s: i64,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
    pub expect_status_low: Option<i64>,
    pub expect_status_high: Option<i64>,
    pub expect_substring: Option<String>,
    pub headers_json: Option<String>,
    pub tags_json: Option<String>,
    pub enabled: i64,
    pub last_checked_at_ts: Option<i64>,
    pub next_run_at_ts: Option<i64>,
    pub current_status: String,
    pub last_ok: i64,
    pub consecutive_failures: i64,
    pub current_incident_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn get_d1(state: &AppState) -> Result<D1Database> {
    state.env.d1("DB")
}

pub async fn get_monitor_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Monitor>, StatusCode> {
    let d1 = get_d1(&state)?.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let statement = d1.prepare("SELECT * FROM monitors WHERE id = ?1");
    let query = statement
        .bind(&[id.into()])?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match query.first::<Monitor>(None).await {
        Ok(Some(monitor)) => Ok(Json(monitor)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_monitors_by_org_id(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<Monitor>>, StatusCode> {
    let d1 = get_d1(&state)?.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let statement = d1.prepare("SELECT * FROM monitors WHERE org_id = ?1 ORDER BY created_at DESC");
    let query = statement
        .bind(&[org_id.into()])?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match query.all().await {
        Ok(result) => Ok(Json(result.results::<Monitor>()?)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_monitor(
    State(state): State<AppState>,
    Json(monitor): Json<CreateMonitor>,
) -> Result<Json<Monitor>, StatusCode> {
    let d1 = get_d1(&state)?.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let statement = d1.prepare("INSERT INTO monitors (id, org_id, name, kind, url, interval_s, timeout_ms, follow_redirects, verify_tls, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)");
    let id = create_id().to_string();

    let bind_values = vec![
        JsValue::from_str(&id),
        JsValue::from_str(&monitor.org_id),
        JsValue::from_str(&monitor.name),
        JsValue::from_str("http"),
        JsValue::from_str(&monitor.url),
        js_number(monitor.interval),
        js_number(monitor.timeout),
        js_number(monitor.follow_redirects as i64),
        js_number(monitor.verify_tls as i64),
        js_number(1762845925),
        js_number(1762845925),
    ];

    let query = statement
        .bind(&bind_values)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    query
        .run()
        .await?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match query.first::<Monitor>(None).await {
        Ok(Some(monitor)) => Ok(Json(monitor)),
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
