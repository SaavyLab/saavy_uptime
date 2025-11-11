use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, RouteContext, Result, Response};

use crate::utils::wasm_types::{js_number, js_optional_number, js_optional_string};

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

pub async fn get_monitor_by_id(ctx: &RouteContext<()>, id: &str) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let statement = d1.prepare("SELECT * FROM monitors WHERE id = ?1");
    let query = statement.bind(&[id.into()])?;
    let result = query.first::<Monitor>(None).await?;
    match result {
        Some(monitor) => Response::from_json(&monitor),
        None => Response::error("Monitor not found", 404),
    }
}

pub async fn get_monitors_by_org_id(ctx: &RouteContext<()>, org_id: &str) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let statement = d1.prepare("SELECT * FROM monitors WHERE org_id = ?1 ORDER BY created_at DESC");
    let query = statement.bind(&[org_id.into()])?;
    let result = query.all().await?;
    let monitors = result.results::<Monitor>()?;

    Response::from_json(&monitors)
}

pub async fn create_monitor(ctx: &RouteContext<()>, monitor: Monitor) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let statement = d1.prepare("INSERT INTO monitors (id, org_id, name, kind, url, interval_s, timeout_ms, follow_redirects, verify_tls, expect_status_low, expect_status_high, expect_substring, headers_json, tags_json, enabled, last_checked_at_ts, next_run_at_ts, current_status, last_ok, consecutive_failures, current_incident_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)");

    let bind_values = vec![
        JsValue::from_str(&monitor.id),
        JsValue::from_str(&monitor.org_id),
        JsValue::from_str(&monitor.name),
        JsValue::from_str(&monitor.kind),
        JsValue::from_str(&monitor.url),
        js_number(monitor.interval_s),
        js_number(monitor.timeout_ms),
        js_number(monitor.follow_redirects),
        js_number(monitor.verify_tls),
        js_optional_number(monitor.expect_status_low),
        js_optional_number(monitor.expect_status_high),
        js_optional_string(monitor.expect_substring.as_ref()),
        js_optional_string(monitor.headers_json.as_ref()),
        js_optional_string(monitor.tags_json.as_ref()),
        js_number(monitor.enabled),
        js_optional_number(monitor.last_checked_at_ts),
        js_optional_number(monitor.next_run_at_ts),
        JsValue::from_str(&monitor.current_status),
        js_number(monitor.last_ok),
        js_number(monitor.consecutive_failures),
        js_optional_string(monitor.current_incident_id.as_ref()),
        js_number(monitor.created_at),
        js_number(monitor.updated_at),
    ];

    let query = statement.bind(&bind_values)?;
    query.run().await?;

    Response::from_json(&monitor)
}