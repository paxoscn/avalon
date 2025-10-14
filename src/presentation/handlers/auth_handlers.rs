use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::{
    application::{
        services::AuthApplicationService,
        dto::{
            LoginRequest, RefreshTokenRequest, LogoutRequest, 
            ChangePasswordRequest, AuthContext
        },
    },
    error::PlatformError,
    presentation::extractors::{extract_client_ip, extract_user_agent},
};



/// Login handler
pub async fn login_handler(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    request: Request,
) -> Result<Response, PlatformError> {
    // Extract IP address and user agent from request
    let ip_address = extract_client_ip(&request);
    let user_agent = extract_user_agent(&request);

    // Extract the JSON body
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|e| PlatformError::ValidationError(format!("Failed to read request body: {}", e)))?;
    
    let login_request: LoginRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| PlatformError::ValidationError(format!("Invalid JSON: {}", e)))?;

    // Perform login
    let (login_response, _auth_event) = auth_service
        .login(login_request, ip_address, user_agent)
        .await?;

    // TODO: Publish auth_event to event bus

    Ok((StatusCode::OK, Json(login_response)).into_response())
}

/// Refresh token handler
pub async fn refresh_token_handler(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    request: Request,
) -> Result<Response, PlatformError> {
    // Extract IP address from request
    let ip_address = extract_client_ip(&request);

    // Extract the JSON body
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|e| PlatformError::ValidationError(format!("Failed to read request body: {}", e)))?;
    
    let refresh_request: RefreshTokenRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| PlatformError::ValidationError(format!("Invalid JSON: {}", e)))?;

    // Refresh token
    let (refresh_response, _refresh_event) = auth_service
        .refresh_token(refresh_request, ip_address)
        .await?;

    // TODO: Publish refresh_event to event bus

    Ok((StatusCode::OK, Json(refresh_response)).into_response())
}

/// Logout handler
pub async fn logout_handler(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    request: Request,
) -> Result<Response, PlatformError> {
    // Extract IP address from request
    let ip_address = extract_client_ip(&request);

    // Extract the JSON body
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|e| PlatformError::ValidationError(format!("Failed to read request body: {}", e)))?;
    
    let logout_request: LogoutRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| PlatformError::ValidationError(format!("Invalid JSON: {}", e)))?;

    // Perform logout
    let (logout_response, _logout_event) = auth_service
        .logout(logout_request, ip_address)
        .await?;

    // TODO: Publish logout_event to event bus

    Ok((StatusCode::OK, Json(logout_response)).into_response())
}

/// Change password handler
pub async fn change_password_handler(
    State(auth_service): State<Arc<dyn AuthApplicationService>>,
    axum::Extension(auth_context): axum::Extension<AuthContext>,
    Json(change_password_request): Json<ChangePasswordRequest>,
) -> Result<Response, PlatformError> {
    // Change password
    let (change_password_response, _password_change_event) = auth_service
        .change_password(change_password_request, auth_context)
        .await?;

    // TODO: Publish password_change_event to event bus

    Ok((StatusCode::OK, Json(change_password_response)).into_response())
}

/// Get current user info handler
pub async fn me_handler(
    axum::Extension(auth_context): axum::Extension<AuthContext>,
) -> Result<Response, PlatformError> {
    let user_info = json!({
        "id": auth_context.user_id,
        "tenant_id": auth_context.tenant_id,
        "username": auth_context.username,
        "nickname": auth_context.nickname,
    });

    Ok((StatusCode::OK, Json(user_info)).into_response())
}

/// Health check handler (no authentication required)
pub async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}