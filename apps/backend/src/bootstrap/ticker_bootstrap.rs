use crate::{
    bootstrap::types::BootstrapError, cloudflare::durable_objects::ticker::AppTicker,
    d1c::queries::organizations::select_all_org_ids,
};
use serde::Serialize;
use std::result::Result;
use worker::{console_error, wasm_bindgen::JsValue, D1Database, Method, Request, RequestInit};

#[derive(Serialize)]
struct BootstrapPayload<'a> {
    org_id: &'a str,
}

#[tracing::instrument(
    name = "bootstrap.ensure_ticker_bootstrapped",
    skip(ticker, org_id),
    fields(org_id = %org_id)
)]
pub async fn ensure_ticker_bootstrapped(
    ticker: &AppTicker,
    org_id: &str,
) -> Result<(), BootstrapError> {
    let stub = ticker.stub(org_id)?;

    let body = serde_json::to_string(&BootstrapPayload { org_id })
        .map_err(|err| worker::Error::RustError(format!("ticker bootstrap serialize: {err:?}")))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_body(Some(JsValue::from_str(&body)));

    let mut req = Request::new_with_init("https://ticker/internal/bootstrap", &init)?;
    req.headers_mut()?.set("Content-Type", "application/json")?;

    stub.fetch_with_request(req).await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TickerReconcileSummary {
    pub organizations: usize,
    pub bootstrapped: usize,
    pub failed: usize,
}

#[tracing::instrument(name = "bootstrap.ensure_all_tickers", skip(ticker, d1))]
pub async fn ensure_all_tickers(
    ticker: &AppTicker,
    d1: &D1Database,
) -> Result<TickerReconcileSummary, BootstrapError> {
    let rows = select_all_org_ids(d1).await?;

    let mut summary = TickerReconcileSummary {
        organizations: rows.len(),
        bootstrapped: 0,
        failed: 0,
    };

    for org in rows {
        let Some(org_id) = org.id else {
            continue;
        };

        if let Err(err) = ensure_ticker_bootstrapped(ticker, &org_id).await {
            console_error!(
                "ticker.ensure_all: bootstrap failed for {}: {err:?}",
                org_id
            );
            summary.failed += 1;
        } else {
            summary.bootstrapped += 1;
        }
    }

    Ok(summary)
}
