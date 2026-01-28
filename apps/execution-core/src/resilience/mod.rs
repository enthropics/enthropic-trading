//! Resilience Module - Circuit Breakers, Retries, Bulkheads
//! Phase 3: Fault tolerance patterns for distributed trading systems

mod circuit_breaker;
mod retry;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
pub use retry::{RetryConfig, with_retry_async};

// Bulkhead is optional - only include if the file exists
#[cfg(feature = "bulkhead")]
mod bulkhead;
#[cfg(feature = "bulkhead")]
pub use bulkhead::Bulkhead;