use cuid2::create_id;
use std::result::Result;
use worker::wasm_bindgen::JsValue;
use worker::{console_log, D1Database, ObjectNamespace};

use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::d1c::queries::monitors::{create_monitor, get_monitor_by_id};
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
    let id = create_id().to_string();
    create_monitor(&d1, &id, &org_id, &monitor.name.as_str(), "http", &monitor.url.as_str(), monitor.interval, monitor.timeout, monitor.follow_redirects as i64, monitor.verify_tls as i64, now_ms(), now_ms())
        .await
        .map_err(MonitorError::DbRun)?;

    ensure_ticker_bootstrapped(ticker, org_id)
        .await
        .map_err(MonitorError::Bootstrap)?;

    match get_monitor_by_id(&d1, &id, &org_id).await {
        Ok(Some(monitor)) => Ok(monitor.into()),
        Ok(None) => Err(MonitorError::NotFound),
        Err(_) => Err(MonitorError::DbRun(worker::Error::RustError("Failed to get monitor".to_string()))),
    }
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
