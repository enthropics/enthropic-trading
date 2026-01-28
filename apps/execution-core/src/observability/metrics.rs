//! Prometheus Metrics for Trading Platform
//! Custom metrics for order processing, positions, and system health

use once_cell::sync::Lazy;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, HistogramVec,
    Opts, Registry, TextEncoder, Encoder,
};
use std::sync::Mutex;

/// Global metrics registry
static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// Application metrics
pub struct Metrics {
    pub orders_processed_total: CounterVec,
    pub orders_rejected_total: CounterVec,
    pub order_processing_duration: HistogramVec,
    pub position_updates_total: Counter,
    pub active_positions: Gauge,
    pub position_pnl: GaugeVec,
    pub db_pool_connections: GaugeVec,
    pub nats_messages_received: CounterVec,
    pub nats_messages_published: CounterVec,
    pub circuit_breaker_state: GaugeVec,
    pub retry_attempts_total: CounterVec,
}

static METRICS: Lazy<Mutex<Option<Metrics>>> = Lazy::new(|| Mutex::new(None));

/// Initialize metrics
pub fn init_metrics(service_name: &str) -> anyhow::Result<()> {
    let orders_processed_total = CounterVec::new(
        Opts::new("enthropic_orders_processed_total", "Total orders processed")
            .namespace("enthropic")
            .const_label("service", service_name),
        &["status", "side", "symbol"]
    )?;

    let orders_rejected_total = CounterVec::new(
        Opts::new("enthropic_orders_rejected_total", "Total orders rejected")
            .namespace("enthropic")
            .const_label("service", service_name),
        &["reason"]
    )?;

    let order_processing_duration = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "enthropic_order_processing_duration_seconds",
            "Order processing latency in seconds"
        )
            .namespace("enthropic")
            .const_label("service", service_name)
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
        &["operation"]
    )?;

    let position_updates_total = Counter::new(
        "enthropic_position_updates_total",
        "Total position updates"
    )?;

    let active_positions = Gauge::new(
        "enthropic_active_positions",
        "Number of active positions"
    )?;

    let position_pnl = GaugeVec::new(
        Opts::new("enthropic_position_pnl", "Position PnL by type"),
        &["type"] // realized, unrealized
    )?;

    let db_pool_connections = GaugeVec::new(
        Opts::new("enthropic_db_pool_connections", "Database pool connections"),
        &["state"] // active, idle
    )?;

    let nats_messages_received = CounterVec::new(
        Opts::new("enthropic_nats_messages_received_total", "NATS messages received"),
        &["subject"]
    )?;

    let nats_messages_published = CounterVec::new(
        Opts::new("enthropic_nats_messages_published_total", "NATS messages published"),
        &["subject"]
    )?;

    let circuit_breaker_state = GaugeVec::new(
        Opts::new("enthropic_circuit_breaker_state", "Circuit breaker state (0=closed, 0.5=half-open, 1=open)"),
        &["name"]
    )?;

    let retry_attempts_total = CounterVec::new(
        Opts::new("enthropic_retry_attempts_total", "Total retry attempts"),
        &["operation", "outcome"]
    )?;

    // Register all metrics
    REGISTRY.register(Box::new(orders_processed_total.clone()))?;
    REGISTRY.register(Box::new(orders_rejected_total.clone()))?;
    REGISTRY.register(Box::new(order_processing_duration.clone()))?;
    REGISTRY.register(Box::new(position_updates_total.clone()))?;
    REGISTRY.register(Box::new(active_positions.clone()))?;
    REGISTRY.register(Box::new(position_pnl.clone()))?;
    REGISTRY.register(Box::new(db_pool_connections.clone()))?;
    REGISTRY.register(Box::new(nats_messages_received.clone()))?;
    REGISTRY.register(Box::new(nats_messages_published.clone()))?;
    REGISTRY.register(Box::new(circuit_breaker_state.clone()))?;
    REGISTRY.register(Box::new(retry_attempts_total.clone()))?;

    let metrics = Metrics {
        orders_processed_total,
        orders_rejected_total,
        order_processing_duration,
        position_updates_total,
        active_positions,
        position_pnl,
        db_pool_connections,
        nats_messages_received,
        nats_messages_published,
        circuit_breaker_state,
        retry_attempts_total,
    };

    let mut guard = METRICS.lock().unwrap();
    *guard = Some(metrics);

    tracing::info!("Prometheus metrics initialized");
    Ok(())
}

/// Get metrics instance
pub fn get_metrics() -> std::sync::MutexGuard<'static, Option<Metrics>> {
    METRICS.lock().unwrap()
}

/// Encode metrics to Prometheus text format
pub fn encode_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap_or_default();
    String::from_utf8(buffer).unwrap_or_default()
}