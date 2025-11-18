use crate::{auth::current_user::CurrentUser, cloudflare::d1::AppDb};
use axum::{
    extract::Path,
    http::StatusCode,
    response::Result,
    Json,
};
use cuid2::create_id;
use serde::{Deserialize, Serialize};
use crate::d1c::queries::{create_organization, get_organization_by_id};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreateOrganization {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Organization {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub created_at: i64,
    pub owner_id: String,
}

impl From<crate::d1c::queries::GetOrganizationByIdRow> for Organization {
    fn from(row: crate::d1c::queries::GetOrganizationByIdRow) -> Self {
        Organization {
            id: row.id,
            slug: row.slug,
            name: row.name,
            created_at: row.created_at,
            owner_id: row.owner_id,
        }
    }
}

#[worker::send]
pub async fn get_organization_by_id_handler(
    AppDb(d1): AppDb,
    Path(id): Path<String>,
    CurrentUser {
        email: _,
        subject: _,
        claims: _,
    }: CurrentUser,
) -> Result<Json<Organization>, StatusCode> {
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
    let _ = create_organization(&d1, &id, &payload.slug, &payload.name).await;

    match get_organization_by_id(&d1, &id).await {
        Ok(Some(organization)) => Ok(Json(organization.into())),
        Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
