use crate::d1c::queries::organizations::GetOrganizationByIdRow;
use serde::{Deserialize, Serialize};
use worker::console_error;

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
    pub ae_sample_rate: f64,
}

impl From<GetOrganizationByIdRow> for Organization {
    fn from(row: GetOrganizationByIdRow) -> Self {
        Organization {
            id: row.id.unwrap_or_default(),
            slug: row.slug,
            name: row.name,
            created_at: row.created_at,
            owner_id: row.owner_id,
            ae_sample_rate: row.ae_sample_rate,
        }
    }
}

#[derive(Debug)]
pub enum OrganizationError {
    DbRun(worker::Error),
    NotFound,
}

impl From<worker::Error> for OrganizationError {
    fn from(err: worker::Error) -> Self {
        OrganizationError::DbRun(err)
    }
}

impl From<OrganizationError> for axum::http::StatusCode {
    fn from(err: OrganizationError) -> axum::http::StatusCode {
        match err {
            OrganizationError::DbRun(err) => {
                console_error!("organizations.db.run: {err:?}");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            OrganizationError::NotFound => {
                console_error!("organizations.not.found");
                axum::http::StatusCode::NOT_FOUND
            }
        }
    }
}
