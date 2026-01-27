//! Observability Module - Phase 3: Enterprise-Grade Observability
//! OpenTelemetry Tracing, Prometheus Metrics, Structured JSON Logging

pub mod metrics;
pub mod tracing_setup;
pub mod health;

use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the complete observability stack
pub fn init_observability(service_name: &str) -> anyhow::Result<()> {
    // Initialize tracing with OpenTelemetry
    let tracer = tracing_setup::init_tracer(service_name)?;
    
    // Initialize Prometheus metrics
    metrics::init_metrics()?;
    
    // Setup tracing subscriber with multiple layers
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    
    // Environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,execution_core=debug,tower_http=debug"));

    // JSON formatted logs for production
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .with(telemetry_layer)
        .init();

    tracing::info!(
        service.name = service_name,
        service.version = env!("CARGO_PKG_VERSION"),
        "Observability stack initialized"
    );

    Ok(())
}

/// Graceful shutdown of observability providers
pub fn shutdown_observability() {
    tracing::info!("Shutting down observability providers...");
    opentelemetry::global::shutdown_tracer_provider();
}
