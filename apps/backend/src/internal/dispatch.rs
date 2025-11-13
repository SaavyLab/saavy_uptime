use worker::{console_error, console_log, RequestInit, Method, Request, Fetch, Response};
use std::result::Result;

use crate::cloudflare::d1::get_d1;
use crate::internal::types::{CheckError, CheckResult, DispatchError, DispatchRequest, MonitorKind};
use crate::monitors::service::get_monitor_by_id;
use crate::monitors::types::Monitor;
use crate::router::AppState;
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

pub async fn handle_dispatch(state: AppState, payload: DispatchRequest) -> Result<(), DispatchError> {
    let d1 = get_d1(&state.env()).map_err(|err| DispatchError::database("dispatch.start.db.init", err))?;
    let start = now_ms();

    let start_statement = d1.prepare(
        "UPDATE monitor_dispatches
         SET status = ?1,
             dispatched_at_ts = ?2
         WHERE id = ?3",
    );

    let start_bind_values = vec![
        "running".into(),
        js_number(start),
        payload.dispatch_id.clone().into(),
    ];

    let start_query = start_statement
        .bind(&start_bind_values)
        .map_err(|err| DispatchError::database("dispatch.start.bind", err))?;

    start_query
        .run()
        .await
        .map_err(|err| DispatchError::database("dispatch.start.run", err))?;

    let result = check_monitor(&state, &payload, start).await?;

    let complete_statement = d1.prepare(
        "UPDATE monitor_dispatches
         SET status = ?1,
             completed_at_ts = ?2
         WHERE id = ?3",
    );

    let end = result.end_ms.unwrap_or(0);

    let complete_bind_values = vec![
        "completed".into(),
        js_number(end),
        payload.dispatch_id.into(),
    ];

    let complete_query = complete_statement
        .bind(&complete_bind_values)
        .map_err(|err| DispatchError::database("dispatch.complete.bind", err))?;

    complete_query
        .run()
        .await
        .map_err(|err| DispatchError::database("dispatch.complete.run", err))?;

    Ok(())
}

async fn check_monitor(state: &AppState, payload: &DispatchRequest, start: i64) -> Result<i64, DispatchError> {
    let result = match payload.kind {
        MonitorKind::Http => check_http_monitor(&state, &payload, start).await,
        MonitorKind::Tcp => check_tcp_monitor(&state, &payload).await,
        MonitorKind::Udp => todo!("This will be handled by the container protocol adapter"),
        _ => return Err(DispatchError::Check(CheckError {
            status_code: 500,
            error_msg: "Unsupported monitor kind".to_string(),
            end_ms: Some(now_ms()),
            colo: String::new(), // todo(saavy): get from cf headers
            extra: None,
        })),
    };

    // write_ae_metrics(state, &monitor, ok, duration);
    match result {
        Ok(result) => {
            write_heartbeat(&state, &payload.dispatch_id, &payload.monitor_id, &result).await?;
        }
        Err(err) => {
            todo!("Still actually write the heartbeat for this failed check")
        }
    }

    Ok(result?.end_ms.unwrap_or(0))
}

async fn write_heartbeat(
    state: &AppState, 
    dispatch_id: &str,
    monitor_id: &str,
    result: &CheckResult,
) -> Result<(), DispatchError> {
    let d1 = get_d1(&state.env()).map_err(|err| DispatchError::database("dispatch.heartbeat.db.init", err))?;

    let statement = d1.prepare(
        "INSERT INTO heartbeats (
            monitor_id, 
            dispatch_id,
            ts,
            ok,
            code,
            rtt_ms,
            err,
            region,
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    );

    let end = result.end_ms.unwrap_or(0);
    let ok = result.ok;
    let code = result.status_code.unwrap_or(0);
    let rtt_ms = result.rtt_ms.unwrap_or(0);
    let err_msg = result.error_msg.clone();
    let region = result.colo.clone();

    let bind_values = vec![
        monitor_id.into(),
        js_number(end),
        (if ok { 1 } else { 0 }).into(),
        js_number(code as i64),
        js_number(rtt_ms),
        err_msg.into(),
        region.into(),
        dispatch_id.into(),
    ];

    let query = statement
        .bind(&bind_values)
        .map_err(|err| DispatchError::database("dispatch.heartbeat.bind", err))?;

    query
        .run()
        .await
        .map_err(|err| DispatchError::database("dispatch.heartbeat.run", err))?;

    Ok(())
}

const MAX_REDIRECT_DEPTH: u8 = 10;

async fn check_http_monitor(_state: &AppState, payload: &DispatchRequest, start: i64) -> Result<CheckResult, DispatchError> {
    let mut next_url = payload.monitor_url.clone();
    let follow_redirects = payload.follow_redirects;

    for depth in 0..MAX_REDIRECT_DEPTH {
        let response = perform_fetch(&next_url, payload.timeout_ms, payload.verify_tls).await?;
        let end = now_ms();

        match response.status_code() {
            200..=299 => {
                if depth > 0 { 
                    console_log!("HTTP check passed after {} redirects for monitor {}", depth, payload.monitor_id);
                }
                return Ok(CheckResult {
                    ok: true,
                    status_code: Some(response.status_code()),
                    rtt_ms: Some(end - start),
                    end_ms: Some(end),
                    error_msg: None,
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                });
            },
            300..=399 if follow_redirects => {
                let location = match response.headers().get("Location") {
                    Ok(Some(loc)) => loc,
                    Ok(None) => return Err(DispatchError::Check(CheckError {
                        status_code: response.status_code(),
                        error_msg: "Redirect location not found".to_string(),
                        end_ms: Some(end),
                        colo: String::new(), // todo(saavy): get from cf headers
                        extra: None,
                    })),
                    Err(err) => return Err(DispatchError::Check(CheckError {
                        status_code: response.status_code(),
                        error_msg: format!("Redirect location not found {err:?}"),
                        end_ms: Some(end),
                        colo: String::new(), // todo(saavy): get from cf headers
                        extra: None,
                    })),
                };

                next_url = location;
                continue;
            }
            300..=399 => return Err(DispatchError::Check(CheckError {
                status_code: response.status_code(),
                error_msg: "Redirection not enabled".to_string(),
                end_ms: Some(end),
                colo: String::new(), // todo(saavy): get from cf headers
                extra: None,
            })),
            400..=499 => {
                console_error!("HTTP check failed {} {}", payload.monitor_id, response.status_code());
                return Err(DispatchError::Check(CheckError {
                    status_code: response.status_code(),
                    error_msg: "Client error".to_string(),
                    end_ms: Some(end),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                }));
            }
            _ => {
                console_error!("HTTP check failed {} {}", payload.monitor_id, response.status_code());
                return Err(DispatchError::Check(CheckError {
                    status_code: response.status_code(),
                    error_msg: "Server error".to_string(),
                    end_ms: Some(end),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                }));
            }
        }
    }

    console_error!("HTTP check failed after {} redirects for monitor {}", MAX_REDIRECT_DEPTH, payload.monitor_id);
    Err(DispatchError::Check(CheckError {
        status_code: 429,
        error_msg: "Too many redirects".to_string(),
        end_ms: Some(now_ms()),
        colo: String::new(), // todo(saavy): get from cf headers
        extra: None,
    }))
}

async fn perform_fetch(url: &str, _timeout_ms: i64, _verify_tls: bool) -> Result<Response, DispatchError> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    // todo(saavy): use timeout/verify_tls

    let mut req = Request::new_with_init(&url, &init).map_err(|err| DispatchError::Heartbeat(err))?;
    let headers = req.headers_mut().map_err(|err| DispatchError::Heartbeat(err))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|err| DispatchError::Heartbeat(err))?;

    
    let response = Fetch::Request(req).send().await.map_err(|err| DispatchError::Heartbeat(err))?;

    Ok(response)
}

async fn check_tcp_monitor(_state: &AppState, _payload: &DispatchRequest) -> Result<CheckResult, DispatchError> {
    Err(DispatchError::Check(CheckError {
        status_code: 500,
        error_msg: "TCP check not implemented".to_string(),
        end_ms: Some(now_ms()),
        colo: String::new(), // todo(saavy): get from cf headers
        extra: None,
    }))
}