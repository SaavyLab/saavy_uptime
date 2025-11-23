use worker::{AnalyticsEngineDataset, Context, Env, MessageBatch, Result};

pub async fn process_batch(
    batch: &MessageBatch<serde_json::Value>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let dataset: AnalyticsEngineDataset = env.analytics_engine("AE_TRACES")?;
    hb_tracing::handle_json_batch(batch, &dataset)?;
    Ok(())
}
