//! OpenTelemetry Tracing Configuration
//! Production-grade distributed tracing with OTLP export

use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::env;

/// Initialize OpenTelemetry tracer with OTLP exporter
pub fn init_tracer(service_name: &str) -> anyhow::Result<sdktrace::Tracer> {
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let environment = env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string());

    // Configure trace sampling - 10% in production, 100% in development
    let sampler = if environment == "production" {
        sdktrace::Sampler::ParentBased(Box::new(sdktrace::Sampler::TraceIdRatioBased(0.1)))
    } else {
        sdktrace::Sampler::AlwaysOn
    };

    // Build OTLP exporter with endpoint
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&otlp_endpoint);

    // Build tracer - install_batch returns Tracer directly
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::Config::default()
                .with_sampler(sampler)
                .with_id_generator(sdktrace::RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(32)
                .with_max_links_per_span(32)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    KeyValue::new("service.namespace", "enthropic-trading"),
                    KeyValue::new("deployment.environment", environment),
                ]))
        )
        .install_batch(runtime::Tokio)?;

    tracing::info!(
        service = service_name,
        otlp_endpoint = %otlp_endpoint,
        "OpenTelemetry tracer initialized with OTLP export"
    );

    Ok(tracer)
}