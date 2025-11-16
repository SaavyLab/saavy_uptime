use cuid2::create_id;
use std::result::Result;
use worker::wasm_bindgen::JsValue;
use worker::{console_log, D1Database, ObjectNamespace};

use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::monitors::types::{CreateMonitor, Monitor, MonitorError, UpdateMonitor};
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

#[tracing::instrument(
    name = "monitors.create_for_org",
    skip(d1, monitor),
    fields(org_id = %org_id)
)]
pub async fn create_monitor_for_org(
    ticker: &ObjectNamespace,
    d1: &D1Database,
    org_id: &str,
    monitor: CreateMonitor,
) -> Result<Monitor, MonitorError> {
    let statement = d1.prepare("INSERT INTO monitors (id, org_id, name, kind, url, interval_s, timeout_ms, follow_redirects, verify_tls, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)");
    let id = create_id().to_string();
    let now = now_ms();

    let bind_values = vec![
        JsValue::from_str(&id),
        JsValue::from_str(org_id),
        JsValue::from_str(&monitor.name),
        JsValue::from_str("http"),
        JsValue::from_str(&monitor.url),
        js_number(monitor.interval),
        js_number(monitor.timeout),
        js_number(monitor.follow_redirects as i64),
        js_number(monitor.verify_tls as i64),
        js_number(now),
        js_number(now),
    ];

    let query = statement.bind(&bind_values).map_err(MonitorError::DbBind)?;

    query.run().await.map_err(MonitorError::DbRun)?;

    ensure_ticker_bootstrapped(ticker, org_id)
        .await
        .map_err(MonitorError::Bootstrap)?;

    get_monitor_by_id(&d1, org_id, id).await
}

#[tracing::instrument(name = "monitors.get_by_id", skip(d1), fields(monitor_id = %id))]
pub async fn get_monitor_by_id(
    d1: &D1Database,
    org_id: &str,
    id: String,
) -> Result<Monitor, MonitorError> {
    let statement = d1.prepare("SELECT * FROM monitors WHERE id = ?1 AND org_id = ?2");
    let query = statement
        .bind(&[id.into(), org_id.into()])
        .map_err(|err| MonitorError::DbBind(err))?;

    match query.first::<Monitor>(None).await {
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(MonitorError::NotFound),
        Err(err) => Err(MonitorError::DbRun(err)),
    }
}

#[tracing::instrument(
    name = "monitors.list_for_org",
    skip(d1),
    fields(org_id = %org_id)
)]
pub async fn get_monitors(d1: &D1Database, org_id: &str) -> Result<Vec<Monitor>, MonitorError> {
    let statement = d1.prepare("SELECT * FROM monitors WHERE org_id = ?1 ORDER BY created_at DESC");
    let query = statement
        .bind(&[org_id.into()])
        .map_err(|err| MonitorError::DbBind(err))?;

    let all = query.all().await.map_err(|err| MonitorError::DbRun(err))?;

    all.results::<Monitor>()
        .map_err(|err| MonitorError::DbRun(err))
}

#[tracing::instrument(
    name = "monitors.update_for_org",
    skip(d1),
    fields(org_id = %org_id, monitor_id = %monitor_id)
)]
pub async fn update_monitor_for_org(
    d1: &D1Database,
    org_id: &str,
    monitor_id: &str,
    monitor: UpdateMonitor,
) -> Result<(), MonitorError> {
    let mut fields = Vec::new();
    let mut values: Vec<JsValue> = Vec::new();

    console_log!("update monitor: {monitor:?}");

    if let Some(name) = monitor.name {
        fields.push("name = ?".to_string());
        values.push(JsValue::from_str(&name));
    }

    if let Some(kind) = monitor.kind {
        fields.push("kind = ?".to_string());
        values.push(JsValue::from_str(&kind.to_string()));
    }

    if let Some(url) = monitor.url {
        fields.push("url = ?".to_string());
        values.push(JsValue::from_str(&url));
    }

    if let Some(interval) = monitor.interval {
        fields.push("interval_s = ?".to_string());
        values.push(js_number(interval));
    }

    if let Some(timeout) = monitor.timeout {
        fields.push("timeout_ms = ?".to_string());
        values.push(js_number(timeout));
    }

    if let Some(follow_redirects) = monitor.follow_redirects {
        fields.push("follow_redirects = ?".to_string());
        values.push(js_number(follow_redirects as i64));
    }

    if let Some(verify_tls) = monitor.verify_tls {
        fields.push("verify_tls = ?".to_string());
        values.push(js_number(verify_tls as i64));
    }

    if let Some(expect_status_low) = monitor.expect_status_low {
        fields.push("expect_status_low = ?".to_string());
        values.push(js_number(expect_status_low));
    }

    if let Some(expect_status_high) = monitor.expect_status_high {
        fields.push("expect_status_high = ?".to_string());
        values.push(js_number(expect_status_high));
    }

    if let Some(expect_substring) = monitor.expect_substring {
        fields.push("expect_substring = ?".to_string());
        values.push(JsValue::from_str(&expect_substring));
    }

    if let Some(headers_json) = monitor.headers_json {
        fields.push("headers_json = ?".to_string());
        values.push(JsValue::from_str(&headers_json));
    }

    if let Some(tags_json) = monitor.tags_json {
        fields.push("tags_json = ?".to_string());
        values.push(JsValue::from_str(&tags_json));
    }

    if let Some(enabled) = monitor.enabled {
        fields.push("enabled = ?".to_string());
        values.push(js_number(enabled as i64));
    }

    if fields.is_empty() {
        return Err(MonitorError::NoFieldsToUpdate);
    }

    fields.push("updated_at = ?".to_string());
    values.push(js_number(now_ms()));

    values.push(JsValue::from_str(monitor_id));
    values.push(JsValue::from_str(org_id));

    let sql = format!(
        "UPDATE monitors SET {} WHERE id = ? AND org_id = ?",
        fields.join(", ")
    );

    let query = d1
        .prepare(&sql)
        .bind(&values)
        .map_err(MonitorError::DbBind)?;

    query.run().await.map_err(MonitorError::DbRun)?;

    console_log!("updated monitor: {monitor_id}");

    Ok(())
}

#[tracing::instrument(
    name = "monitors.delete_for_org",
    skip(d1),
    fields(org_id = %org_id, monitor_id = %monitor_id)
)]
pub async fn delete_monitor_for_org(
    d1: &D1Database,
    org_id: &str,
    monitor_id: &str,
) -> Result<(), MonitorError> {
    let statement = d1.prepare("DELETE FROM monitors WHERE id = ?1");
    let query = statement
        .bind(&[monitor_id.into()])
        .map_err(MonitorError::DbBind)?;
    query.run().await.map_err(MonitorError::DbRun)?;
    Ok(())
}
