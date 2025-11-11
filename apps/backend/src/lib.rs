use console_error_panic_hook::set_once as set_panic_hook;
use worker::*;

pub mod monitors;
pub mod organizations;
pub mod utils;

use monitors::{create_monitor, get_monitor_by_id, get_monitors_by_org_id, CreateMonitor};
use organizations::{create_organization, get_organization_by_id, CreateOrganization};
use utils::cors::{is_preflight, preflight_response, with_cors};

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    set_panic_hook();

    if is_preflight(&req) {
        return preflight_response(&req);
    }

    let result = Router::new()
        .get_async("/durable", |_, ctx| async move {
            let namespace = ctx.env.durable_object("TICKER")?;
            let id = namespace.id_from_name("TICKER")?;
            let stub = id.get_stub()?;

            stub.fetch_with_str("https://example.com/messages").await
        })
        .get("/secret", |_, ctx| {
            let secret = ctx.secret("CF_API_TOKEN")?;
            Response::ok(secret.to_string())
        })
        .get_async("/api/monitors/:id", |_, ctx| async move {
            match ctx.param("id") {
                Some(id) => get_monitor_by_id(&ctx, id).await,
                None => Response::error("id is required", 400),
            }
        })
        .get_async("/api/monitors/org/:org_id", |_, ctx| async move {
            match ctx.param("org_id") {
                Some(org_id) => get_monitors_by_org_id(&ctx, org_id).await,
                None => Response::error("org_id is required", 400),
            }
        })
        .post_async("/api/monitors", |mut req, ctx| async move {
            let monitor: CreateMonitor = req.json().await?;
            create_monitor(&ctx, monitor).await
        })
        .post_async("/api/organizations", |mut req, ctx| async move {
            let organization: CreateOrganization = req.json().await?;
            create_organization(&ctx, organization).await
        })
        .get_async("/api/organizations/:id", |_, ctx| async move {
            match ctx.param("id") {
                Some(id) => get_organization_by_id(&ctx, id).await,
                None => Response::error("id is required", 400),
            }
        })
        .get("/api/health", |_, _| Response::ok("ok"))
        .run(req, env)
        .await;

    match result {
        Ok(response) => with_cors(response),
        Err(error) => {
            console_error!("worker error: {error:?}");
            let response = Response::error(error.to_string(), 500)?;
            with_cors(response)
        }
    }
}
