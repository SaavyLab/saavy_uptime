use crate::auth::membership::load_membership;
use crate::{auth::current_user::CurrentUser, cloudflare::d1::AppDb};
use axum::{
    extract::Path,
    http::StatusCode,
    response::Result,
    Json,
};
use cuid2::create_id;
use crate::d1c::queries::{create_organization, get_organization_by_id};
use crate::organizations::types::{CreateOrganization, Organization, OrganizationError};

#[worker::send]
pub async fn get_organization_by_id_handler(
    AppDb(d1): AppDb,
    Path(id): Path<String>,
    CurrentUser {
        email: _,
        subject,
        claims: _,
    }: CurrentUser,
) -> Result<Json<Organization>, StatusCode> {
    let membership = load_membership(&d1, &subject).await?;
    if membership.organization_id != id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    match get_organization_by_id(&d1, &id).await {
        Ok(Some(organization)) => Ok(Json(organization.into())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[worker::send]
pub async fn create_organization_handler(
    AppDb(d1): AppDb,
    CurrentUser {
        email: _,
        subject: _,
        claims: _,
    }: CurrentUser,
    Json(payload): Json<CreateOrganization>,
) -> Result<Json<Organization>, StatusCode> {
    let id = create_id().to_string();
    create_organization(&d1, &id, &payload.slug, &payload.name)
        .await
        .map_err(OrganizationError::DbRun)?;

    match get_organization_by_id(&d1, &id).await {
        Ok(Some(organization)) => Ok(Json(organization.into())),
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
