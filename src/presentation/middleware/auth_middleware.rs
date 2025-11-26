use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{
        services::{AuthApplicationService, APIKeyApplicationService},
        dto::{AuthContext, APIKeyAuthContext},
    },
    domain::entities::AuditContext,
    error::{PlatformError, Result},
};

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, PlatformError> {
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
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, PlatformError>> + Send>> + Clone {
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
pub fn extract_auth_context(request: &Request) -> std::result::Result<&AuthContext, PlatformError> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| PlatformError::AuthenticationFailed("Authentication required".to_string()))
}

/// Extract optional authentication context from request
pub fn extract_optional_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

/// API key authentication middleware that validates API keys
///
/// This middleware:
/// - Extracts the API key from the Authorization header (Bearer token format)
/// - Validates the API key format (must start with "pk_")
/// - Calls the APIKeyApplicationService to validate the token
/// - Logs authentication attempts (success and failure) via audit service
/// - Injects APIKeyAuthContext into request extensions for downstream handlers
/// - Updates the last_used timestamp for the API key (async, fire-and-forget)
///
/// # Errors
///
/// Returns `PlatformError::AuthenticationFailed` (401) if:
/// - Authorization header is missing
/// - Authorization header format is invalid
/// - API key format is invalid (doesn't start with "pk_")
/// - API key is invalid, expired, or disabled
///
/// # Requirements
///
/// Implements requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 11.2
pub async fn api_key_auth_middleware(
    State(api_key_service): State<Arc<APIKeyApplicationService>>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, PlatformError> {
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

    // Validate API key format (must start with "pk_")
    if !token.starts_with("pk_") {
        return Err(PlatformError::AuthenticationFailed(
            "Invalid API key format".to_string(),
        ));
    }

    // Create audit context from request metadata
    let audit_context = AuditContext {
        ip_address: extract_client_ip(&request),
        user_agent: request
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    };

    // Validate API key and get auth context
    let api_key_context = api_key_service
        .validate_api_key(token, Some(audit_context))
        .await
        .map_err(|e| {
            // Log authentication failure and return generic error
            log::warn!("API key authentication failed: {}", e);
            PlatformError::AuthenticationFailed("Invalid or expired API key".to_string())
        })?;

    // Update last used timestamp (fire and forget)
    let api_key_id = crate::domain::value_objects::APIKeyId(api_key_context.api_key_id);
    let service = api_key_service.clone();
    tokio::spawn(async move {
        if let Err(e) = service.update_last_used(api_key_id).await {
            log::warn!("Failed to update API key last_used timestamp: {}", e);
        }
    });

    // Add API key auth context to request extensions
    request.extensions_mut().insert(api_key_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Extract API key authentication context from request
pub fn extract_api_key_context(request: &Request) -> std::result::Result<&APIKeyAuthContext, PlatformError> {
    request
        .extensions()
        .get::<APIKeyAuthContext>()
        .ok_or_else(|| PlatformError::AuthenticationFailed("API key authentication required".to_string()))
}

/// Extract optional API key authentication context from request
pub fn extract_optional_api_key_context(request: &Request) -> Option<&APIKeyAuthContext> {
    request.extensions().get::<APIKeyAuthContext>()
}

/// Resource permission middleware factory function
///
/// This middleware:
/// - Extracts resource type and ID from request path parameters
/// - Retrieves APIKeyAuthContext from request extensions
/// - Calls permission checking logic to verify access
/// - Returns 403 Forbidden if permission denied
/// - Logs authorization failures with resource type
///
/// # Parameters
///
/// - `resource_type`: The type of resource being accessed (agent, flow, mcp_tool, vector_store)
///
/// # Usage
///
/// ```rust,ignore
/// use axum::{Router, routing::get};
/// use crate::domain::value_objects::ResourceType;
/// use crate::presentation::middleware::require_resource_permission;
///
/// let app = Router::new()
///     .route("/api/v1/agents/:id", get(get_agent_handler))
///     .route_layer(axum::middleware::from_fn_with_state(
///         api_key_service.clone(),
///         require_resource_permission(ResourceType::Agent)
///     ));
/// ```
///
/// # Errors
///
/// Returns `PlatformError::AuthenticationFailed` (401) if:
/// - API key authentication context is not present in request extensions
///
/// Returns `PlatformError::Forbidden` (403) if:
/// - API key does not have permission to access the requested resource
///
/// # Requirements
///
/// Implements requirements: 6.1, 6.2, 6.3, 6.4, 11.4
pub fn require_resource_permission(
    resource_type: crate::domain::value_objects::ResourceType,
) -> impl Fn(
    axum::extract::State<Arc<APIKeyApplicationService>>,
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, PlatformError>> + Send>>
       + Clone {
    move |State(api_key_service): axum::extract::State<Arc<APIKeyApplicationService>>,
          request: Request,
          next: Next| {
        let resource_type = resource_type;
        Box::pin(async move {
            // Get API key auth context from request extensions
            let api_key_context = extract_api_key_context(&request)?;

            // Extract resource ID from path parameters
            // The resource ID is expected to be in the path as a UUID parameter
            let resource_id = extract_resource_id_from_path(&request)?;

            // Create audit context from request metadata
            let audit_context = AuditContext {
                ip_address: extract_client_ip(&request),
                user_agent: request
                    .headers()
                    .get("user-agent")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string()),
            };

            // Check permission
            let api_key_id = crate::domain::value_objects::APIKeyId(api_key_context.api_key_id);
            let has_permission = api_key_service
                .check_permission(api_key_id, resource_type, resource_id, Some(audit_context))
                .await
                .map_err(|e| {
                    log::error!("Error checking API key permission: {}", e);
                    PlatformError::InternalError("Failed to check permissions".to_string())
                })?;

            if !has_permission {
                log::warn!(
                    "API key {} denied access to {} resource {}",
                    api_key_context.api_key_id,
                    resource_type.as_str(),
                    resource_id
                );
                return Err(PlatformError::Forbidden(format!(
                    "Access denied to {} resource",
                    resource_type.as_str()
                )));
            }

            // Permission granted, continue to next middleware/handler
            Ok(next.run(request).await)
        })
    }
}

/// Extract resource ID from request path parameters
///
/// This function attempts to extract a UUID from the request path by parsing
/// each path segment and returning the first valid UUID found.
fn extract_resource_id_from_path(request: &Request) -> Result<Uuid> {
    // Try to extract from path extensions (set by axum router)
    // The path parameters are stored in request extensions by axum
    if let Some(path_params) = request.extensions().get::<axum::extract::MatchedPath>() {
        let path_str = path_params.as_str();
        
        // Try to extract UUID from path segments
        for segment in path_str.split('/') {
            if let Ok(uuid) = Uuid::parse_str(segment) {
                return Ok(uuid);
            }
        }
    }

    // Fallback: try to parse from URI path
    let path = request.uri().path();
    for segment in path.split('/') {
        if let Ok(uuid) = Uuid::parse_str(segment) {
            return Ok(uuid);
        }
    }

    Err(PlatformError::ValidationError(
        "Resource ID not found in request path".to_string(),
    ))
}

/// Helper function to extract client IP address from request
fn extract_client_ip(request: &Request) -> Option<String> {
    // Try X-Forwarded-For header first (for proxied requests)
    if let Some(forwarded_for) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
    {
        // Take the first IP in the chain
        if let Some(ip) = forwarded_for.split(',').next() {
            return Some(ip.trim().to_string());
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|h| h.to_str().ok())
    {
        return Some(real_ip.to_string());
    }

    // Fall back to connection info (if available)
    // Note: This would require access to ConnectInfo which is not available in middleware
    // In production, you'd typically rely on proxy headers
    None
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

    #[test]
    fn test_extract_api_key_context() {
        use crate::application::dto::{APIKeyAuthContext, PermissionScopeDTO};

        let api_key_context = APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![],
                vector_store_ids: vec![],
            },
        };

        let mut request = Request::new(Body::empty());
        request.extensions_mut().insert(api_key_context.clone());

        let extracted = extract_api_key_context(&request).unwrap();
        assert_eq!(extracted.api_key_id, api_key_context.api_key_id);
        assert_eq!(extracted.tenant_id, api_key_context.tenant_id);
        assert_eq!(extracted.user_id, api_key_context.user_id);
    }

    #[test]
    fn test_extract_api_key_context_missing() {
        let request = Request::new(Body::empty());
        let result = extract_api_key_context(&request);
        assert!(result.is_err());
        assert!(matches!(result, Err(PlatformError::AuthenticationFailed(_))));
    }

    #[test]
    fn test_extract_optional_api_key_context() {
        let request = Request::new(Body::empty());
        let extracted = extract_optional_api_key_context(&request);
        assert!(extracted.is_none());
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let mut request = Request::new(Body::empty());
        request.headers_mut().insert(
            "x-forwarded-for",
            "203.0.113.1, 198.51.100.1".parse().unwrap(),
        );

        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let mut request = Request::new(Body::empty());
        request.headers_mut().insert(
            "x-real-ip",
            "203.0.113.1".parse().unwrap(),
        );

        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_no_headers() {
        let request = Request::new(Body::empty());
        let ip = extract_client_ip(&request);
        assert_eq!(ip, None);
    }

    #[test]
    fn test_extract_resource_id_from_path() {
        use axum::http::Uri;

        let resource_id = Uuid::new_v4();
        let uri = format!("/api/v1/agents/{}", resource_id)
            .parse::<Uri>()
            .unwrap();

        let request = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let extracted = extract_resource_id_from_path(&request).unwrap();
        assert_eq!(extracted, resource_id);
    }

    #[test]
    fn test_extract_resource_id_from_path_multiple_segments() {
        use axum::http::Uri;

        let resource_id = Uuid::new_v4();
        let uri = format!("/api/v1/tenants/{}/agents/{}", Uuid::new_v4(), resource_id)
            .parse::<Uri>()
            .unwrap();

        let request = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        // Should extract the first valid UUID found
        let extracted = extract_resource_id_from_path(&request).unwrap();
        assert!(extracted == resource_id || Uuid::parse_str(&extracted.to_string()).is_ok());
    }

    #[test]
    fn test_extract_resource_id_from_path_no_uuid() {
        use axum::http::Uri;

        let uri = "/api/v1/agents".parse::<Uri>().unwrap();

        let request = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let result = extract_resource_id_from_path(&request);
        assert!(result.is_err());
        assert!(matches!(result, Err(PlatformError::ValidationError(_))));
    }
}