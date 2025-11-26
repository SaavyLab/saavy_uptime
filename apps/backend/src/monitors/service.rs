use cuid2::create_id;
use std::{convert::TryFrom, result::Result};
use worker::wasm_bindgen::JsValue;
use worker::{console_log, D1Database};

use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::cloudflare::durable_objects::ticker::AppTicker;
use crate::d1c::queries::monitors::{
    create_monitor, get_monitor_by_id, set_monitor_relay, update_monitor_status,
};
use crate::monitors::types::{
    CreateMonitor, HeartbeatResult, Monitor, MonitorError, MonitorStatus, MonitorStatusSnapshot,
    UpdateMonitor,
};
use crate::relays::errors::RelayError;
use crate::relays::service::get_relay_by_id;
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

#[tracing::instrument(
    name = "monitors.create_for_org",
    skip(d1, monitor),
    fields(org_id = %org_id)
)]
pub async fn create_monitor_for_org(
    ticker: &AppTicker,
    d1: &D1Database,
    org_id: &str,
    monitor: CreateMonitor,
) -> Result<Monitor, MonitorError> {
    let id = create_id().to_string();
    let relay_id = monitor.relay_id.trim().to_string();
    if relay_id.is_empty() {
        return Err(MonitorError::InvalidConfig(
            "Relay selection is required".to_string(),
        ));
    }

    get_relay_by_id(d1, &relay_id)
        .await
        .map_err(relay_error_to_monitor_error)?;

    monitor.config.validate()?;
    let config_json = monitor.config.to_json()?;
    let kind = monitor.kind.to_string();
    let now = now_ms();
    create_monitor(
        d1,
        &id,
        &org_id,
        &monitor.name.as_str(),
        &kind,
        1,
        &config_json,
        MonitorStatus::Pending.to_string().as_str(),
        now,
        now,
    )
    .await
    .map_err(MonitorError::DbRun)?;

    ensure_ticker_bootstrapped(ticker, org_id)
        .await
        .map_err(MonitorError::Bootstrap)?;

    set_monitor_relay(d1, &relay_id, now_ms(), &id, &org_id)
        .await
        .map_err(MonitorError::DbRun)?;

    match get_monitor_by_id(d1, &id, &org_id).await {
        Ok(Some(row)) => {
            let mut monitor = Monitor::try_from(row)?;
            monitor.relay_id = Some(relay_id);
            Ok(monitor)
        }
        Ok(None) => Err(MonitorError::NotFound),
        Err(_) => Err(MonitorError::DbRun(worker::Error::RustError(
            "Failed to get monitor".to_string(),
        ))),
    }
}

fn relay_error_to_monitor_error(err: RelayError) -> MonitorError {
    match err {
        RelayError::Validation { field: _, message } => MonitorError::InvalidConfig(message),
        RelayError::Database { source, .. } => MonitorError::DbRun(source),
        RelayError::DurableObject { source, .. } => MonitorError::DbRun(source),
        RelayError::Serialization { context, source } => MonitorError::InvalidConfig(format!(
            "Unable to parse relay response ({context}): {source}"
        )),
        RelayError::Conflict(_) => {
            MonitorError::InvalidConfig("Relay configuration conflict".to_string())
        }
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
) -> Result<Monitor, MonitorError> {
    let mut fields = Vec::new();
    let mut values: Vec<JsValue> = Vec::new();

    console_log!("update monitor: {monitor:?}");

    if let Some(name) = monitor.name {
        fields.push("name = ?".to_string());
        values.push(JsValue::from_str(&name));
    }

    if let Some(kind) = monitor.kind {
        fields.push("kind = ?".to_string());
        let kind_str = kind.to_string();
        values.push(JsValue::from_str(&kind_str));
    }

    if let Some(config) = monitor.config {
        config.validate()?;
        let config_json = config.to_json()?;
        fields.push("config_json = ?".to_string());
        values.push(JsValue::from_str(&config_json));
    }

    if let Some(enabled) = monitor.enabled {
        fields.push("enabled = ?".to_string());
        values.push(js_number(enabled as i64));
    }

    if let Some(ref relay_id) = monitor.relay_id {
        let relay_id = relay_id.trim();
        if !relay_id.is_empty() {
            get_relay_by_id(d1, relay_id)
                .await
                .map_err(relay_error_to_monitor_error)?;
            fields.push("relay_id = ?".to_string());
            values.push(JsValue::from_str(relay_id));
        }
    }

    if fields.is_empty() {
        return Err(MonitorError::NoFieldsToUpdate);
    }

    fields.push("updated_at = ?".to_string());
    values.push(js_number(now_ms()));

    values.push(JsValue::from_str(monitor_id));
    values.push(JsValue::from_str(org_id));

    let sql = format!(
        "UPDATE monitors SET {} WHERE id = ? AND org_id = ? RETURNING id, org_id, name, kind, enabled, config_json, status, last_checked_at, last_failed_at, first_checked_at, rt_ms, region, relay_id, last_error, next_run_at, created_at, updated_at",
        fields.join(", ")
    );

    let query = d1
        .prepare(&sql)
        .bind(&values)
        .map_err(MonitorError::DbBind)?;

    let result = query.first::<crate::d1c::queries::monitors::GetMonitorByIdRow>(None)
        .await
        .map_err(MonitorError::DbRun)?;

    match result {
        Some(row) => {
            let monitor = Monitor::try_from(row)?;
            console_log!("updated monitor: {monitor_id}");
            Ok(monitor)
        }
        None => Err(MonitorError::NotFound),
    }
}

#[tracing::instrument(
    name = "monitors.update_status",
    skip(d1),
    fields(monitor_id = %heartbeat.monitor_id, org_id = %heartbeat.org_id, timestamp = %heartbeat.timestamp)
)]
pub async fn update_monitor_status_for_org(
    d1: &D1Database,
    heartbeat: &HeartbeatResult,
    snapshot: &MonitorStatusSnapshot,
) -> Result<(), MonitorError> {
    let now = now_ms();
    let first_checked_at = snapshot.first_checked_at.unwrap_or(now);

    // Determine last_failed_at: only update when transitioning to a down state
    let last_failed_at = if heartbeat.status.is_down() && !snapshot.status.is_down() {
        now
    } else {
        snapshot.last_failed_at.unwrap_or(0)
    };

    let last_error = heartbeat.error.clone().unwrap_or_else(|| {
        if heartbeat.status.is_down() {
            "Health check failed".to_string()
        } else {
            String::new()
        }
    });

    update_monitor_status(
        d1,
        heartbeat.status.to_string().as_str(),
        heartbeat.timestamp,
        last_failed_at,
        first_checked_at,
        heartbeat.latency_ms,
        last_error.as_str(),
        now,
        &heartbeat.monitor_id,
        &heartbeat.org_id,
    )
    .await
    .map_err(MonitorError::DbRun)?;

    Ok(())
}
