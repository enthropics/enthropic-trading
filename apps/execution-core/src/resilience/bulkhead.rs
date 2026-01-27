//! Bulkhead Pattern
//! Isolates failures by limiting concurrent operations

use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{warn, instrument};

#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent operations
    pub max_concurrent: usize,
    /// Name for metrics/logging
    pub name: String,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 100,
            name: "default".to_string(),
        }
    }
}

pub struct Bulkhead {
    semaphore: Arc<Semaphore>,
    config: BulkheadConfig,
}

impl Bulkhead {
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
        }
    }

    /// Execute with bulkhead protection
    #[instrument(skip(self, f), fields(bulkhead = %self.config.name))]
    pub async fn execute<F, Fut, T>(&self, f: F) -> Result<T, BulkheadError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let permit = self.semaphore.try_acquire().map_err(|_| {
            warn!(
                bulkhead = %self.config.name,
                max_concurrent = self.config.max_concurrent,
                "Bulkhead full, rejecting request"
            );
            BulkheadError::Full
        })?;

        let result = f().await;
        drop(permit);
        Ok(result)
    }

    /// Execute with timeout
    pub async fn execute_with_timeout<F, Fut, T>(
        &self,
        f: F,
        timeout: std::time::Duration,
    ) -> Result<T, BulkheadError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let permit = tokio::time::timeout(timeout, self.semaphore.acquire())
            .await
            .map_err(|_| BulkheadError::Timeout)?
            .map_err(|_| BulkheadError::Closed)?;

        let result = f().await;
        drop(permit);
        Ok(result)
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[derive(Debug)]
pub enum BulkheadError {
    Full,
    Timeout,
    Closed,
}

impl std::fmt::Display for BulkheadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkheadError::Full => write!(f, "Bulkhead is full"),
            BulkheadError::Timeout => write!(f, "Bulkhead timeout"),
            BulkheadError::Closed => write!(f, "Bulkhead is closed"),
        }
    }
}

impl std::error::Error for BulkheadError {}
