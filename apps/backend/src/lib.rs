use axum::{body::Body as AxumBody, response::Response as AxumResponse};
use console_error_panic_hook::set_once as set_panic_hook;
use tower_service::Service;
use worker::{Context, Env, HttpRequest, Result};
use worker_macros::event;

pub mod auth;
pub mod bootstrap;
pub mod cloudflare;
pub mod heartbeats;
pub mod internal;
pub mod monitors;
pub mod observability;
pub mod organizations;
pub mod router;
pub mod utils;

#[event(fetch, respond_with_errors)]
pub async fn main(req: HttpRequest, env: Env, _ctx: Context) -> Result<AxumResponse> {
    set_panic_hook();
    observability::init(&env);

    let mut router = router::create_router(env);
    let request = req.map(AxumBody::new);

    Ok(router.call(request).await?)
}
