use std::sync::OnceLock;

use axum::{body::Body as AxumBody, response::Response as AxumResponse};
use console_error_panic_hook::set_once as set_panic_hook;
use hb_tracing::ConsoleLayer;
use tower_service::Service;
use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt};
use tracing::subscriber::set_global_default;
use worker::{Context, Env, HttpRequest, Result};
use worker_macros::event;

pub mod auth;
pub mod bootstrap;
pub mod cloudflare;
pub mod d1c;
pub mod external;
pub mod heartbeats;
pub mod internal;
pub mod monitors;
pub mod organizations;
pub mod router;
pub mod utils;

static TRACE_SUBSCRIBER: OnceLock<()> = OnceLock::new();

#[event(fetch, respond_with_errors)]
pub async fn main(req: HttpRequest, env: Env, ctx: Context) -> Result<AxumResponse> {
    set_panic_hook();

    // Initialize tracing
    let (buffer, guard) = hb_tracing::buffer_layer();

    TRACE_SUBSCRIBER.get_or_init(|| {
        registry().with(buffer).with(ConsoleLayer).init();
    });

    let mut router = router::create_router(&env);
    let request = req.map(AxumBody::new);

    let response = router.call(request).await?;

    let queue = env.queue("TRACE_QUEUE")?;
    ctx.wait_until(async move {
        if let Err(e) = guard.flush(&queue).await {
            worker::console_error!("Error flushing traces: {:?}", e);
        }
    });

    Ok(response)
}
