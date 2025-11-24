use std::{f64, str::FromStr};

use serde::{Deserialize, Serialize};
use worker::Result;

use crate::monitors::types::MonitorStatus;

use super::client::AeQueryClient;

#[derive(Debug, Clone, Copy)]
pub struct TimeWindow {
    pub since_ms: i64,
    pub until_ms: i64,
}

impl TimeWindow {
    pub fn last_hours(hours: i64, now_ms: i64) -> Self {
        let duration = hours.max(1) * 3_600_000;
        Self {
            since_ms: now_ms - duration,
            until_ms: now_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatSample {
    pub timestamp_ms: i64,
    pub status: MonitorStatus,
    pub latency_ms: i64,
    pub region: Option<String>,
    pub colo: Option<String>,
    pub error: Option<String>,
    pub code: Option<u16>,
    pub sample_rate: f64,
    pub dispatch_id: Option<String>,
}

#[derive(Deserialize)]
struct SqlResponse<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
struct HeartbeatRow {
    timestamp_ms: f64,
    status: String,
    latency_ms: Option<f64>,
    region: Option<String>,
    colo: Option<String>,
    error: Option<String>,
    code: Option<f64>,
    sample_rate: Option<f64>,
    dispatch_id: Option<String>,
}

pub async fn recent_heartbeats(
    client: &AeQueryClient,
    monitor_id: &str,
    org_id: &str,
    window: &TimeWindow,
    limit: usize,
) -> Result<Vec<HeartbeatSample>> {
    let dataset = client.dataset();
    let sql = format!(
        r#"SELECT
            index1 as monitor_id,
            blob1 as org_id,
            blob2 as dispatch_id,
            double1 as timestamp_ms,
            blob3 as status,
            blob4 as region,
            blob5 as colo,
            blob6 as error,
            double2 as latency_ms,
            double3 as code,
            double4 as sample_rate,
        FROM `{dataset}`
        WHERE monitor_id = '{monitor}'
          AND org_id = '{org}'
          AND timestamp_ms BETWEEN {since} AND {until}
        ORDER BY timestamp_ms DESC
        LIMIT {limit}
        FORMAT JSON"#,
        dataset = dataset,
        monitor = escape_literal(monitor_id),
        org = escape_literal(org_id),
        since = window.since_ms,
        until = window.until_ms,
        limit = limit.max(1),
    );

    let response: SqlResponse<HeartbeatRow> = client.query(&sql).await?;
    let mut samples = Vec::with_capacity(response.data.len());

    for row in response.data {
        let status = MonitorStatus::from_str(&row.status).unwrap_or(MonitorStatus::Down);
        let code = row.code.and_then(|value| {
            let rounded = value.round() as i64;
            if rounded <= 0 {
                None
            } else {
                Some(rounded as u16)
            }
        });
        let sample_rate = row.sample_rate.unwrap_or(1.0).max(f64::MIN_POSITIVE);

        samples.push(HeartbeatSample {
            timestamp_ms: row.timestamp_ms.round() as i64,
            status,
            latency_ms: row.latency_ms.unwrap_or_default().round() as i64,
            region: normalize_string(row.region),
            colo: normalize_string(row.colo),
            error: normalize_string(row.error),
            code,
            sample_rate,
            dispatch_id: normalize_string(row.dispatch_id),
        });
    }

    Ok(samples)
}

fn escape_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn normalize_string(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
