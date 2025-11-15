mod analytics_layer;
mod console_layer;
mod visitor;

use once_cell::sync::OnceCell;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use worker::{console_error, console_log, Env};

pub use analytics_layer::AnalyticsEngineLayer;
pub use console_layer::ConsoleLayer;

static INITIALIZED: OnceCell<()> = OnceCell::new();

pub fn init(env: &Env) {
    INITIALIZED.get_or_init(|| {
        let console_layer = ConsoleLayer::default();
        let registry = Registry::default().with(console_layer);

        let subscriber = match env.analytics_engine("AE") {
            Ok(dataset) => {
                console_log!("observability: analytics engine enabled");
                registry.with(AnalyticsEngineLayer::new(dataset))
            }
            Err(err) => {
                console_error!("observability.analytics: {err:?}");
                registry
            }
        };

        if tracing::subscriber::set_global_default(subscriber).is_err() {
            console_error!("observability: tracing subscriber already initialized");
        }
    });
}
