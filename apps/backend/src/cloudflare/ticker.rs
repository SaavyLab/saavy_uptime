use serde::{Deserialize, Serialize};
use worker::{durable_object, State, Env, Result, Request, Response, DurableObject};
use crate::utils::date::now_ms;
use crate::cloudflare::d1::get_d1;
use crate::utils::wasm_types::js_number;
use axum::http::StatusCode;

const DEFAULT_TICK_INTERVAL_MS: u64 = 15_000;
const DEFAULT_BATCH_SIZE: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TickerConfig {
  org_id: String,
  tick_interval_ms: u64,
  batch_size: usize,
  last_tick_ts: u64,
  consecutive_errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TickerState {
  config: Option<TickerConfig>,
}

impl Default for TickerState {
  fn default() -> Self {
    Self { config: None }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitorRow {
  id: String,
}

#[durable_object]
pub struct Ticker {
  state: State,
  env: Env,
}

impl Ticker {
  async fn load_state(&self) -> Result<TickerState> {
    let storage = self.state.storage();
    Ok(storage
      .get::<TickerState>("state")
      .await?
      .unwrap_or_default())
  }

  async fn save_state(&self, state: TickerState) -> Result<()> {
    let storage = self.state.storage();
    storage.put("state", state).await
  }

  async fn bootstrap(&self, mut state: TickerState, req: Request) -> Result<Response> {
    #[derive(Deserialize)]
    struct Payload {
      org_id: String,
      tick_interval_ms: Option<u64>,
      batch_size: Option<usize>,
    }

    let payload: Payload = req.json().await?;

    state.config = Some(TickerConfig {
      org_id: payload.org_id,
      tick_interval_ms: payload.tick_interval_ms.unwrap_or(DEFAULT_TICK_INTERVAL_MS),
      batch_size: payload.batch_size.unwrap_or(DEFAULT_BATCH_SIZE),
      last_tick_ts: js_sys::Date::now() as u64,
      consecutive_errors: 0,
    });

    self.save_state(state).await?;

    self.arm_alarm(state.config.as_ref().unwrap().tick_interval_ms).await?;

    Response::ok("bootstrapped")
  }

  async fn run_tick(&self, manual: bool) -> Result<()> {
    let mut state = self.load_state().await?;
    let config = match state.config.as_mut() {
      Some(cfg) => cfg,
      None => return Ok(()),
    };

    let now = now_ms();
    let claimed = self.claim_due_monitors(&config).await?;

    for monitor in &claimed {
      self.dispatch_monitor(&config, monitor).await?;
    }

    config.last_tick_ts = now;
    config.consecutive_errors = 0;
    self.save_state(&state).await?;

    if claimed.len() >= config.batch_size {
      self.arm_alarm(1_000).await?;
    } else if manual {
      self.arm_alarm(config.tick_interval_ms).await?;
    } else {
      self.arm_alarm(config.tick_interval_ms).await?;
    }

    Ok(())
  }

  async fn claim_due_monitors(&self, config: &TickerConfig) -> Result<Vec<MonitorRow>> {
    let d1 = get_d1(&self.env).map_err(|err| internal_error("claim_due_monitors.d1", err))?;

    let select_statement = d1.prepare("
      SELECT id 
      FROM monitors 
      WHERE org_id = ?1 
      AND last_tick_ts < ?2 
      AND next_run_at_ts <= ?3
      ORDER BY last_tick_ts ASC 
      LIMIT ?4"
    );
  
    let update_statement = d1.prepare("
      UPDATE monitors 
      SET last_tick_ts = ?1 
      WHERE org_id = ?2
      AND last_tick_ts < ?3
      AND next_run_at_ts <= ?4"
    );

    let select_query = select_statement.bind(&[
      config.org_id.clone(),
      js_number(config.last_tick_ts),
      js_number(now),
      js_number(config.batch_size as i64),
    ]).map_err(|err| internal_error("claim_due_monitors.select_bind", err))?;

    let update_query = update_statement.bind(&[
      js_number(now),
      config.org_id.clone(),
      js_number(config.last_tick_ts),
      js_number(now),
    ]).map_err(|err| internal_error("claim_due_monitors.update_bind", err))?;

    let statements = vec![select_query, update_query];

    let batch_results = d1.batch(statements).await.map_err(|err| {
      console_error!("claim_due_monitors: batch execution failed: {err:?}");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(err) = batch_results.iter().find_map(|result| result.error()) {
      console_error!("claim_due_monitors.statement failed: {err:?}");
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut monitors = Vec::new();
    for row in select_query.results::<MonitorRow>() {

    todo!()
  }

  async fn dispatch_monitor(
    &self,
    config: &TickerConfig,
    monitor: &MonitorRow,
  ) -> Result<()> {
    todo!()
  }
}

impl DurableObject for Ticker {
  fn new(state: State, env: Env) -> Self {
    Self { state, env }
  }

  async fn fetch(&self, req: Request) -> Result<Response> {
    let url = url::Url::parse(&req.url()?.to_string())?;
    let path = url.path().to_string();

    let current_state = self.load_state().await?;

    match (req.method().as_str(), path.as_str()) {
      ("POST", "/internal/bootstrap") => self.bootstrap(current_state, req).await,
      ("POST", "/internal/poke") => self.poke().await,
      ("GET", "/internal/status") => self.status(current_state).await,
      _ => Response::error("Not found", 404),
    }
  }

  async fn alarm(&self) -> Result<Response> {
    if let Err(err) = self.run_tick(false).await {
      let mut state = self.load_state().await?;
      if let Some(cfg) = state.config.as_mut() {
        cfg.consecutive_errors += 1;
        self.save_state(&state).await?;

        let delay = (cfg.tick_interval_ms * (cfg.consecutive_errors + 1) as u64)
          .min(60_000);
        self.arm_alarm(delay).await?;
      }
      console_error!("tick error: {err:?}");
    }
    Ok(())
  }
}