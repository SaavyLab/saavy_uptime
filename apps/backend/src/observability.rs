mod analytics_layer;
mod console_layer;
mod visitor;

use once_cell::sync::OnceCell;
use tracing::Subscriber;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use worker::{console_error, console_log, Env};

pub use analytics_layer::AnalyticsEngineLayer;
pub use console_layer::ConsoleLayer;

static INITIALIZED: OnceCell<()> = OnceCell::new();

pub fn init(env: &Env) {
    INITIALIZED.get_or_init(|| match env.analytics_engine("AE_TRACES") {
        Ok(dataset) => {
            console_log!("observability: analytics engine enabled");
            let subscriber = Registry::default()
                .with(ConsoleLayer::default())
                .with(AnalyticsEngineLayer::new(dataset));
            install_subscriber(subscriber);
        }
        Err(err) => {
            console_error!("observability.analytics: {err:?}");
            let subscriber = Registry::default().with(ConsoleLayer::default());
            install_subscriber(subscriber);
        }
    });
}

fn install_subscriber<S>(subscriber: S)
where
    S: Subscriber + Send + Sync + 'static,
{
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        console_error!("observability: tracing subscriber already initialized");
    }
}
