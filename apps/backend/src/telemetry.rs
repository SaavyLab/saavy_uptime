use std::collections::HashMap;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{layer::Context, Layer};
use worker::{console_error, console_log};

/// Minimal tracing layer that prints events to the Workers console so they show up in Cloudflare logs.
#[derive(Debug, Default, Clone)]
pub struct ConsoleLayer;

impl<S> Layer<S> for ConsoleLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut recorder = FieldRecorder::default();
        event.record(&mut recorder);

        let metadata = event.metadata();
        let pairs = recorder.format_pairs();
        let message = if pairs.is_empty() {
            format!("[{}] {}", metadata.level(), metadata.target())
        } else {
            format!("[{}] {} {pairs}", metadata.level(), metadata.target())
        };

        match *metadata.level() {
            Level::ERROR | Level::WARN => console_error!("{:?}", message),
            _ => console_log!("{:?}", message),
        }
    }
}

#[derive(Debug, Default)]
struct FieldRecorder {
    fields: HashMap<String, serde_json::Value>,
}

impl FieldRecorder {
    fn format_pairs(&self) -> String {
        self.fields
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl tracing::field::Visit for FieldRecorder {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::String(format!("{value:?}")),
        );
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Number::from_f64(value)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
        );
    }
}
