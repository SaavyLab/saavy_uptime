use std::result::Result;
use worker::{
    console_error, console_log, wasm_bindgen::JsValue, Fetch, Method, Request, RequestInit,
    Response,
};

use crate::cloudflare::d1::get_d1;
use crate::internal::types::{CheckResult, DispatchError, DispatchRequest, MonitorKind};
use crate::router::AppState;
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

pub async fn handle_dispatch(
    state: AppState,
    payload: DispatchRequest,
) -> Result<(), DispatchError> {
    let d1 = get_d1(&state.env())
        .map_err(|err| DispatchError::database("dispatch.start.db.init", err))?;
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

    let check = check_monitor(&state, &payload, start).await;

    let (end_ms, dispatch_status) = match &check {
        Ok(result) => (result.end_ms.unwrap_or(0), "completed"),
        Err(DispatchError::CheckFailed(result)) => (result.end_ms.unwrap_or(0), "failed"),
        Err(_err) => (now_ms(), "failed"),
    };

    update_dispatch(&state, &payload.dispatch_id, end_ms, dispatch_status).await?;

    Ok(())
}

async fn update_dispatch(
    state: &AppState,
    dispatch_id: &str,
    end_ms: i64,
    dispatch_status: &str,
) -> Result<(), DispatchError> {
    let d1 = get_d1(&state.env())
        .map_err(|err| DispatchError::database("dispatch.update.db.init", err))?;

    let complete_statement = d1.prepare(
        "UPDATE monitor_dispatches
         SET status = ?1,
             completed_at_ts = ?2
         WHERE id = ?3",
    );

    let complete_bind_values = vec![
        dispatch_status.into(),
        js_number(end_ms),
        dispatch_id.into(),
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

async fn check_monitor(
    state: &AppState,
    payload: &DispatchRequest,
    start: i64,
) -> Result<CheckResult, DispatchError> {
    let check = match &payload.kind {
        MonitorKind::Http => check_http_monitor(&state, &payload, start).await,
        MonitorKind::Tcp => check_tcp_monitor(&state, &payload, start).await,
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
            write_heartbeat(&state, &payload.dispatch_id, &payload.monitor_id, &failure).await?;
            return Err(DispatchError::CheckFailed(failure));
        }
    };

    write_heartbeat(&state, &payload.dispatch_id, &payload.monitor_id, &result).await?;
    Ok(result)
}

async fn write_heartbeat(
    state: &AppState,
    dispatch_id: &str,
    monitor_id: &str,
    result: &CheckResult,
) -> Result<(), DispatchError> {
    let d1 = get_d1(&state.env())
        .map_err(|err| DispatchError::database("dispatch.heartbeat.db.init", err))?;

    let statement = d1.prepare(
        "INSERT INTO heartbeats (
            monitor_id, 
            dispatch_id,
            ts,
            ok,
            code,
            rtt_ms,
            err,
            region
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    );

    let end = result.end_ms.unwrap_or(0);
    let ok = result.ok;
    let code = result.status_code.unwrap_or(0);
    let rtt_ms = result.rtt_ms.unwrap_or(0);
    let err_msg = result.error_msg.clone();
    let region = result.colo.clone();

    let bind_values = vec![
        JsValue::from_str(monitor_id),
        JsValue::from_str(dispatch_id),
        js_number(end),
        JsValue::from_f64(if ok { 1.0 } else { 0.0 }),
        js_number(code as i64),
        js_number(rtt_ms),
        match err_msg {
            Some(msg) => JsValue::from_str(&msg),
            None => JsValue::NULL,
        },
        if region.is_empty() {
            JsValue::NULL
        } else {
            JsValue::from_str(&region)
        },
    ];

    let query = statement
        .bind(&bind_values)
        .map_err(|err| DispatchError::database("dispatch.heartbeat.bind", err))?;

    if let Err(err) = query.run().await {
        console_error!(
            "dispatch.heartbeat.run failed (monitor_id={}, dispatch_id={}): {err:?}",
            monitor_id,
            dispatch_id
        );
        return Err(DispatchError::database("dispatch.heartbeat.run", err));
    }

    Ok(())
}

const MAX_REDIRECT_DEPTH: u8 = 10;

async fn check_http_monitor(
    _state: &AppState,
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
    _state: &AppState,
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
