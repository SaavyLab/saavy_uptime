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
#[tracing::instrument(name = "d1c.create_monitor", skip(d1))]
pub async fn create_monitor(
    d1: &D1Database,
    id: &str,
    org_id: &str,
    name: &str,
    kind: &str,
    url: &str,
    interval_s: i64,
    timeout_ms: i64,
    follow_redirects: i64,
    verify_tls: i64,
    created_at: i64,
    updated_at: i64,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "INSERT INTO monitors (id, org_id, name, kind, url, interval_s, timeout_ms, follow_redirects, verify_tls, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        );
    let stmt = stmt
        .bind(
            &[
                id.into(),
                org_id.into(),
                name.into(),
                kind.into(),
                url.into(),
                (interval_s as f64).into(),
                (timeout_ms as f64).into(),
                (follow_redirects as f64).into(),
                (verify_tls as f64).into(),
                (created_at as f64).into(),
                (updated_at as f64).into(),
            ],
        )?;
    stmt.run().await?;
    Ok(())
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GetMonitorByIdRow {
    pub id: Option<String>,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub interval_s: i64,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
    pub expect_status_low: Option<i64>,
    pub expect_status_high: Option<i64>,
    pub expect_substring: Option<String>,
    pub headers_json: Option<String>,
    pub tags_json: Option<String>,
    pub enabled: i64,
    pub last_checked_at_ts: Option<i64>,
    pub next_run_at_ts: Option<i64>,
    pub current_status: String,
    pub last_ok: i64,
    pub consecutive_failures: i64,
    pub current_incident_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
#[tracing::instrument(name = "d1c.get_monitor_by_id", skip(d1))]
pub async fn get_monitor_by_id(
    d1: &D1Database,
    id: &str,
    org_id: &str,
) -> Result<Option<GetMonitorByIdRow>> {
    let stmt = d1.prepare("SELECT * FROM monitors WHERE id = ?1 AND org_id = ?2");
    let stmt = stmt.bind(&[id.into(), org_id.into()])?;
    let result = stmt.first::<GetMonitorByIdRow>(None).await?;
    Ok(result)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GetMonitorsByOrgIdRow {
    pub id: Option<String>,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub interval_s: i64,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
    pub expect_status_low: Option<i64>,
    pub expect_status_high: Option<i64>,
    pub expect_substring: Option<String>,
    pub headers_json: Option<String>,
    pub tags_json: Option<String>,
    pub enabled: i64,
    pub last_checked_at_ts: Option<i64>,
    pub next_run_at_ts: Option<i64>,
    pub current_status: String,
    pub last_ok: i64,
    pub consecutive_failures: i64,
    pub current_incident_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
#[tracing::instrument(name = "d1c.get_monitors_by_org_id", skip(d1))]
pub async fn get_monitors_by_org_id(
    d1: &D1Database,
    org_id: &str,
) -> Result<Vec<GetMonitorsByOrgIdRow>> {
    let stmt = d1
        .prepare("SELECT * FROM monitors WHERE org_id = ?1 ORDER BY created_at DESC");
    let stmt = stmt.bind(&[org_id.into()])?;
    let result = stmt.all().await?;
    let rows = result.results::<GetMonitorsByOrgIdRow>()?;
    Ok(rows)
}
#[tracing::instrument(name = "d1c.delete_monitor", skip(d1))]
pub async fn delete_monitor(d1: &D1Database, id: &str, org_id: &str) -> Result<()> {
    let stmt = d1.prepare("DELETE FROM monitors WHERE id = ?1 AND org_id = ?2");
    let stmt = stmt.bind(&[id.into(), org_id.into()])?;
    stmt.run().await?;
    Ok(())
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GetOrganizationByIdRow {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub created_at: i64,
    pub owner_id: String,
}
#[tracing::instrument(name = "d1c.get_organization_by_id", skip(d1))]
pub async fn get_organization_by_id(
    d1: &D1Database,
    id: &str,
) -> Result<Option<GetOrganizationByIdRow>> {
    let stmt = d1.prepare("SELECT * FROM organizations WHERE id = ?1");
    let stmt = stmt.bind(&[id.into()])?;
    let result = stmt.first::<GetOrganizationByIdRow>(None).await?;
    Ok(result)
}
#[tracing::instrument(name = "d1c.create_organization", skip(d1, name))]
pub async fn create_organization(
    d1: &D1Database,
    id: &str,
    slug: &str,
    name: &str,
) -> Result<()> {
    let stmt = d1
        .prepare("INSERT INTO organizations (id, slug, name) VALUES (?1, ?2, ?3)");
    let stmt = stmt.bind(&[id.into(), slug.into(), name.into()])?;
    stmt.run().await?;
    Ok(())
}
#[tracing::instrument(name = "d1c.check_if_bootstrapped", skip(d1))]
pub async fn check_if_bootstrapped(d1: &D1Database) -> Result<Option<i64>> {
    let stmt = d1.prepare("SELECT COUNT(*) AS count FROM organizations");
    let result = stmt.first::<i64>(Some("count")).await?;
    Ok(result)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SelectAllOrgIdsRow {
    pub id: Option<String>,
}
#[tracing::instrument(name = "d1c.select_all_org_ids", skip(d1))]
pub async fn select_all_org_ids(d1: &D1Database) -> Result<Vec<SelectAllOrgIdsRow>> {
    let stmt = d1.prepare("SELECT id FROM organizations");
    let result = stmt.all().await?;
    let rows = result.results::<SelectAllOrgIdsRow>()?;
    Ok(rows)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SelectOrgMemberRow {
    pub organization_id: String,
    pub role: String,
}
#[tracing::instrument(name = "d1c.select_org_member", skip(d1))]
pub async fn select_org_member(
    d1: &D1Database,
    identity_id: &str,
) -> Result<Option<SelectOrgMemberRow>> {
    let stmt = d1
        .prepare(
            "SELECT organization_id, role FROM organization_members WHERE identity_id = ?1 ORDER BY created_at DESC LIMIT 1",
        );
    let stmt = stmt.bind(&[identity_id.into()])?;
    let result = stmt.first::<SelectOrgMemberRow>(None).await?;
    Ok(result)
}
