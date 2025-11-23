use crate::monitors::types::HeartbeatResult;
use worker::{
    event, AnalyticsEngineDataPointBuilder, AnalyticsEngineDataset, Context, Env, MessageBatch,
    Result,
};

#[event(queue)]
pub async fn process_batch(
    batch: MessageBatch<HeartbeatResult>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let dataset = env.analytics_engine("AE_HEARTBEATS")?;
    for msg in batch.iter() {
        match msg {
            Ok(msg) => handle_event(msg.body(), &dataset)?,
            Err(e) => worker::console_error!("Failed to deserialize heartbeat summary: {:?}", e),
        }
    }
    batch.ack_all();
    Ok(())
}

fn handle_event(event: &HeartbeatResult, dataset: &AnalyticsEngineDataset) -> Result<()> {
    let builder = AnalyticsEngineDataPointBuilder::new()
        .indexes(vec![event.monitor_id.as_str(), event.org_id.as_str()])
        .add_blob(event.dispatch_id.as_str())
        .add_double(event.timestamp as f64)
        .add_blob(event.status)
        .add_double(event.latency_ms as f64)
        .add_blob(event.region.as_str())
        .add_blob(event.colo.as_str())
        .add_blob(event.error.as_ref().unwrap_or(&String::new()).as_str())
        .add_double(event.code.unwrap_or(0) as f64)
        .add_double(event.sample_rate);

    if let Err(e) = dataset.write_data_point(&builder.build()) {
        worker::console_error!("Failed to write heartbeat result to AE: {:?}", e);
    }
    Ok(())
}
