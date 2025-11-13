use js_sys::wasm_bindgen::JsValue;
use worker::*;
use std::result::Result;

use crate::cloudflare::d1::get_d1;
use crate::internal::types::DispatchRequest;
use crate::monitors::service::get_monitor_by_id;
use crate::monitors::types::Monitor;
use crate::router::AppState;
use crate::utils::date::now_ms;
use crate::utils::errors::{internal_error, rust_error};
use crate::utils::wasm_types::js_number;

async fn handle_dispatch(state: AppState, payload: DispatchRequest) -> Result<(), Error> {
    let d1 = get_d1(&state).map_err(|_| Error::RustError("Failed to get D1".to_string()))?;
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
        .map_err(|err| internal_error("dispatch.update.start.bind", err))?;

    start_query
        .run()
        .await
        .map_err(|err| internal_error("dispatch.update.start", err))?;

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
        .map_err(|err| internal_error("dispatch.update.complete.bind", err))?;

    complete_query
        .run()
        .await
        .map_err(|err| internal_error("dispatch.update.complete.run", err))?;

    Ok(())
}

async fn check_monitor(state: &AppState, payload: &DispatchRequest, start: i64) -> Result<CheckResult, CheckError> {
    let monitor = get_monitor_by_id(&state, payload.monitor_id.clone()).await?;

    let result = match monitor.kind.as_str() {
        "http" => check_http_monitor(&state, &monitor, start).await,
        "tcp" => check_tcp_monitor(&state, &monitor).await,
        "udp" => todo!("This will be handled by the container protocol adapter"),
        _ => return Err(CheckError {
            status_code: 500,
            error_msg: "Unsupported monitor kind".to_string(),
            colo: String::new(), // todo(saavy): get from cf headers
            extra: None,
        }),
    };

    write_heartbeat(
        &state, 
        &payload.dispatch_id, 
        &monitor, 
        start,
        &result,
    );

    // write_ae_metrics(state, &monitor, ok, duration);

    if let Err(err) = write_heartbeat(&state, &payload.dispatch_id, &monitor, start, &result).await {
        console_error!("dispatch.write_heartbeat: {err:?}");
        // this is certainly not right, should write_heartbeat return Result<CheckResult, CheckError>?
        return Err(CheckError {
            status_code: 500,
            error_msg: "Failed to write heartbeat".to_string(),
            colo: String::new(), // todo(saavy): get from cf headers
            extra: None,
        });
    }

    Ok(result?)
}

async fn write_heartbeat(
    state: &AppState, 
    dispatch_id: &str,
    monitor: &Monitor,
    start: i64,
    result: &Result<CheckResult, CheckError>,
) -> Result<(), Error> {
    let d1 = get_d1(&state).map_err(|err| internal_error("write_heartbeat.d1", err))?;
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
    let error = result.error_msg.as_ref().map(|s| s.as_str());
    let code = result.status_code.unwrap_or(0);
    let rtt_ms = result.rtt_ms.unwrap_or(0);
    let err = result.error_msg.as_ref().map(|s| s.as_str());
    let region = result.colo.clone();

    let bind_values = vec![
        monitor.id.clone().into(), 
        dispatch_id.into(),
        js_number(end),
        (if ok { 1 } else { 0 }).into(),
        js_number(code as i64),
        js_number(end - start),
        error.map(|s| s.to_string()).into(),
        JsValue::NULL, // idk how we get this, env var?
    ];

    let query =statement
        .bind(&bind_values)
        .map_err(|err| internal_error("write_heartbeat.bind", err))?;

    query
        .run()
        .await
        .map_err(|err| internal_error("write_heartbeat.run", err))?;

    Ok(())
}

const MAX_REDIRECT_DEPTH: u8 = 10;

struct CheckResult {
    ok: bool,
    status_code: Option<u16>,
    rtt_ms: Option<i64>,
    end_ms: Option<i64>,
    error_msg: Option<String>,
    colo: String,
    extra: Option<serde_json::Value>,
}

struct CheckError {
    status_code: u16,
    error_msg: String,
    colo: String,
    extra: Option<serde_json::Value>,
}

async fn check_http_monitor(_state: &AppState, monitor: &Monitor, start: i64) -> Result<CheckResult, CheckError> {
    let mut next_url = monitor.url.clone();
    let follow_redirects = monitor.follow_redirects != 0;

    for depth in 0..MAX_REDIRECT_DEPTH {
        let response = perform_fetch(&next_url, monitor.timeout_ms, monitor.verify_tls != 0).await?;

        match response.status_code() {
            200..=299 => {
                if depth > 0 { 
                    console_log!("HTTP check passed after {} redirects for monitor {}", depth, monitor.id);
                }
                let end = now_ms();
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
                    Ok(None) => return Err(CheckError {
                        status_code: response.status_code(),
                        error_msg: "Redirect location not found".to_string(),
                        colo: String::new(), // todo(saavy): get from cf headers
                        extra: None,
                    }),
                    Err(err) => return Err(CheckError {
                        status_code: response.status_code(),
                        error_msg: "Redirect location not found".to_string(),
                        colo: String::new(), // todo(saavy): get from cf headers
                        extra: None,
                    }),
                };

                next_url = location;
                continue;
            }
            300..=399 => return Err(CheckError {
                status_code: response.status_code(),
                error_msg: "Redirection not enabled".to_string(),
                colo: String::new(), // todo(saavy): get from cf headers
                extra: None,
            }),
            400..=499 => {
                console_error!("HTTP check failed {} {}", monitor.id, response.status_code());
                return Err(CheckError {
                    status_code: response.status_code(),
                    error_msg: "Client error".to_string(),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                });
            }
            _ => {
                console_error!("HTTP check failed {} {}", monitor.id, response.status_code());
                return Err(CheckError {
                    status_code: response.status_code(),
                    error_msg: "Server error".to_string(),
                    colo: String::new(), // todo(saavy): get from cf headers
                    extra: None,
                });
            }
        }
    }

    console_error!("HTTP check failed after {} redirects for monitor {}", MAX_REDIRECT_DEPTH, monitor.id);
    Err(CheckError {
        status_code: 429,
        error_msg: "Too many redirects".to_string(),
        colo: String::new(), // todo(saavy): get from cf headers
        extra: None,
    })
}

async fn perform_fetch(url: &str, _timeout_ms: i64, _verify_tls: bool) -> Result<Response, Error> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    // todo(saavy): use timeout/foll

    let mut req = Request::new_with_init(&url, &init).map_err(|err| rust_error("dispatch.http.new_request", err))?;
    let headers = req.headers_mut().map_err(|err| rust_error("dispatch.http.headers", err))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|err| rust_error("dispatch.http.headers.set", err))?;

    
    let response = Fetch::Request(req).send().await.map_err(|err| rust_error("dispatch.fetch.http", err))?;

    Ok(response)
}

async fn check_tcp_monitor(_state: &AppState, _monitor: &Monitor) -> Result<CheckResult, CheckError> {
    Ok(CheckResult {
        ok: true,
        status_code: None,
        rtt_ms: None,
        end_ms: None,
        error_msg: None,
        colo: String::new(), // todo(saavy): get from cf headers
        extra: None,
    })
}