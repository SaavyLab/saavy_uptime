use cuid2::create_id;
use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, Response, Result, RouteContext};

use crate::utils::wasm_types::js_number;

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

pub async fn create_monitor(ctx: &RouteContext<()>, monitor: CreateMonitor) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
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

    let query = statement.bind(&bind_values)?;
    query.run().await?;

    Response::from_json(&monitor)
}
