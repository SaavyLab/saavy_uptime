use axum::http::StatusCode;
use serde::Deserialize;
use worker::{console_error, D1Database};

use crate::d1c::queries::select_org_member;

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
    match select_org_member(&d1, identity_id).await {
        Ok(Some(row)) => Ok(Membership {
            organization_id: row.organization_id,
            role: row.role,
        }),
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
