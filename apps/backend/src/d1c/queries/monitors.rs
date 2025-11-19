use worker::D1Database;
use worker::Result;
#[tracing::instrument(name = "d1c.create_monitor", skip(d1))]
pub async fn create_monitor(
    d1: &D1Database,
    id: &str,
    org_id: &str,
    name: &str,
    kind: &str,
    enabled: i64,
    config_json: &str,
    status: &str,
    created_at: i64,
    updated_at: i64,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "INSERT INTO monitors (id, org_id, name, kind, enabled, config_json, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        );
    let stmt = stmt
        .bind(
            &[
                id.into(),
                org_id.into(),
                name.into(),
                kind.into(),
                (enabled as f64).into(),
                config_json.into(),
                status.into(),
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
    pub enabled: i64,
    pub config_json: String,
    pub status: String,
    pub last_checked_at: Option<i64>,
    pub last_failed_at: Option<i64>,
    pub first_checked_at: Option<i64>,
    pub rt_ms: Option<i64>,
    pub region: Option<String>,
    pub last_error: Option<String>,
    pub next_run_at: Option<i64>,
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
    pub enabled: i64,
    pub config_json: String,
    pub status: String,
    pub last_checked_at: Option<i64>,
    pub last_failed_at: Option<i64>,
    pub first_checked_at: Option<i64>,
    pub rt_ms: Option<i64>,
    pub region: Option<String>,
    pub last_error: Option<String>,
    pub next_run_at: Option<i64>,
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
#[tracing::instrument(name = "d1c.update_monitor_status", skip(d1))]
pub async fn update_monitor_status(
    d1: &D1Database,
    status: &str,
    last_checked_at: i64,
    last_failed_at: i64,
    first_checked_at: i64,
    rt_ms: i64,
    last_error: &str,
    updated_at: i64,
    id: &str,
    org_id: &str,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "UPDATE monitors SET status = ?1, last_checked_at = ?2, last_failed_at = ?3, first_checked_at = ?4, rt_ms = ?5, last_error = ?6, updated_at = ?7 WHERE id = ?8 AND org_id = ?9",
        );
    let stmt = stmt
        .bind(
            &[
                status.into(),
                (last_checked_at as f64).into(),
                (last_failed_at as f64).into(),
                (first_checked_at as f64).into(),
                (rt_ms as f64).into(),
                last_error.into(),
                (updated_at as f64).into(),
                id.into(),
                org_id.into(),
            ],
        )?;
    stmt.run().await?;
    Ok(())
}
