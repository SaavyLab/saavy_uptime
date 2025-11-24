use std::{str::FromStr, time::Duration};

use cuid2::create_id;
use futures::stream::{self, StreamExt, TryStreamExt};
use js_sys::wasm_bindgen::JsValue;
use serde::Deserialize;
use serde_json::to_string;
use worker::*;

use crate::{
    cloudflare::durable_objects::ticker_types::{
        DispatchPayload, MonitorDispatchRow, TickerConfig, TickerError, TickerState,
    },
    d1c::queries::{
        monitors::{list_due_monitors, update_monitor_next_run_at_stmt},
        organizations::get_org_sample_rate,
    },
    dispatch_state::record_pending_dispatch,
    internal::types::MonitorKind,
    monitors::types::HttpMonitorConfig,
    utils::date::now_ms,
};

const DEFAULT_TICK_INTERVAL_MS: u64 = 15_000;
const DEFAULT_BATCH_SIZE: usize = 100;
const MIN_REARM_DELAY_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;
const MAX_CONCURRENT_DISPATCHES: usize = 20;

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

    #[tracing::instrument(
        name = "external.durable_objects.ticker.arm_alarm",
        skip(self, delay_ms),
        fields(delay_ms = %delay_ms)
    )]
    async fn arm_alarm(&self, delay_ms: u64) -> std::result::Result<(), TickerError> {
        let clamped = delay_ms.max(MIN_REARM_DELAY_MS);
        self.state
            .storage()
            .set_alarm(Duration::from_millis(clamped))
            .await
            .map_err(|err| TickerError::arm_alarm("ticker.arm_alarm", err))
    }

    async fn bootstrap(
        &self,
        mut state: TickerState,
        req: &mut Request,
    ) -> std::result::Result<(), TickerError> {
        #[derive(Deserialize)]
        struct Payload {
            org_id: String,
            tick_interval_ms: Option<u64>,
            batch_size: Option<usize>,
        }

        let payload: Payload = req.json().await?;
        let config = TickerConfig {
            org_id: payload.org_id,
            tick_interval_ms: payload.tick_interval_ms.unwrap_or(DEFAULT_TICK_INTERVAL_MS),
            batch_size: payload.batch_size.unwrap_or(DEFAULT_BATCH_SIZE),
        };

        let delay = config.tick_interval_ms;
        state.config = Some(config);
        state.last_tick_ts = now_ms();
        state.consecutive_errors = 0;
        self.save_state(&state)
            .await
            .map_err(|err| TickerError::save_state("ticker.bootstrap", err))?;
        self.arm_alarm(delay).await?;

        Ok(())
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.poke",
        skip(self),
        fields(manual = true)
    )]
    async fn poke(&self) -> std::result::Result<(), TickerError> {
        self.run_tick(true).await?;
        Ok(())
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.run_tick",
        skip(self, _manual),
        fields(manual = %_manual)
    )]
    async fn run_tick(&self, _manual: bool) -> std::result::Result<(), TickerError> {
        let mut state = self.load_state().await?;
        let config = match state.config.as_ref() {
            Some(cfg) => cfg.clone(),
            None => return Ok(()),
        };

        let claimed = self.claim_due_monitors(&config).await?;
        let sample_rate = self.load_sample_rate(&config.org_id).await?;
        let claimed_count = claimed.len();

        self.dispatch_monitors(&config, claimed, sample_rate)
            .await?;

        state.last_tick_ts = now_ms();
        state.consecutive_errors = 0;
        self.save_state(&state).await?;

        if claimed_count >= config.batch_size {
            self.arm_alarm(MIN_REARM_DELAY_MS).await?;
        } else {
            self.arm_alarm(config.tick_interval_ms).await?;
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.claim_due_monitors",
        skip(self, config),
        fields(org_id = %config.org_id)
    )]
    async fn claim_due_monitors(
        &self,
        config: &TickerConfig,
    ) -> std::result::Result<Vec<MonitorDispatchRow>, TickerError> {
        let d1 = self.env.d1("DB")?;
        let now = now_ms();

        let rows = list_due_monitors(&d1, &config.org_id, Some(now), config.batch_size as i64)
            .await
            .map_err(|err| TickerError::database("ticker.claim.list_due", err))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut statements = Vec::with_capacity(rows.len());
        let mut claimed = Vec::with_capacity(rows.len());
        for row in rows {
            let monitor_id = row.id.clone().unwrap_or_default();
            let kind = MonitorKind::from_str(&row.kind)
                .map_err(|_| TickerError::unknown("ticker.claim.kind_parse", row.kind.clone()))?;

            if kind != MonitorKind::Http {
                return Err(TickerError::unsupported_monitor_kind(
                    "ticker.claim.unsupported_kind",
                    kind,
                ));
            }
            let next_run_at = now + config.tick_interval_ms as i64;
            let update_statement = update_monitor_next_run_at_stmt(
                &d1,
                next_run_at,
                now,
                now,
                &monitor_id,
                &config.org_id,
            )?;

            statements.push(update_statement);

            let config: HttpMonitorConfig =
                serde_json::from_str(&row.config_json).map_err(|err| {
                    TickerError::unknown("ticker.claim.config_parse", err.to_string())
                })?;

            let scheduled_for_ts = row.next_run_at.unwrap_or(now);

            claimed.push(MonitorDispatchRow {
                id: monitor_id,
                kind,
                config,
                scheduled_for_ts,
                status: row.status,
                first_checked_at: row.first_checked_at,
                last_failed_at: row.last_failed_at,
            });
        }

        d1.batch(statements)
            .await
            .map_err(|err| TickerError::database("ticker.claim.batch", err))?;

        Ok(claimed)
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.dispatch_monitors",
        skip(self, config, monitors, sample_rate),
        fields(org_id = %config.org_id, monitors = %monitors.len(), sample_rate = %sample_rate)
    )]
    async fn dispatch_monitors(
        &self,
        config: &TickerConfig,
        monitors: Vec<MonitorDispatchRow>,
        sample_rate: f64,
    ) -> std::result::Result<(), TickerError> {
        if monitors.is_empty() {
            return Ok(());
        }

        let concurrency = config.batch_size.max(1).min(MAX_CONCURRENT_DISPATCHES);

        stream::iter(monitors.into_iter())
            .map(|monitor| self.dispatch_monitor(config, monitor, sample_rate))
            .buffer_unordered(concurrency)
            .try_collect::<Vec<_>>()
            .await
            .map(|_| ())
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.dispatch_monitor",
        skip(self, config, monitor, sample_rate),
        fields(org_id = %config.org_id, monitor_id = %monitor.id, sample_rate = %sample_rate)
    )]
    async fn dispatch_monitor(
        &self,
        config: &TickerConfig,
        monitor: MonitorDispatchRow,
        sample_rate: f64,
    ) -> std::result::Result<(), TickerError> {
        let dispatch_id = create_id().to_string();
        self.record_pending_dispatch(&config.org_id, &monitor, &dispatch_id)
            .await?;
        self.send_dispatch_request(&dispatch_id, &config.org_id, &monitor, sample_rate)
            .await
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.record_pending_dispatch",
        skip(self, org_id, monitor, dispatch_id),
        fields(org_id = %org_id, monitor_id = %monitor.id, dispatch_id = %dispatch_id)
    )]
    async fn record_pending_dispatch(
        &self,
        org_id: &str,
        monitor: &MonitorDispatchRow,
        dispatch_id: &str,
    ) -> std::result::Result<(), TickerError> {
        let d1 = self.env.d1("DB")?;
        let now = now_ms();
        record_pending_dispatch(
            &d1,
            &monitor.id,
            org_id,
            dispatch_id,
            monitor.scheduled_for_ts,
            now,
        )
        .await
        .map_err(|err| TickerError::database("ticker.dispatch.hot", err))
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.send_dispatch_request",
        skip(self, dispatch_id, org_id, monitor, sample_rate),
        fields(dispatch_id = %dispatch_id, org_id = %org_id, monitor_id = %monitor.id, sample_rate = %sample_rate)
    )]
    async fn send_dispatch_request(
        &self,
        dispatch_id: &str,
        org_id: &str,
        monitor: &MonitorDispatchRow,
        sample_rate: f64,
    ) -> std::result::Result<(), TickerError> {
        let service = self
            .env
            .service("DISPATCH_SERVICE")
            .map_err(|err| TickerError::request("ticker.dispatch.service", err))?;
        let token = self
            .env
            .var("DISPATCH_TOKEN")
            .map_err(|_| TickerError::missing_var("ticker.dispatch.token", "DISPATCH_TOKEN"))?
            .to_string();

        let payload = DispatchPayload {
            dispatch_id: dispatch_id.to_string(),
            monitor_id: monitor.id.clone(),
            org_id: org_id.to_string(),
            monitor_url: monitor.config.url.clone(),
            kind: monitor.kind.clone(),
            scheduled_for_ts: monitor.scheduled_for_ts,
            timeout_ms: monitor.config.timeout,
            follow_redirects: monitor.config.follow_redirects,
            verify_tls: monitor.config.verify_tls,
            status: monitor.status.clone(),
            first_checked_at: monitor.first_checked_at,
            last_failed_at: monitor.last_failed_at,
            sample_rate,
        };

        let body = to_string(&payload).map_err(|err| {
            TickerError::request(
                "ticker.dispatch.serialize",
                worker::Error::SerdeJsonError(err),
            )
        })?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(JsValue::from_str(&body)));

        let mut req = Request::new_with_init("/api/internal/dispatch/run", &init)?;
        {
            let headers = req.headers_mut()?;
            headers
                .set("Content-Type", "application/json")
                .map_err(|err| TickerError::request("ticker.dispatch.headers", err))?;
            headers
                .set("X-Dispatch-Token", &token)
                .map_err(|err| TickerError::request("ticker.dispatch.headers", err))?;
        }

        let response = service
            .fetch_request(req)
            .await
            .map_err(|err| TickerError::request("ticker.dispatch.fetch", err))?;

        if response.status().as_u16() >= 400 {
            return Err(TickerError::response_status(
                "ticker.dispatch.response",
                response.status().as_u16(),
            ));
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.load_sample_rate",
        skip(self, org_id),
        fields(org_id = %org_id)
    )]
    async fn load_sample_rate(&self, org_id: &str) -> std::result::Result<f64, TickerError> {
        let d1 = self.env.d1("DB")?;
        let rate = get_org_sample_rate(&d1, org_id)
            .await
            .map_err(|err| TickerError::database("ticker.sample_rate.fetch", err))?
            .map(|row| row.ae_sample_rate)
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);
        Ok(rate)
    }
}

impl DurableObject for Ticker {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.fetch",
        skip(self, req),
        fields(method = %req.method(), path = %req.path())
    )]
    async fn fetch(&self, mut req: Request) -> Result<Response> {
        let method = req.method().clone();
        let path = req.path();
        match (method, path.as_str()) {
            (Method::Post, "/internal/bootstrap") => {
                let state = self.load_state().await?;
                match self.bootstrap(state, &mut req).await {
                    Ok(()) => Response::ok("ok"),
                    Err(err) => Err(err.into()),
                }
            }
            (Method::Post, "/internal/poke") => match self.poke().await {
                Ok(()) => Response::ok("ok"),
                Err(err) => Err(err.into()),
            },
            (Method::Get, "/internal/status") => {
                let state = self.load_state().await?;
                Response::from_json(&state)
            }
            _ => Response::error("Not found", 404),
        }
    }

    #[tracing::instrument(
        name = "external.durable_objects.ticker.alarm",
        skip(self),
    )]
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
