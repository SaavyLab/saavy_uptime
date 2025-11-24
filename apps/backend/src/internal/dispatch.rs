use std::result::Result;
use std::str::FromStr;
use std::time::Duration;

use futures::{future::select, future::Either, pin_mut};
use js_sys::Math;
use worker::D1Database;
use worker::{
    console_error, console_log, AbortController, AnalyticsEngineDataPointBuilder,
    AnalyticsEngineDataset, Cf, Delay, Fetch, Method, Request, RequestInit, Response,
};

use crate::dispatch_state::{finalize_dispatch, mark_dispatch_running};
use crate::internal::types::{DispatchError, DispatchRequest, MonitorKind};
use crate::monitors::service::update_monitor_status_for_org;
use crate::monitors::types::{HeartbeatResult, MonitorStatus, MonitorStatusSnapshot};
use crate::utils::date::now_ms;

pub async fn handle_dispatch(
    d1: D1Database,
    analytics: &AnalyticsEngineDataset,
    payload: DispatchRequest,
    cf: Cf,
) -> Result<(), DispatchError> {
    let start = now_ms();
    let snapshot = monitor_snapshot_from_payload(&payload);
    let region: String = cf.region().unwrap_or("unknown".to_string());
    let colo: String = cf.colo();

    mark_dispatch_running(
        &d1,
        &payload.monitor_id,
        &payload.dispatch_id,
        Some(&colo),
        start,
    )
    .await
    .map_err(|err| DispatchError::database("dispatch.hot.running", err))?;

    let check = check_monitor(&payload, start, &region, &colo).await;
    let fallback_end = now_ms();
    let mut dispatch_status = "completed";
    let mut dispatch_error: Option<String> = None;

    let completion_ts = match check {
        Ok(result) => {
            let ts = result.timestamp;
            persist_heartbeat_result(&d1, analytics, &snapshot, result).await?;
            ts
        }
        Err(DispatchError::CheckFailed(result)) => {
            let ts = result.timestamp;
            let error_text = result
                .error
                .clone()
                .unwrap_or_else(|| "check failed".to_string());
            dispatch_status = "failed";
            dispatch_error = Some(error_text);
            persist_heartbeat_result(&d1, analytics, &snapshot, result).await?;
            ts
        }
        Err(err) => {
            let failure_ts = fallback_end;
            let latency_ms = failure_ts - start;
            let error_message: String = err.into();
            dispatch_status = "failed";
            dispatch_error = Some(error_message.clone());
            persist_heartbeat_result(
                &d1,
                analytics,
                &snapshot,
                HeartbeatResult {
                    monitor_id: payload.monitor_id.clone(),
                    org_id: payload.org_id.clone(),
                    dispatch_id: payload.dispatch_id.clone(),
                    timestamp: failure_ts,
                    status: MonitorStatus::Down,
                    latency_ms,
                    region,
                    colo,
                    sample_rate: payload.sample_rate,
                    error: Some(error_message),
                    code: None,
                },
            )
            .await?;
            failure_ts
        }
    };

    finalize_dispatch(
        &d1,
        &payload.monitor_id,
        &payload.dispatch_id,
        dispatch_status,
        completion_ts,
        dispatch_error.as_deref(),
    )
    .await
    .map_err(|err| DispatchError::database("dispatch.hot.complete", err))?;

    Ok(())
}

async fn persist_heartbeat_result(
    d1: &D1Database,
    analytics: &AnalyticsEngineDataset,
    snapshot: &MonitorStatusSnapshot,
    result: HeartbeatResult,
) -> Result<(), DispatchError> {
    update_monitor_status_for_org(d1, &result, snapshot)
        .await
        .map_err(DispatchError::Monitor)?;
    if should_record(result.sample_rate) {
        write_heartbeat_to_analytics(analytics, &result).map_err(DispatchError::Heartbeat)?;
    }
    Ok(())
}

fn should_record(sample_rate: f64) -> bool {
    if sample_rate >= 1.0 {
        return true;
    }

    if sample_rate <= 0.0 {
        return false;
    }

    Math::random() < sample_rate
}

fn monitor_snapshot_from_payload(payload: &DispatchRequest) -> MonitorStatusSnapshot {
    let status = payload
        .status
        .as_deref()
        .and_then(|raw| MonitorStatus::from_str(raw).ok())
        .unwrap_or(MonitorStatus::Pending);

    MonitorStatusSnapshot {
        status,
        first_checked_at: payload.first_checked_at,
        last_failed_at: payload.last_failed_at,
    }
}

async fn check_monitor(
    payload: &DispatchRequest,
    start: i64,
    region: &String,
    colo: &String,
) -> Result<HeartbeatResult, DispatchError> {
    let check = match &payload.kind {
        MonitorKind::Http => check_http_monitor(payload, start, region.clone(), colo.clone()).await,
        MonitorKind::Tcp => check_tcp_monitor(payload, start, region.clone(), colo.clone()).await,
        MonitorKind::Udp => todo!("This will be handled by the container protocol adapter"),
    };

    let result = match check {
        Ok(res) => res,
        Err(err) => {
            let end = now_ms();
            let rtt_ms = end - start;
            let failure = HeartbeatResult {
                monitor_id: payload.monitor_id.clone(),
                org_id: payload.org_id.clone(),
                dispatch_id: payload.dispatch_id.clone(),
                timestamp: end,
                status: MonitorStatus::Down,
                latency_ms: rtt_ms,
                region: region.to_string(),
                colo: colo.to_string(),
                sample_rate: payload.sample_rate,
                error: Some(format!("{err:?}")),
                code: None,
            };

            return Err(DispatchError::CheckFailed(failure));
        }
    };

    Ok(result)
}

fn write_heartbeat_to_analytics(
    dataset: &AnalyticsEngineDataset,
    event: &HeartbeatResult,
) -> worker::Result<()> {
    let builder = AnalyticsEngineDataPointBuilder::new()
        .indexes(vec![event.monitor_id.as_str(), event.org_id.as_str()])
        .add_blob(event.dispatch_id.as_str())
        .add_double(event.timestamp as f64)
        .add_blob(event.status)
        .add_double(event.latency_ms as f64)
        .add_blob(event.region.as_str())
        .add_blob(event.colo.as_str())
        .add_blob(event.error.as_ref().unwrap_or(&String::new()).as_str())
        .add_double(event.code.unwrap_or(0) as f64)
        .add_double(event.sample_rate);

    dataset.write_data_point(&builder.build())
}

const MAX_REDIRECT_DEPTH: u8 = 10;

async fn check_http_monitor(
    payload: &DispatchRequest,
    start: i64,
    region: String,
    colo: String,
) -> Result<HeartbeatResult, DispatchError> {
    let mut next_url = payload.monitor_url.clone();
    let follow_redirects = payload.follow_redirects;

    for depth in 0..MAX_REDIRECT_DEPTH {
        let response = match perform_fetch(&next_url, payload.timeout_ms, payload.verify_tls).await
        {
            Ok(resp) => resp,
            Err(DispatchError::Heartbeat(err)) => {
                let end = now_ms();
                return Err(DispatchError::CheckFailed(HeartbeatResult {
                    monitor_id: payload.monitor_id.clone(),
                    org_id: payload.org_id.clone(),
                    dispatch_id: payload.dispatch_id.clone(),
                    timestamp: end,
                    status: MonitorStatus::Down,
                    latency_ms: end - start,
                    region,
                    colo,
                    sample_rate: payload.sample_rate,
                    error: Some(format!("HTTP fetch error: {err:?}")),
                    code: None,
                }));
            }
            Err(other) => return Err(other),
        };
        let end = now_ms();

        match response.status_code() {
            200..=299 => {
                if depth > 0 {
                    console_log!(
                        "HTTP check passed after {} redirects for monitor {}",
                        depth,
                        payload.monitor_id
                    );
                }
                console_log!(
                    "HTTP check passed for monitor {} {} at {}",
                    payload.monitor_id,
                    payload.monitor_url,
                    end
                );
                return Ok(HeartbeatResult {
                    monitor_id: payload.monitor_id.clone(),
                    org_id: payload.org_id.clone(),
                    dispatch_id: payload.dispatch_id.clone(),
                    timestamp: end,
                    status: MonitorStatus::Up,
                    latency_ms: end - start,
                    region,
                    colo,
                    sample_rate: payload.sample_rate,
                    error: None,
                    code: None,
                });
            }
            300..=399 if follow_redirects => {
                let location = match response.headers().get("Location") {
                    Ok(Some(loc)) => loc,
                    Ok(None) => {
                        return Err(DispatchError::CheckFailed(HeartbeatResult {
                            monitor_id: payload.monitor_id.clone(),
                            org_id: payload.org_id.clone(),
                            dispatch_id: payload.dispatch_id.clone(),
                            timestamp: end,
                            status: MonitorStatus::Down,
                            latency_ms: end - start,
                            region,
                            colo,
                            sample_rate: payload.sample_rate,
                            error: Some("Redirect location not found".to_string()),
                            code: Some(response.status_code() as u16),
                        }));
                    }
                    Err(err) => {
                        return Err(DispatchError::CheckFailed(HeartbeatResult {
                            monitor_id: payload.monitor_id.clone(),
                            org_id: payload.org_id.clone(),
                            dispatch_id: payload.dispatch_id.clone(),
                            timestamp: end,
                            status: MonitorStatus::Down,
                            latency_ms: end - start,
                            region,
                            colo,
                            sample_rate: payload.sample_rate,
                            error: Some(format!("Redirect location not found {err:?}")),
                            code: Some(response.status_code() as u16),
                        }));
                    }
                };

                next_url = location;
                continue;
            }
            300..=399 => {
                return Err(DispatchError::CheckFailed(HeartbeatResult {
                    monitor_id: payload.monitor_id.clone(),
                    org_id: payload.org_id.clone(),
                    dispatch_id: payload.dispatch_id.clone(),
                    timestamp: end,
                    status: MonitorStatus::Down,
                    latency_ms: end - start,
                    region,
                    colo,
                    sample_rate: payload.sample_rate,
                    error: Some("Redirection not enabled".to_string()),
                    code: None,
                }));
            }
            400..=499 => {
                console_error!(
                    "HTTP check failed {} {} {}",
                    payload.monitor_id,
                    payload.monitor_url,
                    response.status_code()
                );
                return Err(DispatchError::CheckFailed(HeartbeatResult {
                    monitor_id: payload.monitor_id.clone(),
                    org_id: payload.org_id.clone(),
                    dispatch_id: payload.dispatch_id.clone(),
                    timestamp: end,
                    status: MonitorStatus::Down,
                    latency_ms: end - start,
                    region,
                    colo,
                    sample_rate: payload.sample_rate,
                    error: Some("Client error".to_string()),
                    code: Some(response.status_code() as u16),
                }));
            }
            _ => {
                console_error!(
                    "HTTP check failed {} {} {}",
                    payload.monitor_id,
                    payload.monitor_url,
                    response.status_code()
                );
                return Err(DispatchError::CheckFailed(HeartbeatResult {
                    monitor_id: payload.monitor_id.clone(),
                    org_id: payload.org_id.clone(),
                    dispatch_id: payload.dispatch_id.clone(),
                    timestamp: end,
                    status: MonitorStatus::Down,
                    latency_ms: end - start,
                    region,
                    colo,
                    sample_rate: payload.sample_rate,
                    error: Some("Server error".to_string()),
                    code: Some(response.status_code() as u16),
                }));
            }
        }
    }

    console_error!(
        "HTTP check failed after {} redirects for monitor {}",
        MAX_REDIRECT_DEPTH,
        payload.monitor_id
    );
    let end = now_ms();
    Err(DispatchError::CheckFailed(HeartbeatResult {
        monitor_id: payload.monitor_id.clone(),
        org_id: payload.org_id.clone(),
        dispatch_id: payload.dispatch_id.clone(),
        timestamp: end,
        status: MonitorStatus::Down,
        latency_ms: end - start,
        region,
        colo,
        sample_rate: payload.sample_rate,
        error: Some("Too many redirects".to_string()),
        code: None,
    }))
}

const DEFAULT_HTTP_TIMEOUT_MS: i64 = 30_000;

async fn perform_fetch(
    url: &str,
    timeout_ms: i64,
    _verify_tls: bool,
) -> Result<Response, DispatchError> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    // todo(saavy): use timeout/verify_tls

    let mut req = Request::new_with_init(url, &init).map_err(DispatchError::Heartbeat)?;
    let headers = req.headers_mut().map_err(DispatchError::Heartbeat)?;
    headers
        .set("Content-Type", "application/json")
        .map_err(DispatchError::Heartbeat)?;

    let controller = AbortController::default();
    let signal = controller.signal();

    let fetch = Fetch::Request(req);
    let fetch_future = fetch.send_with_signal(&signal);
    let effective_timeout_ms = if timeout_ms <= 0 {
        DEFAULT_HTTP_TIMEOUT_MS
    } else {
        timeout_ms
    } as u64;
    let timeout_future = Delay::from(Duration::from_millis(effective_timeout_ms));

    pin_mut!(fetch_future, timeout_future);

    match select(fetch_future, timeout_future).await {
        Either::Left((result, _)) => result.map_err(DispatchError::Heartbeat),
        Either::Right((_unit, _)) => {
            controller.abort();
            Err(DispatchError::Heartbeat(worker::Error::RustError(
                "HTTP fetch timed out".to_string(),
            )))
        }
    }
}

async fn check_tcp_monitor(
    payload: &DispatchRequest,
    start: i64,
    region: String,
    colo: String,
) -> Result<HeartbeatResult, DispatchError> {
    let end = now_ms();
    Err(DispatchError::CheckFailed(HeartbeatResult {
        monitor_id: payload.monitor_id.clone(),
        org_id: payload.org_id.clone(),
        dispatch_id: payload.dispatch_id.clone(),
        timestamp: end,
        status: MonitorStatus::Down,
        latency_ms: end - start,
        region,
        colo,
        sample_rate: payload.sample_rate,
        error: Some("TCP check not implemented".to_string()),
        code: None,
    }))
}
