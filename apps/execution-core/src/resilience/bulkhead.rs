//! Retry with Exponential Backoff
//! Handles transient failures with configurable retry policies

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Execute an async function with retry logic
pub async fn with_retry_async<F, Fut, T, E>(
    operation: &str,
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;

        match f().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= config.max_attempts {
                    warn!(
                        operation = operation,
                        attempt = attempt,
                        error = %e,
                        "Operation failed after all retries"
                    );
                    return Err(e);
                }

                warn!(
                    operation = operation,
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    error = %e,
                    delay_ms = delay.as_millis(),
                    "Operation failed, retrying"
                );

                // Add jitter if configured
                let actual_delay = if config.jitter {
                    let jitter = (rand_jitter() * delay.as_millis() as f64 * 0.3) as u64;
                    Duration::from_millis(delay.as_millis() as u64 + jitter)
                } else {
                    delay
                };

                sleep(actual_delay).await;

                // Calculate next delay with exponential backoff
                delay = Duration::from_millis(
                    (delay.as_millis() as f64 * config.multiplier) as u64
                );
                if delay > config.max_delay {
                    delay = config.max_delay;
                }
            }
        }
    }
}

/// Simple pseudo-random jitter (deterministic for reproducibility)
fn rand_jitter() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64 % 1000.0) / 1000.0
}