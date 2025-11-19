use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: String,
    pub name: String,
    pub target: String,
    pub level: String,
    pub duration_ms: f64,
    pub start_time_ms: f64,
    pub fields: String, // Serialized JSON
}