use worker::D1Database;
use worker::Result;
#[tracing::instrument(name = "d1c.upsert_dispatch_pending", skip(d1))]
pub async fn upsert_dispatch_pending(
    d1: &D1Database,
    monitor_id: &str,
    dispatch_id: &str,
    org_id: &str,
    scheduled_for_ts: i64,
    updated_at: i64,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "INSERT INTO monitor_dispatch_hot (monitor_id, dispatch_id, org_id, status, scheduled_for_ts, dispatched_at_ts, completed_at_ts, runner_colo, error, updated_at) VALUES (?1, ?2, ?3, 'pending', ?4, NULL, NULL, NULL, NULL, ?5) ON CONFLICT(monitor_id) DO UPDATE SET dispatch_id = excluded.dispatch_id, org_id = excluded.org_id, status = 'pending', scheduled_for_ts = excluded.scheduled_for_ts, dispatched_at_ts = NULL, completed_at_ts = NULL, runner_colo = NULL, error = NULL, updated_at = excluded.updated_at",
        );
    let stmt = stmt
        .bind(
            &[
                monitor_id.into(),
                dispatch_id.into(),
                org_id.into(),
                (scheduled_for_ts as f64).into(),
                (updated_at as f64).into(),
            ],
        )?;
    stmt.run().await?;
    Ok(())
}
#[tracing::instrument(name = "d1c.mark_dispatch_running", skip(d1))]
pub async fn mark_dispatch_running(
    d1: &D1Database,
    dispatched_at_ts: i64,
    runner_colo: Option<&str>,
    updated_at: i64,
    monitor_id: &str,
    dispatch_id: &str,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "UPDATE monitor_dispatch_hot SET status = 'running', dispatched_at_ts = ?1, runner_colo = ?2, updated_at = ?3 WHERE monitor_id = ?4 AND dispatch_id = ?5",
        );
    let stmt = stmt
        .bind(
            &[
                (dispatched_at_ts as f64).into(),
                match runner_colo {
                    Some(value) => value.into(),
                    None => worker::wasm_bindgen::JsValue::NULL,
                },
                (updated_at as f64).into(),
                monitor_id.into(),
                dispatch_id.into(),
            ],
        )?;
    stmt.run().await?;
    Ok(())
}
#[tracing::instrument(name = "d1c.finalize_dispatch", skip(d1))]
pub async fn finalize_dispatch(
    d1: &D1Database,
    status: &str,
    completed_at_ts: i64,
    error: Option<&str>,
    updated_at: i64,
    monitor_id: &str,
    dispatch_id: &str,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "UPDATE monitor_dispatch_hot SET status = ?1, completed_at_ts = ?2, error = ?3, updated_at = ?4 WHERE monitor_id = ?5 AND dispatch_id = ?6",
        );
    let stmt = stmt
        .bind(
            &[
                status.into(),
                (completed_at_ts as f64).into(),
                match error {
                    Some(value) => value.into(),
                    None => worker::wasm_bindgen::JsValue::NULL,
                },
                (updated_at as f64).into(),
                monitor_id.into(),
                dispatch_id.into(),
            ],
        )?;
    stmt.run().await?;
    Ok(())
}
