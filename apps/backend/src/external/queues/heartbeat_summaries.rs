use worker::{Context, Env, MessageBatch, Result};
use crate::cloudflare::queues::heartbeat_summary_types::HeartbeatSummary;

pub async fn process_batch(batch: &MessageBatch<serde_json::Value>, _env: Env, _ctx: Context) -> Result<()> {
    let summaries: Vec<HeartbeatSummary> = batch.iter().filter_map(|msg_result| {
        match msg_result {
            Ok(msg) => serde_json::from_value::<HeartbeatSummary>(msg.body().clone()).ok(),
            Err(e) => {
                worker::console_error!("Failed to deserialize heartbeat summary: {:?}", e);
                None
            }
        }
    }).collect();

    if !summaries.is_empty() {
        // write to AE
    }
    Ok(())
}