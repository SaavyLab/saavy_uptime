use crate::utils::date::now_ms;
use crate::heartbeats::types::GetHeartbeatsParams;
use crate::heartbeats::types::Heartbeat;
use crate::heartbeats::types::HeartbeatError;
use js_sys::wasm_bindgen::JsValue;
use worker::D1Database;
use worker::console_log;

#[tracing::instrument(
    name = "heartbeats.get_by_monitor_id",
    skip(d1),
    fields(monitor_id = %monitor_id)
)]
pub async fn get_heartbeats_by_monitor_id(
    d1: &D1Database,
    org_id: &str,
    monitor_id: &str,
    params: GetHeartbeatsParams,
) -> Result<Vec<Heartbeat>, HeartbeatError> {
  let limit = params.limit.unwrap_or(50);
  let before = params.before.unwrap_or(now_ms());

  let bind_values = vec![
    JsValue::from_str(org_id),
    JsValue::from_str(monitor_id),
    JsValue::from_f64(before as f64),
    JsValue::from_f64(limit as f64),
  ];

  console_log!("bind_values: {bind_values:?}");

  let statement = d1.prepare(
    "SELECT h.*, m.org_id FROM heartbeats h
      INNER JOIN monitors m ON h.monitor_id = m.id
      WHERE m.org_id = ?1 AND h.monitor_id = ?2 AND h.ts < ?3
      ORDER BY h.ts DESC 
      LIMIT ?4
  ");
  let query = statement.bind(&bind_values).map_err(HeartbeatError::DbBind)?;
  let all = query.all().await.map_err(HeartbeatError::DbRun)?;
  let results = all.results::<Heartbeat>().map_err(HeartbeatError::DbRun)?;
  console_log!("found {} heartbeats", results.len());
  Ok(results)
}