use std::collections::HashMap;
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};

#[derive(Debug, Default)]
pub struct FieldRecorder {
    fields: HashMap<String, serde_json::Value>,
}

impl FieldRecorder {
    pub fn from_attributes(attrs: &Attributes<'_>) -> Self {
        let mut recorder = Self::default();
        attrs.record(&mut recorder);
        recorder
    }

    pub fn extend(&mut self, record: &Record<'_>) {
        record.record(self);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.fields).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn format_pairs(&self) -> String {
        self.fields
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Visit for FieldRecorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::String(format!("{value:?}")),
        );
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Bool(value),
        );
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Number::from_f64(value)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        );
    }
}