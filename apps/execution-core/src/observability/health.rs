//! Health Check & Metrics HTTP Server
//! Provides /health, /health/live, /health/ready, /metrics endpoints

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::TcpListener;
use tracing::{info, instrument};

use super::metrics::encode_metrics;

#[derive(Clone)]
pub struct HealthState {
    pub db_pool: PgPool,
    pub nats_connected: Arc<AtomicBool>,
    pub redis_connected: Arc<AtomicBool>,
    pub ready: Arc<AtomicBool>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: ComponentHealth,
    nats: ComponentHealth,
    redis: ComponentHealth,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Start the health check and metrics HTTP server
#[instrument(skip(state))]
pub async fn start_health_server(port: u16, state: HealthState) -> anyhow::Result<()> {
    START_TIME.get_or_init(std::time::Instant::now);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/metrics", get(prometheus_metrics))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!(port = port, "Health/metrics server started");

    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument(skip(state))]
async fn health_check(State(state): State<HealthState>) -> impl IntoResponse {
    // Check database
    let db_health = match check_database(&state.db_pool).await {
        Ok(latency) => ComponentHealth {
            status: "healthy".to_string(),
            latency_ms: Some(latency),
            error: None,
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            latency_ms: None,
            error: Some(e.to_string()),
        },
    };

    // Check NATS
    let nats_health = if state.nats_connected.load(Ordering::Relaxed) {
        ComponentHealth {
            status: "healthy".to_string(),
            latency_ms: None,
            error: None,
        }
    } else {
        ComponentHealth {
            status: "unhealthy".to_string(),
            latency_ms: None,
            error: Some("NATS not connected".to_string()),
        }
    };

    // Check Redis
    let redis_health = if state.redis_connected.load(Ordering::Relaxed) {
        ComponentHealth {
            status: "healthy".to_string(),
            latency_ms: None,
            error: None,
        }
    } else {
        ComponentHealth {
            status: "unhealthy".to_string(),
            latency_ms: None,
            error: Some("Redis not connected".to_string()),
        }
    };

    let overall_healthy = db_health.status == "healthy"
        && nats_health.status == "healthy"
        && redis_health.status == "healthy";

    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);

    let response = HealthResponse {
        status: if overall_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        checks: HealthChecks {
            database: db_health,
            nats: nats_health,
            redis: redis_health,
        },
    };

    let status_code = if overall_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

async fn check_database(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let start = std::time::Instant::now();
    // Use sqlx::query_as with explicit type to avoid type inference issues
    let _row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(start.elapsed().as_millis() as u64)
}

async fn liveness() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "alive" })))
}

#[instrument(skip(state))]
async fn readiness(State(state): State<HealthState>) -> impl IntoResponse {
    if !state.ready.load(Ordering::Relaxed) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not_ready", "reason": "initializing" })),
        );
    }

    // Check database with explicit type
    let db_result: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT 1")
        .fetch_one(&state.db_pool)
        .await;
    let db_ok = db_result.is_ok();
    let nats_ok = state.nats_connected.load(Ordering::Relaxed);
    let redis_ok = state.redis_connected.load(Ordering::Relaxed);

    if db_ok && nats_ok && redis_ok {
        (StatusCode::OK, Json(serde_json::json!({ "status": "ready" })))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "status": "not_ready",
            "database": db_ok,
            "nats": nats_ok,
            "redis": redis_ok
        })))
    }
}

async fn prometheus_metrics() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        encode_metrics(),
    )
}