use crate::domain::services::llm_service::LLMError;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration for LLM operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<RetryableErrorType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetryableErrorType {
    RateLimit,
    NetworkError,
    InternalServerError,
    Timeout,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retryable_errors: vec![
                RetryableErrorType::RateLimit,
                RetryableErrorType::NetworkError,
                RetryableErrorType::InternalServerError,
                RetryableErrorType::Timeout,
            ],
        }
    }
}

/// Retry wrapper for LLM operations
pub struct RetryWrapper {
    config: RetryConfig,
}

impl RetryWrapper {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn execute_with_retry<F, T, Fut>(&self, operation: F) -> Result<T, LLMError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, LLMError>>,
    {
        let mut delay = self.config.base_delay;
        let mut last_error = None;

        for attempt in 1..=self.config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error.clone());
                    
                    if attempt == self.config.max_attempts || !self.is_retryable(&error) {
                        break;
                    }

                    log::warn!(
                        "LLM operation failed on attempt {}/{}: {}. Retrying in {:?}",
                        attempt,
                        self.config.max_attempts,
                        error,
                        delay
                    );

                    sleep(delay).await;
                    
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * self.config.backoff_multiplier) as u64),
                        self.config.max_delay,
                    );
                }
            }
        }

        Err(last_error.unwrap_or_else(|| LLMError::InternalError("Unknown error during retry".to_string())))
    }

    fn is_retryable(&self, error: &LLMError) -> bool {
        let error_type = match error {
            LLMError::RateLimitExceeded(_) => RetryableErrorType::RateLimit,
            LLMError::NetworkError(_) => RetryableErrorType::NetworkError,
            LLMError::InternalError(_) => RetryableErrorType::InternalServerError,
            _ => return false,
        };

        self.config.retryable_errors.contains(&error_type)
    }
}

/// Circuit breaker for LLM operations
pub struct CircuitBreaker {
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

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure_time: std::sync::Mutex::new(None),
            state: std::sync::Mutex::new(CircuitBreakerState::Closed),
        }
    }

    pub async fn execute<F, T, Fut>(&self, operation: F) -> Result<T, LLMError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, LLMError>>,
    {
        // Check if circuit breaker should allow the operation
        if !self.should_allow_request() {
            return Err(LLMError::InternalError("Circuit breaker is open".to_string()));
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
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
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() > self.recovery_timeout {
                        drop(state);
                        *self.state.lock().unwrap() = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
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

/// Error mapper for converting HTTP errors to LLM errors
pub struct ErrorMapper;

impl ErrorMapper {
    pub fn map_http_error(status: u16, body: &str) -> LLMError {
        match status {
            400 => LLMError::InvalidConfiguration(format!("Bad request: {}", body)),
            401 => LLMError::AuthenticationFailed("Invalid API key or authentication failed".to_string()),
            403 => LLMError::AuthenticationFailed("Access forbidden".to_string()),
            404 => LLMError::ModelNotFound("Model not found".to_string()),
            429 => LLMError::RateLimitExceeded("Rate limit exceeded".to_string()),
            500..=599 => LLMError::InternalError(format!("Server error: {}", body)),
            _ => LLMError::NetworkError(format!("HTTP error {}: {}", status, body)),
        }
    }

    pub fn map_network_error(error: &str) -> LLMError {
        if error.contains("timeout") {
            LLMError::NetworkError("Request timeout".to_string())
        } else if error.contains("connection") {
            LLMError::NetworkError("Connection error".to_string())
        } else {
            LLMError::NetworkError(error.to_string())
        }
    }

    pub fn map_serialization_error(error: &str) -> LLMError {
        LLMError::SerializationError(format!("Serialization error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_wrapper_success() {
        let retry_wrapper = RetryWrapper::new(RetryConfig::default());
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_wrapper.execute_with_retry(|| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok::<i32, LLMError>(42)
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_retry_wrapper_with_retryable_error() {
        let retry_wrapper = RetryWrapper::new(RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        });
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_wrapper.execute_with_retry(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
                if count < 3 {
                    Err(LLMError::RateLimitExceeded("Rate limited".to_string()))
                } else {
                    Ok::<i32, LLMError>(42)
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let circuit_breaker = CircuitBreaker::new(2, Duration::from_millis(100));
        
        // First failure
        let result1 = circuit_breaker.execute(|| async {
            Err::<i32, LLMError>(LLMError::InternalError("Test error".to_string()))
        }).await;
        assert!(result1.is_err());

        // Second failure - should open circuit
        let result2 = circuit_breaker.execute(|| async {
            Err::<i32, LLMError>(LLMError::InternalError("Test error".to_string()))
        }).await;
        assert!(result2.is_err());

        // Third attempt - should be blocked by open circuit
        let result3 = circuit_breaker.execute(|| async {
            Ok::<i32, LLMError>(42)
        }).await;
        assert!(result3.is_err());
        assert!(result3.unwrap_err().to_string().contains("Circuit breaker is open"));
    }

    #[test]
    fn test_error_mapper_http_errors() {
        assert!(matches!(
            ErrorMapper::map_http_error(401, "Unauthorized"),
            LLMError::AuthenticationFailed(_)
        ));

        assert!(matches!(
            ErrorMapper::map_http_error(429, "Rate limited"),
            LLMError::RateLimitExceeded(_)
        ));

        assert!(matches!(
            ErrorMapper::map_http_error(500, "Internal error"),
            LLMError::InternalError(_)
        ));
    }
}