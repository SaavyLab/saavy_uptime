use std::time::Duration;

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use worker::*;

use crate::utils::{date::now_ms, wasm_types::js_number};

const DEFAULT_TICK_INTERVAL_MS: u64 = 15_000;
const DEFAULT_BATCH_SIZE: usize = 100;
const MIN_REARM_DELAY_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;

fn internal_error(context: &str, err: impl std::fmt::Debug) -> worker::Error {
    worker::Error::RustError(format!("{context}: {err:?}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TickerConfig {
    org_id: String,
    tick_interval_ms: u64,
    batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TickerState {
    config: Option<TickerConfig>,
    last_tick_ts: i64,
    consecutive_errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitorRow {
    id: String,
    interval_s: i64,
    url: String,
    timeout_ms: i64,
    follow_redirects: i64,
    verify_tls: i64,
}

#[derive(Debug, Clone)]
struct MonitorDispatch {
    id: String,
    url: String,
    scheduled_for_ts: i64,
    timeout_ms: i64,
    follow_redirects: bool,
    verify_tls: bool,
}

impl From<(MonitorRow, i64)> for MonitorDispatch {
    fn from((row, scheduled_for_ts): (MonitorRow, i64)) -> Self {
        Self {
            id: row.id,
            url: row.url,
            scheduled_for_ts,
            timeout_ms: row.timeout_ms,
            follow_redirects: row.follow_redirects != 0,
            verify_tls: row.verify_tls != 0,
        }
    }
}

#[durable_object]
pub struct Ticker {
    state: State,
    env: Env,
}

impl Ticker {
    async fn load_state(&self) -> Result<TickerState> {
        Ok(self
            .state
            .storage()
            .get::<TickerState>("state")
            .await
            .unwrap_or_default())
    }

    async fn save_state(&self, state: &TickerState) -> Result<()> {
        self.state.storage().put("state", state).await
    }

    async fn arm_alarm(&self, delay_ms: u64) -> Result<()> {
        let clamped = delay_ms.max(MIN_REARM_DELAY_MS);
        self.state
            .storage()
            .set_alarm(Duration::from_millis(clamped))
            .await
            .map_err(|err| internal_error("ticker.arm_alarm", err))
    }

    async fn bootstrap(&self, mut state: TickerState, req: &mut Request) -> Result<Response> {
        #[derive(Deserialize)]
        struct Payload {
            org_id: String,
            tick_interval_ms: Option<u64>,
            batch_size: Option<usize>,
        }

        let payload: Payload = req.json().await?;
        let config = TickerConfig {
            org_id: payload.org_id,
            tick_interval_ms: payload
                .tick_interval_ms
                .unwrap_or(DEFAULT_TICK_INTERVAL_MS),
            batch_size: payload.batch_size.unwrap_or(DEFAULT_BATCH_SIZE),
        };

        let delay = config.tick_interval_ms;
        state.config = Some(config);
        state.last_tick_ts = now_ms();
        state.consecutive_errors = 0;
        self.save_state(&state).await?;
        self.arm_alarm(delay).await?;

        Response::ok("bootstrapped")
    }

    async fn poke(&self) -> Result<Response> {
        self.run_tick(true).await?;
        Response::ok("poked")
    }

    async fn status(&self, state: TickerState) -> Result<Response> {
        Response::from_json(&state)
    }

    async fn run_tick(&self, _manual: bool) -> Result<()> {
        let mut state = self.load_state().await?;
        let config = match state.config.as_ref() {
            Some(cfg) => cfg.clone(),
            None => return Ok(()),
        };

        let claimed = self.claim_due_monitors(&config).await?;

        for monitor in &claimed {
            self.dispatch_monitor(&config, monitor).await?;
        }

        state.last_tick_ts = now_ms();
        state.consecutive_errors = 0;
        self.save_state(&state).await?;

        if claimed.len() >= config.batch_size {
            self.arm_alarm(MIN_REARM_DELAY_MS).await?;
        } else {
            self.arm_alarm(config.tick_interval_ms).await?;
        }

        Ok(())
    }

    async fn claim_due_monitors(
        &self,
        config: &TickerConfig,
    ) -> Result<Vec<MonitorDispatch>> {
        let d1 = self.env.d1("DB")?;
        let now = now_ms();

        let select_statement = d1.prepare(
            "
            SELECT id, interval_s, url, timeout_ms, follow_redirects, verify_tls
            FROM monitors
            WHERE org_id = ?1
              AND enabled = 1
              AND (next_run_at_ts IS NULL OR next_run_at_ts <= ?2)
            ORDER BY COALESCE(next_run_at_ts, 0) ASC
            LIMIT ?3
            ",
        );

        let select_query = select_statement
            .bind(&[
                JsValue::from_str(&config.org_id),
                js_number(now),
                js_number(config.batch_size as i64),
            ])
            .map_err(|err| internal_error("ticker.claim.bind", err))?;

        let rows = select_query
            .all()
            .await
            .map_err(|err| internal_error("ticker.claim.select", err))?
            .results::<MonitorRow>()
            .map_err(|err| internal_error("ticker.claim.results", err))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut statements = Vec::with_capacity(rows.len());
        let mut claimed = Vec::with_capacity(rows.len());

        for row in rows {
            let next_run_at = now + row.interval_s * 1_000;
            let update_statement = d1.prepare(
                "
                UPDATE monitors
                SET last_checked_at_ts = ?1,
                    next_run_at_ts = ?2,
                    updated_at = ?1
                WHERE id = ?3
                ",
            );

            let update_query = update_statement
                .bind(&[
                    js_number(now),
                    js_number(next_run_at),
                    JsValue::from_str(&row.id),
                ])
                .map_err(|err| internal_error("ticker.claim.update_bind", err))?;

            statements.push(update_query);
            claimed.push(MonitorDispatch::from((row, next_run_at)));
        }

        d1.batch(statements)
            .await
            .map_err(|err| internal_error("ticker.claim.batch", err))?;

        Ok(claimed)
    }

    async fn dispatch_monitor(
        &self,
        config: &TickerConfig,
        monitor: &MonitorDispatch,
    ) -> Result<()> {
        // Placeholder dispatch for now; future work will POST to the internal runner.
        console_log!(
            "{}: Dispatching monitor {} for org {} (url: {})",
            Date::now().to_string(),
            monitor.id,
            config.org_id,
            monitor.url
        );
        Ok(())
    }
}

impl DurableObject for Ticker {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, mut req: Request) -> Result<Response> {
        let method = req.method().clone();
        let path = req.path();
        match (method, path.as_str()) {
            (Method::Post, "/internal/bootstrap") => {
                let state = self.load_state().await?;
                self.bootstrap(state, &mut req).await
            }
            (Method::Post, "/internal/poke") => self.poke().await,
            (Method::Get, "/internal/status") => {
                let state = self.load_state().await?;
                self.status(state).await
            }
            _ => Response::error("Not found", 404),
        }
    }

    async fn alarm(&self) -> Result<Response> {
        if let Err(err) = self.run_tick(false).await {
            console_log!("ticker alarm error: {err:?}");

            let mut state = self.load_state().await?;
            let tick_interval = state
                .config
                .as_ref()
                .map(|cfg| cfg.tick_interval_ms)
                .unwrap_or(DEFAULT_TICK_INTERVAL_MS);
            state.consecutive_errors = state.consecutive_errors.saturating_add(1);
            let delay = (tick_interval * (state.consecutive_errors as u64 + 1))
                .clamp(MIN_REARM_DELAY_MS, MAX_BACKOFF_MS);
            self.save_state(&state).await?;
            self.arm_alarm(delay).await?;
        }

        Response::ok("ok")
    }
}
