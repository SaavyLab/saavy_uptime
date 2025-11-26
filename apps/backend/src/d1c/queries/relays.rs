use worker::D1Database;
use worker::Result;
#[tracing::instrument(name = "d1c.insert_relay", skip(d1))]
pub async fn insert_relay(
    d1: &D1Database,
    id: &str,
    slug: &str,
    name: &str,
    location_hint: &str,
    jurisdiction: &str,
    durable_object_id: &str,
    enabled: i64,
    last_bootstrapped_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
) -> Result<()> {
    let stmt = d1
        .prepare(
            "INSERT INTO relays (id, slug, name, location_hint, jurisdiction, durable_object_id, enabled, last_bootstrapped_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        );
    let stmt = stmt.bind(&[
        id.into(),
        slug.into(),
        name.into(),
        location_hint.into(),
        jurisdiction.into(),
        durable_object_id.into(),
        (enabled as f64).into(),
        match last_bootstrapped_at {
            Some(value) => (value as f64).into(),
            None => worker::wasm_bindgen::JsValue::NULL,
        },
        (created_at as f64).into(),
        (updated_at as f64).into(),
    ])?;
    stmt.run().await?;
    Ok(())
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FindRelayBySlugRow {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub location_hint: String,
    pub jurisdiction: String,
    pub durable_object_id: String,
    pub enabled: i64,
    pub last_bootstrapped_at: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
#[tracing::instrument(name = "d1c.find_relay_by_slug", skip(d1))]
pub async fn find_relay_by_slug(d1: &D1Database, slug: &str) -> Result<Option<FindRelayBySlugRow>> {
    let stmt = d1.prepare("SELECT * FROM relays WHERE slug = ?1 LIMIT 1");
    let stmt = stmt.bind(&[slug.into()])?;
    let result = stmt.first::<FindRelayBySlugRow>(None).await?;
    Ok(result)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FindRelayByIdRow {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub location_hint: String,
    pub jurisdiction: String,
    pub durable_object_id: String,
    pub enabled: i64,
    pub last_bootstrapped_at: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
#[tracing::instrument(name = "d1c.find_relay_by_id", skip(d1))]
pub async fn find_relay_by_id(d1: &D1Database, id: &str) -> Result<Option<FindRelayByIdRow>> {
    let stmt = d1.prepare("SELECT * FROM relays WHERE id = ?1 LIMIT 1");
    let stmt = stmt.bind(&[id.into()])?;
    let result = stmt.first::<FindRelayByIdRow>(None).await?;
    Ok(result)
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ListRelaysRow {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub location_hint: String,
    pub jurisdiction: String,
    pub durable_object_id: String,
    pub enabled: i64,
    pub last_bootstrapped_at: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
#[tracing::instrument(name = "d1c.list_relays", skip(d1))]
pub async fn list_relays(d1: &D1Database) -> Result<Vec<ListRelaysRow>> {
    let stmt = d1.prepare("SELECT * FROM relays ORDER BY created_at DESC");
    let result = stmt.all().await?;
    let rows = result.results::<ListRelaysRow>()?;
    Ok(rows)
}
