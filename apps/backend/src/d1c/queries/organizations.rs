use worker::D1Database;
use worker::Result;
pub fn create_organization_stmt(
    d1: &D1Database,
    id: &str,
    slug: &str,
    name: &str,
    owner_id: &str,
    created_at: i64,
) -> Result<worker::D1PreparedStatement> {
    let stmt = d1
        .prepare(
            "INSERT INTO organizations (id, slug, name, owner_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        );
    let stmt = stmt
        .bind(
            &[
                id.into(),
                slug.into(),
                name.into(),
                owner_id.into(),
                (created_at as f64).into(),
            ],
        )?;
    Ok(stmt)
}
#[tracing::instrument(name = "d1c.create_organization", skip(d1))]
pub async fn create_organization(
    d1: &D1Database,
    id: &str,
    slug: &str,
    name: &str,
    owner_id: &str,
    created_at: i64,
) -> Result<()> {
    let stmt = create_organization_stmt(d1, id, slug, name, owner_id, created_at)?;
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
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GetOrganizationMembersRow {
    pub email: String,
    pub role: String,
}
#[tracing::instrument(name = "d1c.get_organization_members", skip(d1))]
pub async fn get_organization_members(
    d1: &D1Database,
    organization_id: &str,
) -> Result<Vec<GetOrganizationMembersRow>> {
    let stmt = d1
        .prepare(
            "SELECT m.email, om.role FROM members AS m JOIN organization_members AS om ON m.identity_id = om.identity_id WHERE om.organization_id = ?1",
        );
    let stmt = stmt.bind(&[organization_id.into()])?;
    let result = stmt.all().await?;
    let rows = result.results::<GetOrganizationMembersRow>()?;
    Ok(rows)
}
