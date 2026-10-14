use tracing::subscriber::set_global_default;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

/// Initializes the logging system for the application.
pub fn init() {
    let env_filter = EnvFilter::from_default_env()
        .add_directive("axum=error".parse().unwrap())
        .add_directive("tower_http=error".parse().unwrap())
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("h2=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    let formatting_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .with_span_events(fmt::format::FmtSpan::NONE)
        .with_timer(LocalTime::rfc_3339());

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set global subscriber");
}