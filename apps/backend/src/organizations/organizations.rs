use cuid2::create_id;
use serde::{Deserialize, Serialize};
use worker::{wasm_bindgen::JsValue, Response, Result, RouteContext};

use crate::utils::wasm_types::js_number;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CreateOrganization {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub created_at: i64,
}

pub async fn get_organization_by_id(ctx: &RouteContext<()>, id: &str) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let statement = d1.prepare("SELECT * FROM organizations WHERE id = ?1");
    let query = statement.bind(&[id.into()])?;
    let organization = query.first::<Organization>(None).await?;

    match organization {
        Some(org) => Response::from_json(&org),
        None => Response::error("Organization not found", 404),
    }
}

pub async fn create_organization(
    ctx: &RouteContext<()>,
    payload: CreateOrganization,
) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let statement = d1
        .prepare("INSERT INTO organizations (id, slug, name, created_at) VALUES (?1, ?2, ?3, ?4)");
    let id = create_id().to_string();
    let created_at = 1_762_845_925;

    let bind_values = vec![
        JsValue::from_str(&id),
        JsValue::from_str(&payload.slug),
        JsValue::from_str(&payload.name),
        js_number(created_at),
    ];

    let query = statement.bind(&bind_values)?;
    query.run().await?;

    let organization = Organization {
        id,
        slug: payload.slug,
        name: payload.name,
        created_at,
    };

    Response::from_json(&organization)
}
