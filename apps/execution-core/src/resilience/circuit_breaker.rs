//! Circuit Breaker Pattern
//! Prevents cascade failures by stopping calls to failing services

use crate::observability::metrics::get_metrics;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, instrument, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation - requests flow through
    Open,      // Failure detected - requests blocked
    HalfOpen,  // Testing recovery - limited requests allowed
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "closed"),
            CircuitState::Open => write!(f, "open"),
            CircuitState::HalfOpen => write!(f, "half_open"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open before closing
    pub success_threshold: u32,
    /// Time to wait before transitioning from open to half-open
    pub timeout: Duration,
    /// Maximum calls allowed in half-open state
    pub half_open_max_calls: u32,
    /// Name for metrics/logging
    pub name: String,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
            name: "default".to_string(),
        }
    }
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    half_open_calls: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,
    total_calls: AtomicU64,
    total_failures: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let breaker = Self {
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            half_open_calls: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            total_calls: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
        };
        breaker.update_metric(CircuitState::Closed);
        breaker
    }

    pub async fn state(&self) -> CircuitState {
        let mut state = self.state.write().await;
        
        if *state == CircuitState::Open {
            let last_failure = self.last_failure_time.read().await;
            if let Some(time) = *last_failure {
                if time.elapsed() >= self.config.timeout {
                    *state = CircuitState::HalfOpen;
                    self.half_open_calls.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                    self.update_metric(CircuitState::HalfOpen);
                    info!(
                        circuit_breaker = %self.config.name,
                        "Circuit breaker transitioning to half-open"
                    );
                }
            }
        }
        
        *state
    }

    /// Execute an async function with circuit breaker protection
    #[instrument(skip(self, f), fields(circuit_breaker = %self.config.name))]
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        let current_state = self.state().await;

        match current_state {
            CircuitState::Open => {
                warn!(
                    circuit_breaker = %self.config.name,
                    "Circuit breaker is open, rejecting call"
                );
                get_metrics().errors_total
                    .with_label_values(&["circuit_breaker_open", "execution-core"])
                    .inc();
                Err(CircuitBreakerError::Open)
            }
            CircuitState::HalfOpen => {
                let calls = self.half_open_calls.fetch_add(1, Ordering::SeqCst);
                if calls >= self.config.half_open_max_calls {
                    warn!(
                        circuit_breaker = %self.config.name,
                        "Half-open call limit reached"
                    );
                    return Err(CircuitBreakerError::Open);
                }
                self.execute(f).await
            }
            CircuitState::Closed => {
                self.execute(f).await
            }
        }
    }

    async fn execute<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        match f().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(CircuitBreakerError::ServiceError(e))
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        
        match *state {
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    self.reset_counts();
                    self.update_metric(CircuitState::Closed);
                    info!(
                        circuit_breaker = %self.config.name,
                        "Circuit breaker closed after recovery"
                    );
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        let mut state = self.state.write().await;
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        
        *self.last_failure_time.write().await = Some(Instant::now());

        match *state {
            CircuitState::Closed => {
                if failures >= self.config.failure_threshold {
                    *state = CircuitState::Open;
                    self.update_metric(CircuitState::Open);
                    error!(
                        circuit_breaker = %self.config.name,
                        failures = failures,
                        threshold = self.config.failure_threshold,
                        "Circuit breaker OPENED due to failures"
                    );
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                self.reset_counts();
                self.update_metric(CircuitState::Open);
                warn!(
                    circuit_breaker = %self.config.name,
                    "Circuit breaker re-opened after half-open failure"
                );
            }
            _ => {}
        }
    }

    fn reset_counts(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.half_open_calls.store(0, Ordering::SeqCst);
    }

    fn update_metric(&self, state: CircuitState) {
        let value = match state {
            CircuitState::Closed => 0.0,
            CircuitState::HalfOpen => 0.5,
            CircuitState::Open => 1.0,
        };
        get_metrics()
            .circuit_breaker_state
            .with_label_values(&[&self.config.name])
            .set(value);
    }

    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            total_calls: self.total_calls.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
            current_failures: self.failure_count.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreakerStats {
    pub total_calls: u64,
    pub total_failures: u64,
    pub current_failures: u32,
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    ServiceError(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Open => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::ServiceError(e) => write!(f, "Service error: {}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::ServiceError(e) => Some(e),
            _ => None,
        }
    }
}
