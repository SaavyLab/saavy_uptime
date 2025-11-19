use crate::span_event::SpanEvent;
use worker::*;

/// Handles a slice of span events by writing them to the Analytics Engine dataset.
///
/// This function is synchronous because the underlying `write` operation
/// to Analytics Engine is a non-blocking, fire-and-forget binding call.
pub fn handle_slice(events: &[SpanEvent], dataset: &AnalyticsEngineDataset) -> Result<()> {
    for event in events {
        let builder = AnalyticsEngineDataPointBuilder::new()
            .indexes(vec![event.trace_id.as_str()])
            .add_blob(event.span_id.as_str())
            .add_blob(event.parent_id.as_str())
            .add_blob(event.name.as_str())
            .add_blob(event.target.as_str())
            .add_blob(event.level.as_str())
            .add_blob(event.fields.as_str())
            .add_double(event.duration_ms)
            .add_double(event.start_time_ms);

        if let Err(e) = builder.write_to(dataset) {
            // We log the error but continue processing the batch.
            // Telemetry should be best-effort; failing the whole batch
            // because one span is malformed (e.g. too large) is usually bad.
            worker::console_error!("hb-tracing: failed to write span to AE: {:?}", e);
        }
    }

    Ok(())
}

/// Handles a full message batch from a Queue.
///
/// This helper extracts the events, writes them to AE, and acknowledges the batch.
pub fn handle_batch(
    batch: MessageBatch<SpanEvent>,
    dataset: &AnalyticsEngineDataset,
) -> Result<()> {
    let events: Vec<SpanEvent> = batch
        .iter()
        .filter_map(|msg_result| match msg_result {
            Ok(msg) => Some(msg.body().clone()),
            Err(e) => {
                worker::console_error!("hb-tracing: failed to deserialize queue message: {:?}", e);
                None
            }
        })
        .collect();

    handle_slice(&events, dataset)?;

    batch.ack_all();
    Ok(())
}

/// Handles a batch of raw JSON values, useful for "Router" workers that consume mixed queues.
///
/// This helper attempts to deserialize each JSON value into a `SpanEvent`.
/// Messages that fail deserialization are skipped (assumed to belong to another handler).
pub fn handle_json_batch(
    batch: &MessageBatch<serde_json::Value>,
    dataset: &AnalyticsEngineDataset,
) -> Result<()> {
    let spans: Vec<SpanEvent> = batch
        .iter()
        .filter_map(|msg_result| match msg_result {
            Ok(msg) => serde_json::from_value::<SpanEvent>(msg.body().clone()).ok(),
            Err(e) => {
                worker::console_error!("hb-tracing: failed to deserialize queue message: {:?}", e);
                None
            }
        })
        .collect();

    if !spans.is_empty() {
        handle_slice(&spans, dataset)?;
    }

    Ok(())
}
