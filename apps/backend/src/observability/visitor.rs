use std::fmt::Debug;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};

#[derive(Default)]
pub struct FieldRecorder {
    entries: Vec<(String, String)>,
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

    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<(String, String)> {
        self.entries
    }

    pub fn to_json(&self) -> String {
        let mut object = serde_json::Map::new();
        for (key, value) in &self.entries {
            object.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        serde_json::Value::Object(object).to_string()
    }

    pub fn format_pairs(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }

        self.entries
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn push_value(&mut self, field: &Field, value: String) {
        self.entries.push((field.name().to_string(), value));
    }
}

impl Visit for FieldRecorder {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.push_value(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.push_value(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.push_value(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.push_value(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.push_value(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.push_value(field, value.to_string());
    }
}
