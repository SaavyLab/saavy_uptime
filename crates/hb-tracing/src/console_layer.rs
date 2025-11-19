use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{layer::Context, Layer};
use worker::{console_error, console_log};

use crate::visitor::FieldRecorder;

#[derive(Debug, Default, Clone)]
pub struct ConsoleLayer;

impl<S> Layer<S> for ConsoleLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut recorder = FieldRecorder::default();
        event.record(&mut recorder);
        let pairs = recorder.format_pairs();
        let metadata = event.metadata();
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
