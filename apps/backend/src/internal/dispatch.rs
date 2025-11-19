use std::result::Result;
use worker::D1Database;
use worker::{console_error, console_log, Fetch, Method, Request, RequestInit, Response};

use crate::d1c::queries::{
    heartbeats::write_heartbeat,
    monitor_dispatches::{complete_dispatch, dispatch_monitor},
};
use crate::internal::types::{CheckResult, DispatchError, DispatchRequest, MonitorKind};
use crate::utils::date::now_ms;

pub async fn handle_dispatch(
    d1: D1Database,
    payload: DispatchRequest,
) -> Result<(), DispatchError> {
    let start = now_ms();

    dispatch_monitor(&d1, "running", start, &payload.dispatch_id).await?;

    let check = check_monitor(&d1, &payload, start).await;

    let (end_ms, dispatch_status) = match &check {
        Ok(result) => (result.end_ms.unwrap_or(0), "completed"),
        Err(DispatchError::CheckFailed(result)) => (result.end_ms.unwrap_or(0), "failed"),
        Err(_err) => (now_ms(), "failed"),
    };

    complete_dispatch(&d1, dispatch_status, end_ms, &payload.dispatch_id).await?;

    Ok(())
}

async fn check_monitor(
    d1: &D1Database,
    payload: &DispatchRequest,
    start: i64,
) -> Result<CheckResult, DispatchError> {
    let check = match &payload.kind {
        MonitorKind::Http => check_http_monitor(&payload, start).await,
        MonitorKind::Tcp => check_tcp_monitor(&payload, start).await,
        MonitorKind::Udp => todo!("This will be handled by the container protocol adapter"),
    };

    // write_ae_metrics(state, &monitor, ok, duration);

    let result = match check {
        Ok(res) => res,
        Err(err) => {
            let rtt_ms = now_ms() - start;
            let failure = CheckResult {
                ok: false,
                status_code: None,
                rtt_ms: Some(rtt_ms),
                end_ms: Some(now_ms()),
                error_msg: Some(format!("{err:?}")),
                colo: String::new(),
                extra: None,
            };
            write_heartbeat_handler(&d1, &payload.dispatch_id, &payload.monitor_id, &failure)
                .await?;
            return Err(DispatchError::CheckFailed(failure));
        }
    };

    write_heartbeat_handler(&d1, &payload.dispatch_id, &payload.monitor_id, &result).await?;
    Ok(result)
}

async fn write_heartbeat_handler(
    d1: &D1Database,
    dispatch_id: &str,
    monitor_id: &str,
    result: &CheckResult,
) -> Result<(), DispatchError> {
    write_heartbeat(
        d1,
        monitor_id,
        dispatch_id,
        now_ms(),
        result.ok as i64,
        result.status_code.unwrap_or(0) as i64,
        result.rtt_ms.unwrap_or(0) as i64,
        result.error_msg.clone().unwrap_or_default().as_str(),
        result.colo.clone().as_str(),
    )
    .await?;

    Ok(())
}

const MAX_REDIRECT_DEPTH: u8 = 10;

async fn check_http_monitor(
    payload: &DispatchRequest,
    start: i64,
) -> Result<CheckResult, DispatchError> {
    let mut next_url = payload.monitor_url.clone();
    let follow_redirects = payload.follow_redirects;

    for depth in 0..MAX_REDIRECT_DEPTH {
        let response = match perform_fetch(&next_url, payload.timeout_ms, payload.verify_tls).await
        {
            Ok(resp) => resp,
            Err(DispatchError::Heartbeat(err)) => {
                let end = now_ms();
                return Err(DispatchError::CheckFailed(CheckResult {
                    ok: false,
                    status_code: None,
                    rtt_ms: Some(end - start),
                    end_ms: Some(end),
                    error_msg: Some(format!("HTTP fetch error: {err:?}")),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
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
                return Ok(CheckResult {
                    ok: true,
                    status_code: Some(response.status_code()),
                    rtt_ms: Some(end - start),
                    end_ms: Some(end),
                    error_msg: None,
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                });
            }
            300..=399 if follow_redirects => {
                let location = match response.headers().get("Location") {
                    Ok(Some(loc)) => loc,
                    Ok(None) => {
                        return Err(DispatchError::CheckFailed(CheckResult {
                            ok: false,
                            status_code: Some(response.status_code()),
                            rtt_ms: Some(end - start),
                            end_ms: Some(end),
                            error_msg: Some("Redirect location not found".to_string()),
                            colo: String::new(), // todo(saavy): get from cf headers
                            extra: None,
                        }));
                    }
                    Err(err) => {
                        return Err(DispatchError::CheckFailed(CheckResult {
                            ok: false,
                            status_code: Some(response.status_code()),
                            rtt_ms: Some(end - start),
                            error_msg: Some(format!("Redirect location not found {err:?}")),
                            end_ms: Some(end),
                            colo: String::new(), // todo(saavy): get from cf headers
                            extra: None,
                        }));
                    }
                };

                next_url = location;
                continue;
            }
            300..=399 => {
                return Err(DispatchError::CheckFailed(CheckResult {
                    ok: false,
                    status_code: Some(response.status_code()),
                    rtt_ms: Some(end - start),
                    end_ms: Some(end),
                    error_msg: Some("Redirection not enabled".to_string()),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                }));
            }
            400..=499 => {
                console_error!(
                    "HTTP check failed {} {} {}",
                    payload.monitor_id,
                    payload.monitor_url,
                    response.status_code()
                );
                return Err(DispatchError::CheckFailed(CheckResult {
                    ok: false,
                    status_code: Some(response.status_code()),
                    rtt_ms: Some(end - start),
                    end_ms: Some(end),
                    error_msg: Some("Client error".to_string()),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                }));
            }
            _ => {
                console_error!(
                    "HTTP check failed {} {} {}",
                    payload.monitor_id,
                    payload.monitor_url,
                    response.status_code()
                );
                return Err(DispatchError::CheckFailed(CheckResult {
                    ok: false,
                    status_code: Some(response.status_code()),
                    rtt_ms: Some(end - start),
                    end_ms: Some(end),
                    error_msg: Some("Server error".to_string()),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                }));
            }
        }
    }

    console_error!(
        "HTTP check failed after {} redirects for monitor {}",
        MAX_REDIRECT_DEPTH,
        payload.monitor_id
    );
    Err(DispatchError::CheckFailed(CheckResult {
        ok: false,
        status_code: Some(429),
        rtt_ms: Some(now_ms() - start),
        end_ms: Some(now_ms()),
        error_msg: Some("Too many redirects".to_string()),
        colo: String::new(), // todo(saavy): get from cf headers
        extra: None,
    }))
}

async fn perform_fetch(
    url: &str,
    _timeout_ms: i64,
    _verify_tls: bool,
) -> Result<Response, DispatchError> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    // todo(saavy): use timeout/verify_tls

    let mut req =
        Request::new_with_init(&url, &init).map_err(|err| DispatchError::Heartbeat(err))?;
    let headers = req
        .headers_mut()
        .map_err(|err| DispatchError::Heartbeat(err))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|err| DispatchError::Heartbeat(err))?;

    let response = Fetch::Request(req)
        .send()
        .await
        .map_err(|err| DispatchError::Heartbeat(err))?;

    Ok(response)
}

async fn check_tcp_monitor(
    _payload: &DispatchRequest,
    start: i64,
) -> Result<CheckResult, DispatchError> {
    let end = now_ms();
    Err(DispatchError::CheckFailed(CheckResult {
        ok: false,
        status_code: Some(500),
        rtt_ms: Some(end - start),
        end_ms: Some(end),
        error_msg: Some("TCP check not implemented".to_string()),
        colo: String::new(), // todo(saavy): get from cf headers
        extra: None,
    }))
}
