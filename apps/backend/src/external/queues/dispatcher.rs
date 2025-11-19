use crate::external::queues::{heartbeat_summaries, traces};
use worker::{event, Env, MessageBatch, Result};

#[event(queue)]
pub async fn queue_router(
    batch: MessageBatch<serde_json::Value>,
    env: Env,
    ctx: worker::Context,
) -> Result<()> {
    match batch.queue().as_str() {
        "traces" => traces::process_batch(&batch, env, ctx).await?,
        "heartbeat_summaries" => heartbeat_summaries::process_batch(&batch, env, ctx).await?,
        name => worker::console_log!("Unknown queue: {}", name),
    }
    batch.ack_all();
    Ok(())
}
