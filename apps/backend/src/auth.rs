use hb_auth::{jwt::Claims, RoleMapper};

pub mod membership;

#[derive(Debug, PartialEq, Clone)]
pub enum Role {
    Admin,
    User,
}

impl RoleMapper for Role {
    fn from_claims(claims: &Claims) -> Vec<Self> {
        let mut roles = vec![];
        for group in &claims.groups {
            match group.as_str() {
                "admin" => roles.push(Role::Admin),
                _ => roles.push(Role::User),
            }
        }
        roles
    }
}
