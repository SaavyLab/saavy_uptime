use crate::{FieldRecorder, span_event::SpanEvent};
use std::sync::{Arc, Mutex};
use tracing::{Subscriber, span::{Attributes, Id}};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};
use worker::{Date, Result};

#[derive(Clone)]
pub struct BufferLayer {
    /// The buffer of SpanEvents.
    /// 
    /// Since workers are single-threaded this should never be contended.
    /// The tracing library requires us to satisfy Send + Sync
    buffer: Arc<Mutex<Vec<SpanEvent>>>
}


pub struct FlushGuard {
    buffer: Arc<Mutex<Vec<SpanEvent>>>
}

/// Creates a new tracing layer and a flush guard.
/// 
/// The layer collects spans into a thread-local buffer.
/// The guard must be used to flush these spans to a Queue at the end of the request.
pub fn buffer_layer() -> (BufferLayer, FlushGuard) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    (BufferLayer { buffer: buffer.clone() }, FlushGuard { buffer })
}

impl FlushGuard {
    /// Flushes buffered spans to the provided Cloudflare Queue.
    /// 
    /// You should call this inside `ctx.wait_until` to avoid delaying the response.
    pub async fn flush(self, queue: &worker::Queue) -> Result<()> {
        let events = {
            let mut buffer = match self.buffer.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    worker::console_error!("hb-tracing: buffer lock poisoned during flush: {:?}", e);
                    e.into_inner()
                }
            };
            std::mem::take(&mut *buffer)
        };

        if events.is_empty() {
            return Ok(());
        }

        queue.send_batch(&events).await?;
        Ok(())
    }
}

struct PendingSpan {
    start_time: f64,
    fields: FieldRecorder
}

impl<S> Layer<S> for BufferLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let pending = PendingSpan {
                start_time: Date::now().as_millis() as f64,
                fields: FieldRecorder::from_attributes(attrs),
            };
            span.extensions_mut().insert(pending);
        }
    }

    fn on_record(&self, id: &Id, record: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            if let Some(pending) = span.extensions_mut().get_mut::<PendingSpan>() {
                pending.fields.extend(record);
            }
        }
    }

    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        // We currently don't ship pure events to AE to save costs/bandwidth,
        // focusing on Spans. This is a design choice for "hb".
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let mut extensions = span.extensions_mut();
            let pending = extensions.remove::<PendingSpan>();
            drop(extensions);

            if let Some(pending) = pending {
                let end_time = Date::now();
                let duration_ms = (end_time.as_millis() as f64) - pending.start_time;

                let metadata = span.metadata();
                
                // Simple ID generation (in reality, you might want full 128-bit OTel IDs)
                // But for internal consistency, u64 hex is fine.
                let span_id = format!("{:x}", span.id().into_u64());
                
                // Find parent
                let parent_id = span
                    .parent()
                    .map(|p| format!("{:x}", p.id().into_u64()))
                    .unwrap_or_else(|| "root".to_string());

                // Find root (trace ID)
                // In simple tracing-subscriber, the root of the tree is the trace ID.
                // This assumes we aren't propagating distributed traces from INCOMING headers yet.
                // That is a future 'hb-tracing' feature.
                let root_id = span
                    .scope()
                    .from_root()
                    .next()
                    .map(|r| format!("{:x}", r.id().into_u64()))
                    .unwrap_or_else(|| span_id.clone());

                let event = SpanEvent {
                    trace_id: root_id,
                    span_id,
                    parent_id,
                    name: metadata.name().to_string(),
                    target: metadata.target().to_string(),
                    level: metadata.level().to_string(),
                    duration_ms,
                    start_time_ms: pending.start_time,
                    fields: pending.fields.to_json(),
                };

                match self.buffer.lock() {
                    Ok(mut guard) => guard.push(event),
                    Err(e) => {
                        worker::console_error!("hb-tracing: buffer lock poisoned during flush: {:?}", e);
                    }
                }
            }
        }
    }
}