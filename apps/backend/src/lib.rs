#![warn(clippy::disallowed_methods)]

use axum::{body::Body as AxumBody, response::Response as AxumResponse};
use console_error_panic_hook::set_once as set_panic_hook;
use tower_service::Service;
use worker::{Context, Env, HttpRequest, Result};
use worker_macros::event;

pub mod analytics;
pub mod auth;
pub mod bootstrap;
pub mod cloudflare;
pub mod d1c;
pub mod dispatch_state;
pub mod external;
pub mod internal;
pub mod monitors;
pub mod organizations;
pub mod relays;
pub mod router;
pub mod utils;

#[allow(clippy::disallowed_methods)]
#[event(fetch, respond_with_errors)]
pub async fn main(req: HttpRequest, env: Env, _ctx: Context) -> Result<AxumResponse> {
    set_panic_hook();

    let mut router = router::create_router(&env)?;

    let cf = req.extensions().get::<worker::Cf>().cloned();
    let mut request = req.map(AxumBody::new);
    if let Some(cf) = cf {
        request.extensions_mut().insert(cf);
    }

    let response = router.call(request).await?;

    Ok(response)
}
