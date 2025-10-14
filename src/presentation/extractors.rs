use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use async_trait::async_trait;
use serde_json::json;

use crate::application::dto::AuthContext;
use crate::domain::value_objects::{TenantId, UserId};

/// Rejection type for AuthContext extraction
#[derive(Debug)]
pub enum AuthContextRejection {
    Missing,
}

impl IntoResponse for AuthContextRejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthContextRejection::Missing => (
                StatusCode::UNAUTHORIZED,
                "Authentication required",
            ),
        };

        let body = Json(json!({
            "error": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        (status, body).into_response()
    }
}

/// Authenticated user extractor
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub username: String,
    pub nickname: Option<String>,
}

impl From<AuthContext> for AuthenticatedUser {
    fn from(ctx: AuthContext) -> Self {
        Self {
            user_id: UserId::from_uuid(ctx.user_id),
            tenant_id: TenantId::from_uuid(ctx.tenant_id),
            username: ctx.username,
            nickname: ctx.nickname,
        }
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthContextRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_context = parts
            .extensions
            .get::<AuthContext>()
            .ok_or(AuthContextRejection::Missing)?;
        
        Ok(AuthenticatedUser::from(auth_context.clone()))
    }
}

/// Optional authentication context extractor
/// This will not fail if no authentication context is present
pub struct OptionalAuthContext(pub Option<AuthContext>);

/// Extract authentication context from request
pub fn extract_auth_context(request: &Request) -> Result<&AuthContext, AuthContextRejection> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or(AuthContextRejection::Missing)
}

/// Extract optional authentication context from request
pub fn extract_optional_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

/// Extract client IP address from request
pub fn extract_client_ip(request: &Request) -> Option<String> {
    // Try to get IP from X-Forwarded-For header first (for proxied requests)
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // Take the first IP in the comma-separated list
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    // Fallback to connection info (this would require additional setup in real implementation)
    None
}

/// Extract user agent from request
pub fn extract_user_agent(request: &Request) -> Option<String> {
    request
        .headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|ua| ua.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    

    #[test]
    fn test_extract_client_ip() {
        let request = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_user_agent() {
        let request = Request::builder()
            .header("user-agent", "Mozilla/5.0 (Test Browser)")
            .body(Body::empty())
            .unwrap();

        let user_agent = extract_user_agent(&request);
        assert_eq!(user_agent, Some("Mozilla/5.0 (Test Browser)".to_string()));
    }
}