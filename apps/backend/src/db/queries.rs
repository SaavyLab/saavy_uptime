use worker::D1Database;
use worker::Result;
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ListExampleRow {
    pub id: String,
    pub message: String,
}
pub async fn list_example(d1: &D1Database) -> Result<Vec<ListExampleRow>> {
    let stmt = d1.prepare("SELECT 1 as id, 'hello' as message;");
    let result = stmt.all().await?;
    let rows = result.results::<ListExampleRow>()?;
    Ok(rows)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TestTwoRow {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub interval_s: i64,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
    pub expect_status_low: i64,
    pub expect_status_high: i64,
    pub expect_substring: String,
    pub headers_json: String,
    pub tags_json: String,
    pub enabled: i64,
    pub last_checked_at_ts: i64,
    pub next_run_at_ts: i64,
    pub current_status: String,
    pub last_ok: i64,
    pub consecutive_failures: i64,
    pub current_incident_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Debug, Clone)]
pub struct TestTwoParams {
    pub monitor_id: String,
}
pub async fn test_two(
    d1: &D1Database,
    params: &TestTwoParams,
) -> Result<Vec<TestTwoRow>> {
    let stmt = d1.prepare("SELECT *\nFROM monitors\nWHERE id = ?1;");
    let stmt = stmt.bind(&[&params.monitor_id.into()])?;
    let result = stmt.all().await?;
    let rows = result.results::<TestTwoRow>()?;
    Ok(rows)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MultilineWithWhiteSpaceRow {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub interval_s: i64,
    pub timeout_ms: i64,
    pub follow_redirects: i64,
    pub verify_tls: i64,
    pub expect_status_low: i64,
    pub expect_status_high: i64,
    pub expect_substring: String,
    pub headers_json: String,
    pub tags_json: String,
    pub enabled: i64,
    pub last_checked_at_ts: i64,
    pub next_run_at_ts: i64,
    pub current_status: String,
    pub last_ok: i64,
    pub consecutive_failures: i64,
    pub current_incident_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}
pub async fn multiline_with_white_space(
    d1: &D1Database,
) -> Result<Option<MultilineWithWhiteSpaceRow>> {
    let stmt = d1.prepare("SELECT *\nFROM monitors\nWHERE name = 'test';");
    let result = stmt.first::<MultilineWithWhiteSpaceRow>(None).await?;
    Ok(result.map(|r| r.val))
}
