use crate::auth::jwt::CfAccessClaims;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub email: String,
    pub subject: String,
    pub claims: CfAccessClaims,
}

impl CurrentUser {
    pub fn new(claims: CfAccessClaims) -> Self {
        Self {
            email: claims.email.clone(),
            subject: claims.sub.clone(),
            claims,
        }
    }
}
