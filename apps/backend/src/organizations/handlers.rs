use crate::auth::membership::load_membership;
use crate::{auth::current_user::CurrentUser, cloudflare::d1::AppDb};
use axum::{
    http::StatusCode,
    response::Result,
    Json,
};
use cuid2::create_id;
use worker::console_error;
use crate::d1c::queries::organizations::{GetOrganizationMembersRow, create_organization, get_organization_by_id, get_organization_members};
use crate::organizations::types::{CreateOrganization, Organization, OrganizationError};

#[worker::send]
pub async fn get_organization_by_membership_handler(
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Organization>, StatusCode> {
    let membership = load_membership(&d1, &subject).await?;
    match get_organization_by_id(&d1, &membership.organization_id).await {
        Ok(Some(organization)) => Ok(Json(organization.into())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            console_error!("organizations.get_by_membership: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

#[worker::send]
pub async fn get_organization_members_handler(
    AppDb(d1): AppDb,
    CurrentUser { subject, .. }: CurrentUser,
) -> Result<Json<Vec<GetOrganizationMembersRow>>, StatusCode> {
    let membership = load_membership(&d1, &subject).await?;
    match get_organization_members(&d1, &membership.organization_id).await {
        Ok(members) => Ok(Json(members)),
        Err(err) => {
            console_error!("organizations.get_members: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
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
