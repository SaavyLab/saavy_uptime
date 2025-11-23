use crate::external::queues::{heartbeat_results, traces};
use worker::{event, Env, MessageBatch, Result};

#[event(queue)]
pub async fn queue_router(
    batch: MessageBatch<serde_json::Value>,
    env: Env,
    ctx: worker::Context,
) -> Result<()> {
    match batch.queue().as_str() {
        "trace-queue" => traces::process_batch(&batch, env, ctx).await?,
        "heartbeat-queue" => heartbeat_results::process_batch(&batch, env, ctx).await?,
        name => worker::console_log!("Unknown queue: {}", name),
    }
    batch.ack_all();
    Ok(())
}
