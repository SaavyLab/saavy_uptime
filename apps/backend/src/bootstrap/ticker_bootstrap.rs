use serde::{Deserialize, Serialize};
use worker::{
    console_error, wasm_bindgen::JsValue, Env, Method, Request, RequestInit, Result,
};

#[derive(Serialize)]
struct BootstrapPayload<'a> {
    org_id: &'a str,
}

pub async fn ensure_ticker_bootstrapped(env: &Env, org_id: &str) -> Result<()> {
    let namespace = env.durable_object("TICKER")?;
    let id = namespace.id_from_name(org_id)?;
    let stub = id.get_stub()?;

    let body = serde_json::to_string(&BootstrapPayload { org_id })
        .map_err(|err| worker::Error::RustError(format!("ticker bootstrap serialize: {err:?}")))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_body(Some(JsValue::from_str(&body)));

    let mut req = Request::new_with_init("https://ticker/internal/bootstrap", &init)?;
    req.headers_mut()?
        .set("Content-Type", "application/json")?;

    stub.fetch_with_request(req).await?;
    Ok(())
}

#[derive(Deserialize)]
struct OrgRow {
    id: String,
}

#[derive(Debug, Serialize)]
pub struct TickerReconcileSummary {
    pub organizations: usize,
    pub bootstrapped: usize,
    pub failed: usize,
}

pub async fn ensure_all_tickers(env: &Env) -> Result<TickerReconcileSummary> {
    let d1 = env.d1("DB")?;
    let statement = d1.prepare("SELECT id FROM organizations");
    let rows = statement
        .all()
        .await?
        .results::<OrgRow>()
        .map_err(|err| worker::Error::RustError(format!("ticker bootstrap rows: {err:?}")))?;

    let mut summary = TickerReconcileSummary {
        organizations: rows.len(),
        bootstrapped: 0,
        failed: 0,
    };

    for org in rows {
        if let Err(err) = ensure_ticker_bootstrapped(env, &org.id).await {
            console_error!("ticker.ensure_all: bootstrap failed for {}: {err:?}", org.id);
            summary.failed += 1;
        } else {
            summary.bootstrapped += 1;
        }
    }

    Ok(summary)
}
