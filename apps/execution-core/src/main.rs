//! Execution Core - High-Performance Trading Engine
//! Phase 1: Persistence | Phase 2: Authentication | Phase 3: Observability & Resilience

mod auth;
mod config;
mod engine;
mod nats_handler;
mod observability;
mod resilience;
mod proto;

use crate::auth::AuthService;
use crate::config::Config;
use crate::nats_handler::NatsSubscriber;
use crate::observability::health::{start_health_server, HealthState};
use crate::observability::metrics::get_metrics;
use crate::resilience::{CircuitBreaker, CircuitBreakerConfig, RetryConfig, with_retry_async};
use sqlx::postgres::PgPoolOptions;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize observability (tracing, metrics)
    observability::init_observability("execution-core")?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting Execution Core..."
    );

    // Connection status flags for health checks
    let nats_connected = Arc::new(AtomicBool::new(false));
    let redis_connected = Arc::new(AtomicBool::new(false));

    // Initialize database pool with retry
    let pool = with_retry_async(
        "database_connect",
        &RetryConfig::default(),
        || async {
            PgPoolOptions::new()
                .min_connections(config.pool_min_connections)
                .max_connections(config.pool_max_connections)
                .acquire_timeout(Duration::from_secs(5))
                .connect(&config.database_url)
                .await
        },
    ).await?;

    info!("Connected to PostgreSQL");

    // Update DB pool metrics
    if let Some(ref metrics) = *get_metrics() {
        metrics.db_pool_connections.with_label_values(&["active"]).set(0.0);
        metrics.db_pool_connections.with_label_values(&["idle"]).set(config.pool_min_connections as f64);
    }

    // Initialize Redis with retry
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let _redis_conn = with_retry_async(
        "redis_connect",
        &RetryConfig::default(),
        || async {
            redis::aio::ConnectionManager::new(redis_client.clone()).await
        },
    ).await?;
    redis_connected.store(true, Ordering::Relaxed);
    info!("Connected to Redis");

    // Initialize auth service
    let auth_service = Arc::new(AuthService::new(&config.jwt_secret));
    info!("Auth service initialized");

    // Circuit breaker for NATS (unused but prepared for resilience)
    let _nats_circuit_breaker = Arc::new(CircuitBreaker::new(
        CircuitBreakerConfig {
            name: "nats".to_string(),
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    ));

    // Connect to NATS with retry
    let nats_client = with_retry_async(
        "nats_connect",
        &RetryConfig::default(),
        || async {
            async_nats::connect(&config.nats_url).await
        },
    ).await?;
    nats_connected.store(true, Ordering::Relaxed);
    info!(url = %config.nats_url, "Connected to NATS");

    // Initialize NATS subscriber
    let subscriber = NatsSubscriber::new(
        nats_client,
        pool.clone(),
        auth_service,
    );

    // Load state from database
    subscriber.initialize().await?;
    info!("State loaded from database");

    // Start health/metrics server
    let health_state = HealthState {
        db_pool: pool.clone(),
        nats_connected: nats_connected.clone(),
        redis_connected: redis_connected.clone(),
        ready: Arc::new(AtomicBool::new(true)),
    };

    let metrics_port: u16 = std::env::var("METRICS_PORT")
        .unwrap_or_else(|_| "9100".to_string())
        .parse()
        .unwrap_or(9100);

    tokio::spawn(async move {
        if let Err(e) = start_health_server(metrics_port, health_state).await {
            error!(error = %e, "Health server failed");
        }
    });

    // Graceful shutdown handler
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Received shutdown signal");
        let _ = shutdown_tx.send(());
    });

    // Run subscriber
    tokio::select! {
        result = subscriber.run() => {
            if let Err(e) = result {
                error!(error = %e, "Subscriber error");
            }
        }
        _ = shutdown_rx.recv() => {
            info!("Shutting down...");
        }
    }

    // Graceful shutdown
    observability::shutdown_observability();
    info!("Execution Core stopped");
    Ok(())
}