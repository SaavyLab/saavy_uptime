use worker::D1Database;
use worker::Result;
pub fn create_member_stmt(
    d1: &D1Database,
    identity_id: &str,
    email: &str,
    is_workspace_admin: i64,
    created_at: i64,
    updated_at: i64,
) -> Result<worker::D1PreparedStatement> {
    let stmt = d1
        .prepare(
            "INSERT INTO members (identity_id, email, is_workspace_admin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(identity_id) DO UPDATE SET email = excluded.email, updated_at = excluded.updated_at",
        );
    let stmt = stmt
        .bind(
            &[
                identity_id.into(),
                email.into(),
                (is_workspace_admin as f64).into(),
                (created_at as f64).into(),
                (updated_at as f64).into(),
            ],
        )?;
    Ok(stmt)
}
#[tracing::instrument(name = "d1c.create_member", skip(d1))]
pub async fn create_member(
    d1: &D1Database,
    identity_id: &str,
    email: &str,
    is_workspace_admin: i64,
    created_at: i64,
    updated_at: i64,
) -> Result<()> {
    let stmt = create_member_stmt(
        d1,
        identity_id,
        email,
        is_workspace_admin,
        created_at,
        updated_at,
    )?;
    stmt.run().await?;
    Ok(())
}
pub fn create_organization_member_stmt(
    d1: &D1Database,
    organization_id: &str,
    identity_id: &str,
    role: &str,
    created_at: i64,
    updated_at: i64,
) -> Result<worker::D1PreparedStatement> {
    let stmt = d1
        .prepare(
            "INSERT INTO organization_members (organization_id, identity_id, role, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        );
    let stmt = stmt
        .bind(
            &[
                organization_id.into(),
                identity_id.into(),
                role.into(),
                (created_at as f64).into(),
                (updated_at as f64).into(),
            ],
        )?;
    Ok(stmt)
}
#[tracing::instrument(name = "d1c.create_organization_member", skip(d1))]
pub async fn create_organization_member(
    d1: &D1Database,
    organization_id: &str,
    identity_id: &str,
    role: &str,
    created_at: i64,
    updated_at: i64,
) -> Result<()> {
    let stmt = create_organization_member_stmt(
        d1,
        organization_id,
        identity_id,
        role,
        created_at,
        updated_at,
    )?;
    stmt.run().await?;
    Ok(())
}
