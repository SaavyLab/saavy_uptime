pub mod console_layer;
pub mod layer;
pub mod visitor;
pub mod processor;
pub mod span_event;

// Re-export for convenience
pub use processor::{handle_batch, handle_json_batch};
pub use layer::{buffer_layer, FlushGuard, BufferLayer};
pub use span_event::SpanEvent;
pub use console_layer::ConsoleLayer;
pub use visitor::FieldRecorder;