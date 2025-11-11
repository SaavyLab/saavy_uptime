use console_error_panic_hook::set_once as set_panic_hook;
use worker::*;

pub mod monitors;
pub mod utils;

use monitors::{get_monitor_by_id, get_monitors_by_org_id, create_monitor, Monitor};

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    set_panic_hook();

    Router::new()
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
        .get_async("/monitors/:id", |_, ctx| async move {
            match ctx.param("id") {
                Some(id) => get_monitor_by_id(&ctx, id).await,
                None => Response::error("id is required", 400),
            }
        })
        .get_async("/monitors/org/:org_id", |_, ctx| async move {
            match ctx.param("org_id") {
                Some(org_id) => get_monitors_by_org_id(&ctx, org_id).await,
                None => Response::error("org_id is required", 400),
            }
        })
        .post_async("/monitors", |mut req, ctx| async move {
            let monitor: Monitor = req.json().await?;
            create_monitor(&ctx, monitor).await
        })
        .get("/health", |_, _| Response::ok("ok"))
        .run(req, env)
        .await
}