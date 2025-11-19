use worker::D1Database;
use worker::Result;
#[tracing::instrument(name = "d1c.dispatch_monitor", skip(d1))]
pub async fn dispatch_monitor(
    d1: &D1Database,
    status: &str,
    dispatched_at_ts: i64,
    id: &str,
) -> Result<()> {
    let stmt = d1
        .prepare("UPDATE monitor_dispatches SET status = ?1, dispatched_at_ts = ?2 WHERE id = ?3");
    let stmt = stmt.bind(&[status.into(), (dispatched_at_ts as f64).into(), id.into()])?;
    stmt.run().await?;
    Ok(())
}
#[tracing::instrument(name = "d1c.complete_dispatch", skip(d1))]
pub async fn complete_dispatch(
    d1: &D1Database,
    status: &str,
    completed_at_ts: i64,
    id: &str,
) -> Result<()> {
    let stmt =
        d1.prepare("UPDATE monitor_dispatches SET status = ?1, completed_at_ts = ?2 WHERE id = ?3");
    let stmt = stmt.bind(&[status.into(), (completed_at_ts as f64).into(), id.into()])?;
    stmt.run().await?;
    Ok(())
}
#[tracing::instrument(name = "d1c.create_dispatch", skip(d1))]
pub async fn create_dispatch(
    d1: &D1Database,
    id: &str,
    monitor_id: &str,
    org_id: &str,
    status: &str,
    scheduled_for_ts: i64,
    created_at: i64,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "INSERT INTO monitor_dispatches (id, monitor_id, org_id, status, scheduled_for_ts, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        );
    let stmt = stmt.bind(&[
        id.into(),
        monitor_id.into(),
        org_id.into(),
        status.into(),
        (scheduled_for_ts as f64).into(),
        (created_at as f64).into(),
    ])?;
    stmt.run().await?;
    Ok(())
}
