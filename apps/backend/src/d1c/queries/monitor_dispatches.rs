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
        .prepare(
            "UPDATE monitor_dispatches SET status = ?1, dispatched_at_ts = ?2 WHERE id = ?3",
        );
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
    let stmt = d1
        .prepare(
            "UPDATE monitor_dispatches SET status = ?1, completed_at_ts = ?2 WHERE id = ?3",
        );
    let stmt = stmt.bind(&[status.into(), (completed_at_ts as f64).into(), id.into()])?;
    stmt.run().await?;
    Ok(())
}
