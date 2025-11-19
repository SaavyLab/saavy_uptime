use crate::monitors::types::HeartbeatResult;
use worker::{
    AnalyticsEngineDataPointBuilder, AnalyticsEngineDataset, Context, Env, MessageBatch, Result,
};

pub async fn process_batch(
    batch: &MessageBatch<serde_json::Value>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let summaries: Vec<HeartbeatResult> = batch
        .iter()
        .filter_map(|msg_result| match msg_result {
            Ok(msg) => serde_json::from_value::<HeartbeatResult>(msg.body().clone()).ok(),
            Err(e) => {
                worker::console_error!("Failed to deserialize heartbeat summary: {:?}", e);
                None
            }
        })
        .collect();

    let dataset = env.analytics_engine("AE")?;
    handle_slice(&summaries, &dataset)?;
    batch.ack_all();
    Ok(())
}

fn handle_slice(events: &[HeartbeatResult], dataset: &AnalyticsEngineDataset) -> Result<()> {
    for event in events {
        let builder = AnalyticsEngineDataPointBuilder::new()
            .indexes(vec![event.monitor_id.as_str(), event.org_id.as_str()])
            .add_double(event.timestamp as f64)
            .add_blob(event.status)
            .add_double(event.latency_ms as f64)
            .add_blob(event.region.as_str())
            .add_blob(event.colo.as_str())
            .add_blob(event.error.as_ref().unwrap_or(&String::new()).as_str())
            .add_double(event.code.unwrap_or(0) as f64);

        if let Err(e) = dataset.write_data_point(&builder.build()) {
            worker::console_error!("Failed to write heartbeat result to AE: {:?}", e);
        }
    }

    Ok(())
}
