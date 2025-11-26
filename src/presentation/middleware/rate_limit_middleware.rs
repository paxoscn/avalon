// Rate limiting middleware for API protection
// Requirement 1.4: Implement API rate limiting

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::Client;
use std::sync::Arc;
use std::time::Duration;

pub struct RateLimiter {
    redis_client: Arc<Client>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(redis_client: Arc<Client>, max_requests: u32, window: Duration) -> Self {
        Self {
            redis_client,
            max_requests,
            window,
        }
    }

    async fn check_rate_limit(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Increment counter
        let count: u32 = redis::cmd("INCR")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        
        // Set expiry on first request
        if count == 1 {
            redis::cmd("EXPIRE")
                .arg(key)
                .arg(self.window.as_secs())
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        
        Ok(count <= self.max_requests)
    }

    fn extract_client_id(req: &Request) -> String {
        // Extract client identifier from request
        // Priority: User ID from auth > IP address
        
        // Try to get user ID from headers (set by auth middleware)
        if let Some(user_id) = req.headers().get("x-user-id") {
            if let Ok(id) = user_id.to_str() {
                return format!("user:{}", id);
            }
        }
        
        // Fall back to IP address
        if let Some(forwarded) = req.headers().get("x-forwarded-for") {
            if let Ok(ip) = forwarded.to_str() {
                return format!("ip:{}", ip.split(',').next().unwrap_or("unknown"));
            }
        }
        
        if let Some(real_ip) = req.headers().get("x-real-ip") {
            if let Ok(ip) = real_ip.to_str() {
                return format!("ip:{}", ip);
            }
        }
        
        // Last resort
        "ip:unknown".to_string()
    }
}

pub async fn rate_limit_middleware(
    rate_limiter: Arc<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    let client_id = RateLimiter::extract_client_id(&req);
    let rate_limit_key = format!("rate_limit:{}", client_id);
    
    match rate_limiter.check_rate_limit(&rate_limit_key).await {
        Ok(allowed) => {
            if allowed {
                next.run(req).await
            } else {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded. Please try again later.",
                )
                    .into_response()
            }
        }
        Err(e) => {
            log::error!("Rate limit check failed: {}", e);
            // On error, allow the request (fail open)
            next.run(req).await
        }
    }
}

// Different rate limit tiers
pub struct RateLimitConfig {
    pub default_max_requests: u32,
    pub default_window: Duration,
    pub authenticated_max_requests: u32,
    pub authenticated_window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            default_max_requests: 100,
            default_window: Duration::from_secs(60),
            authenticated_max_requests: 1000,
            authenticated_window: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_client_id() {
        use axum::http::Request;
        
        let req = Request::builder()
            .header("x-user-id", "user-123")
            .body(Body::empty())
            .unwrap();
        
        let client_id = RateLimiter::extract_client_id(&req);
        assert_eq!(client_id, "user:user-123");
    }

    #[test]
    fn test_extract_client_id_from_ip() {
        use axum::http::Request;
        
        let req = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        
        let client_id = RateLimiter::extract_client_id(&req);
        assert_eq!(client_id, "ip:192.168.1.1");
    }
}
