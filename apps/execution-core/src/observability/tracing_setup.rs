//! OpenTelemetry Tracing Configuration

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry::KeyValue;
use std::env;

/// Initialize OpenTelemetry tracer with OTLP exporter
pub fn init_tracer(service_name: &str) -> anyhow::Result<opentelemetry_sdk::trace::Tracer> {
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    // Configure trace sampling - 10% in production, 100% in development
    let sampler = if environment == "production" {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(0.1)))
    } else {
        Sampler::AlwaysOn
    };

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(
            trace::Config::default()
                .with_sampler(sampler)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(32)
                .with_max_links_per_span(32)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    KeyValue::new("service.namespace", "enthropic-trading"),
                    KeyValue::new("deployment.environment", environment),
                ])),
        )
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}