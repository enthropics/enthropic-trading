//! Retry with Exponential Backoff
//! Handles transient failures in database/NATS connections

use crate::observability::metrics::get_metrics;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, instrument};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Quick retries for low-latency operations
    pub fn fast() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Slow retries for external services
    pub fn slow() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Database connection retries
    pub fn database() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Execute an async function with retry and exponential backoff
#[instrument(skip(f), fields(operation = %operation))]
pub async fn with_retry_async<F, Fut, T, E>(
    operation: &str,
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        
        match f().await {
            Ok(result) => {
                if attempt > 1 {
                    get_metrics()
                        .retry_attempts
                        .with_label_values(&[operation, "success"])
                        .inc();
                    info!(
                        operation = %operation,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                get_metrics()
                    .retry_attempts
                    .with_label_values(&[operation, "failure"])
                    .inc();

                if attempt >= config.max_retries {
                    warn!(
                        operation = %operation,
                        attempt = attempt,
                        max_retries = config.max_retries,
                        error = %e,
                        "Operation failed after max retries"
                    );
                    return Err(e);
                }

                let actual_delay = if config.jitter {
                    add_jitter(delay)
                } else {
                    delay
                };

                warn!(
                    operation = %operation,
                    attempt = attempt,
                    error = %e,
                    delay_ms = actual_delay.as_millis(),
                    "Operation failed, retrying..."
                );

                sleep(actual_delay).await;
                
                // Calculate next delay with exponential backoff
                delay = Duration::from_millis(
                    ((delay.as_millis() as f64 * config.multiplier) as u64)
                        .min(config.max_delay.as_millis() as u64)
                );
            }
        }
    }
}

/// Synchronous retry (mainly for initialization)
pub fn with_retry<F, T, E>(
    operation: &str,
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        
        match f() {
            Ok(result) => {
                if attempt > 1 {
                    tracing::info!(
                        operation = %operation,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= config.max_retries {
                    tracing::warn!(
                        operation = %operation,
                        attempt = attempt,
                        error = %e,
                        "Operation failed after max retries"
                    );
                    return Err(e);
                }

                tracing::warn!(
                    operation = %operation,
                    attempt = attempt,
                    error = %e,
                    delay_ms = delay.as_millis(),
                    "Operation failed, retrying..."
                );

                std::thread::sleep(delay);
                delay = Duration::from_millis(
                    ((delay.as_millis() as f64 * config.multiplier) as u64)
                        .min(config.max_delay.as_millis() as u64)
                );
            }
        }
    }
}

fn add_jitter(delay: Duration) -> Duration {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64;
    let jitter_percent = (nanos % 30) as f64 / 100.0; // 0-30% jitter
    let jitter = (delay.as_millis() as f64 * jitter_percent) as u64;
    delay + Duration::from_millis(jitter)
}
