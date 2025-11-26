use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{
        dto::{
            APIKeyDTO, APIKeyListResponse, CreateAPIKeyRequest, CreateAPIKeyResponse,
            UpdateAPIKeyRequest,
        },
        services::APIKeyApplicationService,
    },
    domain::value_objects::APIKeyId,
    error::{PlatformError, Result},
    presentation::extractors::AuthenticatedUser,
};

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for listing API keys
#[derive(Debug, Deserialize)]
pub struct ListAPIKeysQuery {
    #[serde(default = "default_offset")]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub enabled: Option<bool>,
    #[serde(default)]
    pub include_expired: bool,
}

fn default_offset() -> u64 {
    0
}

fn default_limit() -> u64 {
    20
}

// ============================================================================
// CRUD Handlers
// ============================================================================

/// Create a new API key
///
/// POST /api/v1/api-keys
///
/// Creates a new API key with the specified permissions and expiration.
/// The token is returned only once in the response.
pub async fn create_api_key(
    State(service): State<Arc<APIKeyApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<CreateAPIKeyRequest>,
) -> Result<impl IntoResponse> {
    // Validate request
    validate_create_request(&request)?;

    // Create the API key
    let response = service
        .create_api_key(user.tenant_id, user.user_id, request, None)
        .await
        .map_err(|e| handle_database_error(e))?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// List API keys with pagination
///
/// GET /api/v1/api-keys
///
/// Returns a paginated list of API keys for the authenticated user.
/// The token values are not included in the response.
pub async fn list_api_keys(
    State(service): State<Arc<APIKeyApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<ListAPIKeysQuery>,
) -> Result<impl IntoResponse> {
    let response = service
        .list_api_keys(
            user.user_id,
            Some(query.offset),
            Some(query.limit),
            query.enabled,
            query.include_expired,
        )
        .await?;

    Ok(Json(response))
}

/// Get a single API key by ID
///
/// GET /api/v1/api-keys/:id
///
/// Returns the details of a specific API key.
/// The token value is not included in the response.
pub async fn get_api_key(
    State(service): State<Arc<APIKeyApplicationService>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let api_key = service.get_api_key(APIKeyId::from_uuid(id), user.user_id).await?;

    Ok(Json(api_key))
}

/// Update an API key
///
/// PATCH /api/v1/api-keys/:id
///
/// Updates the properties of an API key (name, enabled status, expiration).
/// The permission scope cannot be modified after creation.
pub async fn update_api_key(
    State(service): State<Arc<APIKeyApplicationService>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateAPIKeyRequest>,
) -> Result<impl IntoResponse> {
    // Validate request
    validate_update_request(&request)?;

    let api_key = service
        .update_api_key(APIKeyId::from_uuid(id), user.user_id, request, None)
        .await?;

    Ok(Json(api_key))
}

/// Delete an API key
///
/// DELETE /api/v1/api-keys/:id
///
/// Permanently deletes an API key. This action cannot be undone.
/// All subsequent authentication attempts with this key will fail.
pub async fn delete_api_key(
    State(service): State<Arc<APIKeyApplicationService>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service
        .delete_api_key(APIKeyId::from_uuid(id), user.user_id, None)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Error Handling
// ============================================================================

/// Handle database errors and convert them to appropriate HTTP errors
fn handle_database_error(error: PlatformError) -> PlatformError {
    match &error {
        PlatformError::DatabaseError(db_err) => {
            let err_msg = db_err.to_string();
            
            // Check for unique constraint violation on key_hash
            if err_msg.contains("unique") || err_msg.contains("duplicate") {
                if err_msg.contains("key_hash") {
                    return PlatformError::Conflict(
                        "An API key with this token already exists".to_string(),
                    );
                }
                // Generic duplicate error
                return PlatformError::Conflict(
                    "A duplicate entry was detected".to_string(),
                );
            }
            
            // Check for foreign key constraint violations
            if err_msg.contains("foreign key") {
                return PlatformError::ValidationError(
                    "Referenced resource does not exist".to_string(),
                );
            }
            
            // Return the original error for other database errors
            error
        }
        _ => error,
    }
}

// ============================================================================
// Request Validation
// ============================================================================

/// Validate create API key request
fn validate_create_request(request: &CreateAPIKeyRequest) -> Result<()> {
    // Validate name length
    if request.name.is_empty() {
        return Err(PlatformError::ValidationError(
            "API key name cannot be empty".to_string(),
        ));
    }

    if request.name.len() > 255 {
        return Err(PlatformError::ValidationError(
            "API key name cannot exceed 255 characters".to_string(),
        ));
    }

    // Validate expiration date (if provided)
    if let Some(expires_at) = request.expires_at {
        if expires_at <= chrono::Utc::now() {
            return Err(PlatformError::ValidationError(
                "Expiration date must be in the future".to_string(),
            ));
        }
    }

    // Validate permission scope (at least one resource must be specified)
    let scope = &request.permission_scope;
    if scope.agent_ids.is_empty()
        && scope.flow_ids.is_empty()
        && scope.mcp_tool_ids.is_empty()
        && scope.vector_store_ids.is_empty()
    {
        return Err(PlatformError::ValidationError(
            "Permission scope must include at least one resource".to_string(),
        ));
    }

    Ok(())
}

/// Validate update API key request
fn validate_update_request(request: &UpdateAPIKeyRequest) -> Result<()> {
    // Validate name length if provided
    if let Some(ref name) = request.name {
        if name.is_empty() {
            return Err(PlatformError::ValidationError(
                "API key name cannot be empty".to_string(),
            ));
        }

        if name.len() > 255 {
            return Err(PlatformError::ValidationError(
                "API key name cannot exceed 255 characters".to_string(),
            ));
        }
    }

    // Validate expiration date if provided
    if let Some(expires_at) = request.expires_at {
        if expires_at <= chrono::Utc::now() {
            return Err(PlatformError::ValidationError(
                "Expiration date must be in the future".to_string(),
            ));
        }
    }

    // At least one field must be provided
    if request.name.is_none() && request.enabled.is_none() && request.expires_at.is_none() {
        return Err(PlatformError::ValidationError(
            "At least one field must be provided for update".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::PermissionScopeDTO;
    use chrono::{Duration, Utc};
    use sea_orm;

    #[test]
    fn test_validate_create_request_valid() {
        let request = CreateAPIKeyRequest {
            name: "Test API Key".to_string(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![Uuid::new_v4()],
                flow_ids: vec![],
                mcp_tool_ids: vec![],
                vector_store_ids: vec![],
            },
            expires_at: Some(Utc::now() + Duration::days(30)),
        };

        assert!(validate_create_request(&request).is_ok());
    }

    #[test]
    fn test_validate_create_request_empty_name() {
        let request = CreateAPIKeyRequest {
            name: "".to_string(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![Uuid::new_v4()],
                flow_ids: vec![],
                mcp_tool_ids: vec![],
                vector_store_ids: vec![],
            },
            expires_at: None,
        };

        assert!(validate_create_request(&request).is_err());
    }

    #[test]
    fn test_validate_create_request_name_too_long() {
        let request = CreateAPIKeyRequest {
            name: "a".repeat(256),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![Uuid::new_v4()],
                flow_ids: vec![],
                mcp_tool_ids: vec![],
                vector_store_ids: vec![],
            },
            expires_at: None,
        };

        assert!(validate_create_request(&request).is_err());
    }

    #[test]
    fn test_validate_create_request_expired_date() {
        let request = CreateAPIKeyRequest {
            name: "Test API Key".to_string(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![Uuid::new_v4()],
                flow_ids: vec![],
                mcp_tool_ids: vec![],
                vector_store_ids: vec![],
            },
            expires_at: Some(Utc::now() - Duration::days(1)),
        };

        assert!(validate_create_request(&request).is_err());
    }

    #[test]
    fn test_validate_create_request_empty_scope() {
        let request = CreateAPIKeyRequest {
            name: "Test API Key".to_string(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![],
                vector_store_ids: vec![],
            },
            expires_at: None,
        };

        assert!(validate_create_request(&request).is_err());
    }

    #[test]
    fn test_validate_update_request_valid() {
        let request = UpdateAPIKeyRequest {
            name: Some("Updated Name".to_string()),
            enabled: Some(false),
            expires_at: Some(Utc::now() + Duration::days(30)),
        };

        assert!(validate_update_request(&request).is_ok());
    }

    #[test]
    fn test_validate_update_request_empty_name() {
        let request = UpdateAPIKeyRequest {
            name: Some("".to_string()),
            enabled: None,
            expires_at: None,
        };

        assert!(validate_update_request(&request).is_err());
    }

    #[test]
    fn test_validate_update_request_no_fields() {
        let request = UpdateAPIKeyRequest {
            name: None,
            enabled: None,
            expires_at: None,
        };

        assert!(validate_update_request(&request).is_err());
    }

    #[test]
    fn test_validate_update_request_expired_date() {
        let request = UpdateAPIKeyRequest {
            name: None,
            enabled: None,
            expires_at: Some(Utc::now() - Duration::days(1)),
        };

        assert!(validate_update_request(&request).is_err());
    }

    #[test]
    fn test_default_offset() {
        assert_eq!(default_offset(), 0);
    }

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 20);
    }

    #[test]
    fn test_handle_database_error_unique_key_hash() {
        let db_err = sea_orm::DbErr::Custom("unique constraint violation on key_hash".to_string());
        let error = PlatformError::DatabaseError(db_err);
        
        let result = handle_database_error(error);
        
        match result {
            PlatformError::Conflict(msg) => {
                assert!(msg.contains("token"));
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_handle_database_error_foreign_key() {
        let db_err = sea_orm::DbErr::Custom("foreign key constraint violation".to_string());
        let error = PlatformError::DatabaseError(db_err);
        
        let result = handle_database_error(error);
        
        match result {
            PlatformError::ValidationError(msg) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_handle_database_error_other() {
        let db_err = sea_orm::DbErr::Custom("some other error".to_string());
        let error = PlatformError::DatabaseError(db_err);
        
        let result = handle_database_error(error);
        
        match result {
            PlatformError::DatabaseError(_) => {
                // Should return the original error
            }
            _ => panic!("Expected DatabaseError"),
        }
    }
}
