use console_error_panic_hook::set_once as set_panic_hook;
use worker::*;

pub mod monitors;
pub mod organizations;
pub mod router;
pub mod utils;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    set_panic_hook();

    // Axum router
    Ok(router::create_router(env))
}
