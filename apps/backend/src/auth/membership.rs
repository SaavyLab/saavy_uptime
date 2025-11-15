use axum::http::StatusCode;
use serde::Deserialize;
use worker::{console_error, D1Database};

#[derive(Debug, Clone, Deserialize)]
pub struct Membership {
    pub organization_id: String,
    pub role: String,
}

#[derive(Debug)]
pub enum MembershipError {
    DbInit(worker::Error),
    DbBind(worker::Error),
    DbRun(worker::Error),
    NotFound,
}

pub async fn load_membership(
    d1: &D1Database,
    identity_id: &str,
) -> Result<Membership, MembershipError> {
    let statement = d1.prepare(
        "SELECT organization_id, role
         FROM organization_members
         WHERE identity_id = ?1
         ORDER BY created_at DESC
         LIMIT 1",
    );

    let query = statement
        .bind(&[identity_id.into()])
        .map_err(|err| MembershipError::DbBind(err))?;

    match query.first::<Membership>(None).await {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err(MembershipError::NotFound),
        Err(err) => Err(MembershipError::DbRun(err)),
    }
}

impl From<worker::Error> for MembershipError {
    fn from(err: worker::Error) -> Self {
        MembershipError::DbRun(err)
    }
}

impl From<MembershipError> for axum::http::StatusCode {
    fn from(err: MembershipError) -> axum::http::StatusCode {
        match err {
            MembershipError::DbInit(err) => {
                console_error!("membership.db.init: {err:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            MembershipError::DbBind(err) => {
                console_error!("membership.db.bind: {err:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            MembershipError::DbRun(err) => {
                console_error!("membership.db.run: {err:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            MembershipError::NotFound => {
                console_error!("membership.not.found");
                StatusCode::FORBIDDEN
            }
        }
    }
}
