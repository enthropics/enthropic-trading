//! Observability Module - OpenTelemetry Tracing, Metrics, Structured Logging
//! Phase 3: Enterprise-grade observability for trading systems

pub mod metrics;
pub mod tracing_setup;
pub mod health;

use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize complete observability stack
pub fn init_observability(service_name: &str) -> anyhow::Result<()> {
    // Initialize tracing with OTLP exporter
    let tracer = tracing_setup::init_tracer(service_name)?;

    // Initialize metrics
    metrics::init_metrics(service_name)?;

    // Setup tracing subscriber with JSON formatting
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,execution_core=debug"));

    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .with(telemetry_layer)
        .init();

    tracing::info!(
        service = service_name,
        "Observability stack initialized"
    );

    Ok(())
}

/// Graceful shutdown of observability
pub fn shutdown_observability() {
    tracing::info!("Shutting down observability...");
    global::shutdown_tracer_provider();
}