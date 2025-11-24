use crate::auth::membership::load_membership;
use crate::bootstrap::ticker_bootstrap::ensure_all_tickers;
use crate::cloudflare::analytics::AppAnalytics;
use crate::cloudflare::d1::AppDb;
use crate::cloudflare::durable_objects::ticker::AppTicker;
use crate::cloudflare::request::RequestCf;
use crate::internal::dispatch::handle_dispatch;
use crate::internal::types::{DispatchRequest, MonitorKind, ReconcileResponse};
use crate::monitors::service::create_monitor_for_org;
use crate::monitors::types::{CreateMonitor, HttpMonitorConfig};
use crate::router::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Result,
    Json,
};
use hb_auth::User;
use serde::{Deserialize, Serialize};
use worker::console_error;

#[worker::send]
#[tracing::instrument(
    name = "internal.handlers.reconcile_tickers_handler",
    skip(ticker, d1, _user),
    fields(user = %_user.sub())
)]
pub async fn reconcile_tickers_handler(
    AppTicker(ticker): AppTicker,
    AppDb(d1): AppDb,
    _user: User,
) -> Result<Json<ReconcileResponse>, StatusCode> {
    match ensure_all_tickers(&ticker, &d1).await {
        Ok(summary) => Ok(Json(ReconcileResponse {
            organizations: summary.organizations,
            bootstrapped: summary.bootstrapped,
            failed: summary.failed,
        })),
        Err(err) => Err(err.into()),
    }
}

#[worker::send]
#[axum::debug_handler]
#[tracing::instrument(
    name = "internal.handlers.dispatch_handler",
    skip(state, d1, analytics, cf, headers, payload),
    fields(payload.monitor_id = %payload.monitor_id, payload.org_id = %payload.org_id, payload.dispatch_id = %payload.dispatch_id)
)]
pub async fn dispatch_handler(
    State(state): State<AppState>,
    AppDb(d1): AppDb,
    AppAnalytics(analytics): AppAnalytics,
    RequestCf(cf): RequestCf,
    headers: HeaderMap,
    Json(payload): Json<DispatchRequest>,
) -> Result<StatusCode, StatusCode> {
    validate_dispatch_token(&state, &headers)?;
    handle_dispatch(d1, &analytics, payload, cf).await?;

    Ok(StatusCode::ACCEPTED)
}

fn validate_dispatch_token(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let expected = state
        .env()
        .var("DISPATCH_TOKEN")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    match headers
        .get("x-dispatch-token")
        .and_then(|value| value.to_str().ok())
    {
        Some(actual) if actual == expected => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SeedRequest {
    pub quantity: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedResponse {
    pub created: usize,
    pub failed: usize,
}

#[worker::send]
pub async fn seed_monitors_handler(
    AppTicker(ticker): AppTicker,
    AppDb(d1): AppDb,
    auth: User,
    Json(payload): Json<SeedRequest>,
) -> Result<Json<SeedResponse>, StatusCode> {
    let membership = load_membership(&d1, auth.sub())
        .await
        .map_err(|err| StatusCode::from(err))?;
    let templates = seed_definitions(payload.quantity.unwrap_or(300));
    let mut created = 0;
    let mut failed = 0;

    for template in templates {
        match create_monitor_for_org(&ticker, &d1, &membership.organization_id, template).await {
            Ok(_) => created += 1,
            Err(err) => {
                failed += 1;
                console_error!("seed.monitor: {err:?}");
            }
        }
    }

    Ok(Json(SeedResponse { created, failed }))
}

fn seed_definitions(quantity: usize) -> Vec<CreateMonitor> {
    let mut monitors = Vec::new();

    let target = quantity / 3;

    fn push_many(
        list: &mut Vec<CreateMonitor>,
        prefix: &str,
        urls: &[&str],
        count: usize,
        follow_redirects: bool,
    ) {
        for (idx, url) in urls.iter().cycle().take(count).enumerate() {
            let config = HttpMonitorConfig::new(url, 60, 7000, true, follow_redirects);
            let monitor = CreateMonitor {
                name: format!("{prefix} #{:03}", idx + 1),
                kind: MonitorKind::Http,
                config,
            };

            list.push(monitor);
        }
    }
    let httpstat = [
        "https://httpstat.us/200",
        "https://httpstat.us/404",
        "https://httpstat.us/503",
        "https://httpstat.us/200?sleep=2000",
    ];
    push_many(&mut monitors, "httpstat", &httpstat, target, true);

    let postman = [
        "https://postman-echo.com/status/200",
        "https://postman-echo.com/status/500",
        "https://postman-echo.com/delay/1",
    ];
    push_many(&mut monitors, "postman", &postman, target, true);

    let real = [
        "https://www.cloudflare.com",
        "https://example.com",
        "https://github.com",
        "https://workers.cloudflare.com",
        "https://www.iana.org/domains/reserved",
    ];
    push_many(&mut monitors, "real", &real, target, true);

    monitors
}
