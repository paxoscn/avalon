use std::time::Duration;
use tokio::time::sleep;
use crate::error::PlatformError;

/// Configuration for retry logic in vector operations
#[derive(Debug, Clone)]
pub struct VectorRetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for VectorRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry wrapper for vector operations
pub struct VectorRetryWrapper {
    config: VectorRetryConfig,
}

impl VectorRetryWrapper {
    pub fn new(config: VectorRetryConfig) -> Self {
        Self { config }
    }
    
    /// Execute an operation with retry logic
    pub async fn execute_with_retry<F, T, Fut>(&self, operation: F) -> Result<T, PlatformError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, PlatformError>>,
    {
        let mut delay = self.config.base_delay;
        let mut last_error_msg = None;
        
        for attempt in 1..=self.config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let error_msg = error.to_string();
                    last_error_msg = Some(error_msg.clone());
                    
                    if attempt == self.config.max_attempts {
                        break;
                    }
                    
                    if !self.is_retryable(&error) {
                        return Err(error);
                    }
                    
                    log::warn!(
                        "Vector operation attempt {} failed: {}, retrying in {:?}",
                        attempt, error_msg, delay
                    );
                    
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * self.config.backoff_multiplier) as u64),
                        self.config.max_delay,
                    );
                }
            }
        }
        
        Err(PlatformError::VectorStoreError(
            last_error_msg.unwrap_or_else(|| "Retry operation failed without error".to_string())
        ))
    }
    
    /// Check if an error is retryable
    fn is_retryable(&self, error: &PlatformError) -> bool {
        match error {
            PlatformError::VectorStoreError(msg) => {
                // Retry on network errors, timeouts, and rate limits
                msg.contains("timeout") ||
                msg.contains("network") ||
                msg.contains("connection") ||
                msg.contains("rate limit") ||
                msg.contains("503") ||
                msg.contains("502") ||
                msg.contains("504")
            },
            PlatformError::InternalError(_) => true,
            _ => false,
        }
    }
}

/// Circuit breaker for vector operations
pub struct VectorCircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    failure_count: std::sync::atomic::AtomicU32,
    last_failure_time: std::sync::Mutex<Option<std::time::Instant>>,
    state: std::sync::Mutex<CircuitBreakerState>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl VectorCircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure_time: std::sync::Mutex::new(None),
            state: std::sync::Mutex::new(CircuitBreakerState::Closed),
        }
    }
    
    /// Execute an operation with circuit breaker protection
    pub async fn execute<F, T, Fut>(&self, operation: F) -> Result<T, PlatformError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, PlatformError>>,
    {
        if !self.should_allow_request() {
            return Err(PlatformError::VectorStoreError(
                "Circuit breaker is open - vector store is temporarily unavailable".to_string()
            ));
        }
        
        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            },
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }
    
    fn should_allow_request(&self) -> bool {
        let state = self.state.lock().unwrap();
        
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        // Transition to half-open state
                        drop(state);
                        *self.state.lock().unwrap() = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    fn on_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
        *self.state.lock().unwrap() = CircuitBreakerState::Closed;
    }
    
    fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        *self.last_failure_time.lock().unwrap() = Some(std::time::Instant::now());
        
        if failure_count >= self.failure_threshold {
            *self.state.lock().unwrap() = CircuitBreakerState::Open;
        }
    }
}

/// Error mapper for converting provider-specific errors to platform errors
pub struct VectorErrorMapper;

impl VectorErrorMapper {
    /// Map HTTP status codes to platform errors
    pub fn map_http_error(status: u16, body: &str, provider: &str) -> PlatformError {
        match status {
            400 => PlatformError::ValidationError(
                format!("{} validation error: {}", provider, body)
            ),
            401 => PlatformError::AuthenticationFailed(
                format!("{} authentication failed: {}", provider, body)
            ),
            403 => PlatformError::AuthorizationFailed(
                format!("{} authorization failed: {}", provider, body)
            ),
            404 => PlatformError::NotFound(
                format!("{} resource not found: {}", provider, body)
            ),
            429 => PlatformError::VectorStoreError(
                format!("{} rate limit exceeded: {}", provider, body)
            ),
            500..=599 => PlatformError::VectorStoreError(
                format!("{} server error ({}): {}", provider, status, body)
            ),
            _ => PlatformError::VectorStoreError(
                format!("{} unexpected error ({}): {}", provider, status, body)
            ),
        }
    }
    
    /// Map network errors to platform errors
    pub fn map_network_error(error: &str, provider: &str) -> PlatformError {
        if error.contains("timeout") {
            PlatformError::VectorStoreError(
                format!("{} connection timeout: {}", provider, error)
            )
        } else if error.contains("connection") {
            PlatformError::VectorStoreError(
                format!("{} connection error: {}", provider, error)
            )
        } else {
            PlatformError::VectorStoreError(
                format!("{} network error: {}", provider, error)
            )
        }
    }
    
    /// Map serialization errors to platform errors
    pub fn map_serialization_error(error: &str, provider: &str) -> PlatformError {
        PlatformError::VectorStoreError(
            format!("{} serialization error: {}", provider, error)
        )
    }
    
    /// Map validation errors to platform errors
    pub fn map_validation_error(error: &str, provider: &str) -> PlatformError {
        PlatformError::ValidationError(
            format!("{} validation error: {}", provider, error)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_wrapper_success() {
        let retry_wrapper = VectorRetryWrapper::new(VectorRetryConfig::default());
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let result = retry_wrapper.execute_with_retry(|| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok::<i32, PlatformError>(42)
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_retry_wrapper_eventual_success() {
        let retry_wrapper = VectorRetryWrapper::new(VectorRetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        });
        
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let result = retry_wrapper.execute_with_retry(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if count < 3 {
                    Err(PlatformError::VectorStoreError("timeout".to_string()))
                } else {
                    Ok::<i32, PlatformError>(42)
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let circuit_breaker = VectorCircuitBreaker::new(2, Duration::from_millis(100));
        
        // First failure
        let result1 = circuit_breaker.execute(|| async {
            Err::<(), PlatformError>(PlatformError::VectorStoreError("error".to_string()))
        }).await;
        assert!(result1.is_err());
        
        // Second failure - should open circuit
        let result2 = circuit_breaker.execute(|| async {
            Err::<(), PlatformError>(PlatformError::VectorStoreError("error".to_string()))
        }).await;
        assert!(result2.is_err());
        
        // Third attempt - should be blocked by circuit breaker
        let result3 = circuit_breaker.execute(|| async {
            Ok::<(), PlatformError>(())
        }).await;
        assert!(result3.is_err());
        assert!(result3.unwrap_err().to_string().contains("Circuit breaker is open"));
    }
}