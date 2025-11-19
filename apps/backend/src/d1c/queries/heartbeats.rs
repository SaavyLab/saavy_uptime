use worker::D1Database;
use worker::Result;
#[tracing::instrument(name = "d1c.write_heartbeat", skip(d1))]
pub async fn write_heartbeat(
    d1: &D1Database,
    monitor_id: &str,
    dispatch_id: &str,
    ts: i64,
    ok: i64,
    code: i64,
    rtt_ms: i64,
    err: &str,
    region: &str,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "INSERT INTO heartbeats (monitor_id, dispatch_id, ts, ok, code, rtt_ms, err, region) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        );
    let stmt = stmt
        .bind(
            &[
                monitor_id.into(),
                dispatch_id.into(),
                (ts as f64).into(),
                (ok as f64).into(),
                (code as f64).into(),
                (rtt_ms as f64).into(),
                err.into(),
                region.into(),
            ],
        )?;
    stmt.run().await?;
    Ok(())
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GetHeartbeatsByMonitorIdRow {
    pub monitor_id: String,
    pub ts: i64,
    pub ok: i64,
    pub code: Option<i64>,
    pub rtt_ms: Option<i64>,
    pub err: Option<String>,
    pub region: Option<String>,
    pub dispatch_id: Option<String>,
    pub org_id: String,
}
#[tracing::instrument(name = "d1c.get_heartbeats_by_monitor_id", skip(d1))]
pub async fn get_heartbeats_by_monitor_id(
    d1: &D1Database,
    org_id: &str,
    monitor_id: &str,
    before: i64,
    limit: i64,
) -> Result<Vec<GetHeartbeatsByMonitorIdRow>> {
    let stmt = d1
        .prepare(
            "SELECT h.*, m.org_id FROM heartbeats AS h INNER JOIN monitors AS m ON h.monitor_id = m.id WHERE m.org_id = ?1 AND h.monitor_id = ?2 AND h.ts < ?3 ORDER BY h.ts DESC LIMIT ?4",
        );
    let stmt = stmt
        .bind(
            &[
                org_id.into(),
                monitor_id.into(),
                (before as f64).into(),
                (limit as f64).into(),
            ],
        )?;
    let result = stmt.all().await?;
    let rows = result.results::<GetHeartbeatsByMonitorIdRow>()?;
    Ok(rows)
}
