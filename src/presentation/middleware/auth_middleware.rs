use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{
    application::{services::AuthApplicationService, dto::AuthContext},
    error::PlatformError,
};

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, PlatformError> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| PlatformError::AuthenticationFailed("Missing authorization header".to_string()))?;

    // Extract token from "Bearer <token>" format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| PlatformError::AuthenticationFailed("Invalid authorization header format".to_string()))?;

    // Validate token and get auth context
    let auth_context = auth_service
        .validate_token(token)
        .await
        .map_err(|_| PlatformError::AuthenticationFailed("Invalid or expired token".to_string()))?;

    // Add auth context to request extensions
    request.extensions_mut().insert(auth_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Optional authentication middleware that doesn't fail if no token is provided
pub async fn optional_auth_middleware(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract authorization header
    if let Some(auth_header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        // Try to extract token from "Bearer <token>" format
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Try to validate token and get auth context
            if let Ok(auth_context) = auth_service.validate_token(token).await {
                // Add auth context to request extensions if valid
                request.extensions_mut().insert(auth_context);
            }
        }
    }

    // Continue to next middleware/handler regardless of authentication status
    next.run(request).await
}

/// Tenant permission middleware that checks if user has permission for tenant operations
pub fn tenant_permission_middleware(
    permission: String,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, PlatformError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let permission = permission.clone();
        Box::pin(async move {
            // Get auth context from request extensions
            let _auth_context = request
                .extensions()
                .get::<AuthContext>()
                .ok_or_else(|| PlatformError::AuthenticationFailed("Authentication required".to_string()))?;

            // For now, we'll implement a simple permission check
            // In a real application, this would check against a proper permission system
            let has_permission = match permission.as_str() {
                "read" | "write" | "execute" => true, // All authenticated users have basic permissions
                "admin" => false, // Admin permissions would require additional checks
                _ => false,
            };

            if !has_permission {
                return Err(PlatformError::AuthorizationFailed(
                    format!("Insufficient permissions: {}", permission)
                ));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Extract authentication context from request
pub fn extract_auth_context(request: &Request) -> Result<&AuthContext, PlatformError> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| PlatformError::AuthenticationFailed("Authentication required".to_string()))
}

/// Extract optional authentication context from request
pub fn extract_optional_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use axum::body::Body;
    
    use crate::application::dto::AuthContext;

    #[test]
    fn test_extract_auth_context() {
        let auth_context = AuthContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "testuser".to_string(),
            Some("Test User".to_string()),
            Uuid::new_v4(),
            None,
            None,
        );

        let mut request = Request::new(Body::empty());
        request.extensions_mut().insert(auth_context.clone());

        let extracted = extract_auth_context(&request).unwrap();
        assert_eq!(extracted.username, "testuser");
    }

    #[test]
    fn test_extract_optional_auth_context() {
        let request = Request::new(Body::empty());
        let extracted = extract_optional_auth_context(&request);
        assert!(extracted.is_none());
    }
}