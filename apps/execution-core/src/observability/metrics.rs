//! Prometheus Metrics for Execution Core
//! Trading-specific metrics: orders, positions, latency, circuit breakers

use prometheus::{
    self, Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    Opts, Registry, TextEncoder, Encoder,
};
use std::sync::OnceLock;

static REGISTRY: OnceLock<Registry> = OnceLock::new();
static METRICS: OnceLock<TradingMetrics> = OnceLock::new();

/// Trading-specific metrics collection
#[derive(Clone)]
pub struct TradingMetrics {
    // Order metrics
    pub orders_processed: CounterVec,
    pub orders_rejected: CounterVec,
    pub order_processing_duration: HistogramVec,
    pub order_queue_depth: Gauge,
    
    // Position metrics
    pub position_updates: Counter,
    pub active_positions: Gauge,
    pub position_pnl: GaugeVec,
    
    // Fill metrics
    pub fills_total: CounterVec,
    pub fill_rate: GaugeVec,
    
    // System metrics
    pub db_pool_connections: GaugeVec,
    pub db_query_duration: HistogramVec,
    
    // NATS metrics
    pub nats_messages_received: CounterVec,
    pub nats_messages_published: CounterVec,
    pub nats_message_processing_duration: HistogramVec,
    
    // Resilience metrics
    pub circuit_breaker_state: GaugeVec,
    pub retry_attempts: CounterVec,
    pub errors_total: CounterVec,
    
    // Auth metrics
    pub auth_validations: CounterVec,
}

impl TradingMetrics {
    fn new(registry: &Registry) -> Self {
        // Order metrics
        let orders_processed = CounterVec::new(
            Opts::new("orders_processed_total", "Total orders processed"),
            &["status", "side", "order_type", "symbol"],
        ).unwrap();
        registry.register(Box::new(orders_processed.clone())).unwrap();

        let orders_rejected = CounterVec::new(
            Opts::new("orders_rejected_total", "Total orders rejected"),
            &["reason"],
        ).unwrap();
        registry.register(Box::new(orders_rejected.clone())).unwrap();

        let order_processing_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "order_processing_duration_seconds",
                "Order processing latency in seconds",
            ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["operation"],
        ).unwrap();
        registry.register(Box::new(order_processing_duration.clone())).unwrap();

        let order_queue_depth = Gauge::new("order_queue_depth", "Current order queue depth").unwrap();
        registry.register(Box::new(order_queue_depth.clone())).unwrap();

        // Position metrics
        let position_updates = Counter::new("position_updates_total", "Total position updates").unwrap();
        registry.register(Box::new(position_updates.clone())).unwrap();

        let active_positions = Gauge::new("active_positions_total", "Number of active positions").unwrap();
        registry.register(Box::new(active_positions.clone())).unwrap();

        let position_pnl = GaugeVec::new(
            Opts::new("position_pnl", "Position P&L"),
            &["account_id", "symbol", "pnl_type"],
        ).unwrap();
        registry.register(Box::new(position_pnl.clone())).unwrap();

        // Fill metrics
        let fills_total = CounterVec::new(
            Opts::new("fills_total", "Total fills executed"),
            &["symbol", "side"],
        ).unwrap();
        registry.register(Box::new(fills_total.clone())).unwrap();

        let fill_rate = GaugeVec::new(
            Opts::new("fill_rate", "Fill rate percentage"),
            &["symbol"],
        ).unwrap();
        registry.register(Box::new(fill_rate.clone())).unwrap();

        // System metrics
        let db_pool_connections = GaugeVec::new(
            Opts::new("db_pool_connections", "Database pool connections"),
            &["state"],
        ).unwrap();
        registry.register(Box::new(db_pool_connections.clone())).unwrap();

        let db_query_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "db_query_duration_seconds",
                "Database query duration",
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]),
            &["query_type"],
        ).unwrap();
        registry.register(Box::new(db_query_duration.clone())).unwrap();

        // NATS metrics
        let nats_messages_received = CounterVec::new(
            Opts::new("nats_messages_received_total", "NATS messages received"),
            &["subject"],
        ).unwrap();
        registry.register(Box::new(nats_messages_received.clone())).unwrap();

        let nats_messages_published = CounterVec::new(
            Opts::new("nats_messages_published_total", "NATS messages published"),
            &["subject"],
        ).unwrap();
        registry.register(Box::new(nats_messages_published.clone())).unwrap();

        let nats_message_processing_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "nats_message_processing_duration_seconds",
                "NATS message processing duration",
            ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
            &["subject"],
        ).unwrap();
        registry.register(Box::new(nats_message_processing_duration.clone())).unwrap();

        // Resilience metrics
        let circuit_breaker_state = GaugeVec::new(
            Opts::new("circuit_breaker_state", "Circuit breaker state (0=closed, 0.5=half-open, 1=open)"),
            &["name"],
        ).unwrap();
        registry.register(Box::new(circuit_breaker_state.clone())).unwrap();

        let retry_attempts = CounterVec::new(
            Opts::new("retry_attempts_total", "Retry attempts"),
            &["operation", "outcome"],
        ).unwrap();
        registry.register(Box::new(retry_attempts.clone())).unwrap();

        let errors_total = CounterVec::new(
            Opts::new("errors_total", "Total errors"),
            &["type", "service"],
        ).unwrap();
        registry.register(Box::new(errors_total.clone())).unwrap();

        // Auth metrics
        let auth_validations = CounterVec::new(
            Opts::new("auth_validations_total", "Auth validation attempts"),
            &["result"],
        ).unwrap();
        registry.register(Box::new(auth_validations.clone())).unwrap();

        Self {
            orders_processed,
            orders_rejected,
            order_processing_duration,
            order_queue_depth,
            position_updates,
            active_positions,
            position_pnl,
            fills_total,
            fill_rate,
            db_pool_connections,
            db_query_duration,
            nats_messages_received,
            nats_messages_published,
            nats_message_processing_duration,
            circuit_breaker_state,
            retry_attempts,
            errors_total,
            auth_validations,
        }
    }
}

/// Initialize metrics system
pub fn init_metrics() -> anyhow::Result<()> {
    let registry = Registry::new();
    let metrics = TradingMetrics::new(&registry);
    
    REGISTRY.set(registry).ok();
    METRICS.set(metrics).ok();
    
    Ok(())
}

/// Get metrics instance
pub fn get_metrics() -> &'static TradingMetrics {
    METRICS.get().expect("Metrics not initialized")
}

/// Get registry
pub fn get_registry() -> &'static Registry {
    REGISTRY.get().expect("Registry not initialized")
}

/// Encode metrics for Prometheus scraping
pub fn encode_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = get_registry().gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
