use crate::observability::visitor::FieldRecorder;
use js_sys::Date;
use tracing::{span::Attributes, Event, Id, Metadata, Subscriber};
use tracing_subscriber::{
    layer::{Context, Layer},
    registry::{LookupSpan, SpanRef},
};
use worker::{
    console_error, AnalyticsEngineDataPointBuilder, AnalyticsEngineDataset, Result as WorkerResult,
};

#[derive(Clone)]
pub struct AnalyticsEngineLayer {
    dataset: AnalyticsEngineDataset,
}

impl AnalyticsEngineLayer {
    pub fn new(dataset: AnalyticsEngineDataset) -> Self {
        Self { dataset }
    }
}

struct SpanMetrics {
    name: String,
    target: String,
    level: tracing::Level,
    started_at_ms: f64,
    fields: FieldRecorder,
}

impl SpanMetrics {
    fn new(attrs: &Attributes<'_>, metadata: &Metadata<'_>) -> Self {
        Self {
            name: metadata.name().to_string(),
            target: metadata.target().to_string(),
            level: *metadata.level(),
            started_at_ms: Date::now(),
            fields: FieldRecorder::from_attributes(attrs),
        }
    }

    fn record(&mut self, record: &tracing::span::Record<'_>) {
        self.fields.extend(record);
    }
}

impl<S> Layer<S> for AnalyticsEngineLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            span.extensions_mut()
                .insert(SpanMetrics::new(attrs, span.metadata()));
        }
    }

    fn on_record(&self, id: &Id, record: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            if let Some(metrics) = span.extensions_mut().get_mut::<SpanMetrics>() {
                metrics.record(record);
            }
        }
    }

    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {}

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let mut extensions = span.extensions_mut();
            let metrics = extensions.remove::<SpanMetrics>();
            drop(extensions);

            if let Some(metrics) = metrics {
                let duration_ms = Date::now() - metrics.started_at_ms;

                let root_id = span
                    .scope()
                    .from_root()
                    .next()
                    .map(|root| root.id().into_u64())
                    .unwrap_or_else(|| span.id().into_u64());

                let span_id = format!("{:x}", span.id().into_u64());
                let parent_id = span
                    .parent()
                    .map(|parent| format!("{:x}", parent.id().into_u64()))
                    .unwrap_or_else(|| "root".to_string());
                let trace_id = format!("{:x}", root_id);

                if let Err(err) = write_span(
                    &self.dataset,
                    &SpanWrite {
                        duration_ms,
                        span_id,
                        trace_id,
                        parent_id,
                        metrics,
                    },
                ) {
                    console_error!("observability.analytics.write: {err:?}");
                }
            }
        }
    }
}

struct SpanWrite {
    duration_ms: f64,
    span_id: String,
    trace_id: String,
    parent_id: String,
    metrics: SpanMetrics,
}

fn write_span(dataset: &AnalyticsEngineDataset, span: &SpanWrite) -> WorkerResult<()> {
    let fields_json = span.metrics.fields.to_json();
    let level = span.metrics.level.to_string();

    AnalyticsEngineDataPointBuilder::new()
        .indexes([span.trace_id.as_str()])
        .add_double(span.duration_ms)
        .add_blob(span.span_id.as_str())
        .add_blob(span.parent_id.as_str())
        .add_blob(span.metrics.name.as_str())
        .add_blob(span.metrics.target.as_str())
        .add_blob(fields_json.as_str())
        .add_blob(level.as_str())
        .write_to(dataset)
}
