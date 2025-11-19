use worker::D1Database;
use worker::Result;
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
