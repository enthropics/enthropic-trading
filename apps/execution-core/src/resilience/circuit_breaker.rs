//! Circuit Breaker Implementation
//! Prevents cascading failures by failing fast when a service is unhealthy

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub name: String,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitBreakerState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,
    half_open_calls: AtomicU32,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitBreakerState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            half_open_calls: AtomicU32::new(0),
        }
    }

    pub async fn state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }

    /// Check if circuit allows a call
    pub async fn allow_call(&self) -> bool {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has passed
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                let now = Instant::now().elapsed().as_secs();

                if now - last_failure >= self.config.timeout.as_secs() {
                    // Transition to half-open
                    let mut state = self.state.write().await;
                    *state = CircuitBreakerState::HalfOpen;
                    self.half_open_calls.store(0, Ordering::Relaxed);
                    info!(name = %self.config.name, "Circuit breaker transitioning to half-open");
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                let calls = self.half_open_calls.fetch_add(1, Ordering::Relaxed);
                calls < self.config.half_open_max_calls
            }
        }
    }

    /// Record a successful call
    pub async fn record_success(&self) {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitBreakerState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitBreakerState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

                if successes >= self.config.success_threshold {
                    let mut state = self.state.write().await;
                    *state = CircuitBreakerState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    info!(name = %self.config.name, "Circuit breaker closed after recovery");
                }
            }
            CircuitBreakerState::Open => {}
        }
    }

    /// Record a failed call
    pub async fn record_failure(&self) {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitBreakerState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

                if failures >= self.config.failure_threshold {
                    let mut state = self.state.write().await;
                    *state = CircuitBreakerState::Open;
                    self.last_failure_time.store(
                        Instant::now().elapsed().as_secs(),
                        Ordering::Relaxed
                    );
                    warn!(
                        name = %self.config.name,
                        failures = failures,
                        "Circuit breaker opened"
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                let mut state = self.state.write().await;
                *state = CircuitBreakerState::Open;
                self.last_failure_time.store(
                    Instant::now().elapsed().as_secs(),
                    Ordering::Relaxed
                );
                self.success_count.store(0, Ordering::Relaxed);
                warn!(name = %self.config.name, "Circuit breaker re-opened from half-open");
            }
            CircuitBreakerState::Open => {}
        }
    }
}