use crate::auth::membership::load_membership;
use crate::cloudflare::d1::AppDb;
use crate::d1c::queries::organizations::{
    create_organization, get_organization_by_id, get_organization_members,
    GetOrganizationMembersRow,
};
use crate::organizations::types::{CreateOrganization, Organization, OrganizationError};
use crate::utils::date::now_ms;
use axum::{http::StatusCode, response::Result, Json};
use cuid2::create_id;
use hb_auth::User;
use worker::console_error;

#[worker::send]
pub async fn get_organization_by_membership_handler(
    AppDb(d1): AppDb,
    auth: User,
) -> Result<Json<Organization>, StatusCode> {
    let membership = load_membership(&d1, auth.sub()).await?;
    match get_organization_by_id(&d1, &membership.organization_id).await {
        Ok(Some(organization)) => Ok(Json(organization.into())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            console_error!("organizations.get_by_membership: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[worker::send]
pub async fn get_organization_members_handler(
    AppDb(d1): AppDb,
    auth: User,
) -> Result<Json<Vec<GetOrganizationMembersRow>>, StatusCode> {
    let membership = load_membership(&d1, auth.sub()).await?;
    match get_organization_members(&d1, &membership.organization_id).await {
        Ok(members) => Ok(Json(members)),
        Err(err) => {
            console_error!("organizations.get_members: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[worker::send]
pub async fn create_organization_handler(
    AppDb(d1): AppDb,
    auth: User,
    Json(payload): Json<CreateOrganization>,
) -> Result<Json<Organization>, StatusCode> {
    let id = create_id().to_string();
    create_organization(&d1, &id, &payload.slug, &payload.name, auth.sub(), now_ms())
        .await
        .map_err(OrganizationError::DbRun)?;

    match get_organization_by_id(&d1, &id).await {
        Ok(Some(organization)) => Ok(Json(organization.into())),
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
