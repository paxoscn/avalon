use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{
    APIKeyAuthContext, APIKeyDTO, APIKeyListResponse, CreateAPIKeyRequest, CreateAPIKeyResponse,
    PermissionScopeDTO, UpdateAPIKeyRequest,
};
use crate::domain::entities::{AuditAction, AuditContext, ResourceType as AuditResourceType};
use crate::domain::repositories::{APIKeyRepository, QueryOptions};
use crate::domain::services::APIKeyService;
use crate::domain::value_objects::{
    APIKeyId, PermissionScope, ResourceType, TenantId, UserId,
};
use crate::error::{PlatformError, Result};

use super::AuditApplicationService;

/// Application service for API key management
pub struct APIKeyApplicationService {
    api_key_service: Arc<dyn APIKeyService>,
    repository: Arc<dyn APIKeyRepository>,
    audit_service: Arc<AuditApplicationService>,
}

impl APIKeyApplicationService {
    /// Create a new API key application service
    pub fn new(
        api_key_service: Arc<dyn APIKeyService>,
        repository: Arc<dyn APIKeyRepository>,
        audit_service: Arc<AuditApplicationService>,
    ) -> Self {
        Self {
            api_key_service,
            repository,
            audit_service,
        }
    }

    /// Create a new API key
    pub async fn create_api_key(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        request: CreateAPIKeyRequest,
        context: Option<AuditContext>,
    ) -> Result<CreateAPIKeyResponse> {
        // Convert DTO to domain value object
        let permission_scope = PermissionScope::new(
            request.permission_scope.agent_ids,
            request.permission_scope.flow_ids,
            request.permission_scope.mcp_tool_ids,
            request.permission_scope.vector_store_ids,
        );

        // Create the API key using domain service
        let (api_key, token) = self
            .api_key_service
            .create_api_key(
                tenant_id,
                user_id,
                request.name.clone(),
                permission_scope,
                request.expires_at,
            )
            .await?;

        // Log the creation event
        let details = json!({
            "api_key_id": api_key.id.0,
            "name": api_key.name,
            "expires_at": api_key.expires_at,
        });

        self.audit_service
            .log_event(
                tenant_id.0,
                Some(user_id.0),
                AuditAction::Create,
                AuditResourceType::Custom("api_key".to_string()),
                Some(api_key.id.0),
                Some(details),
                context,
            )
            .await?;

        // Convert to response DTO
        Ok(CreateAPIKeyResponse {
            id: api_key.id.0,
            name: api_key.name,
            token: token.into_string(),
            permission_scope: PermissionScopeDTO {
                agent_ids: api_key.permission_scope.agent_ids,
                flow_ids: api_key.permission_scope.flow_ids,
                mcp_tool_ids: api_key.permission_scope.mcp_tool_ids,
                vector_store_ids: api_key.permission_scope.vector_store_ids,
            },
            enabled: api_key.enabled,
            expires_at: api_key.expires_at,
            created_at: api_key.created_at,
        })
    }

    /// List API keys for a user with pagination
    pub async fn list_api_keys(
        &self,
        user_id: UserId,
        offset: Option<u64>,
        limit: Option<u64>,
        enabled_filter: Option<bool>,
        include_expired: bool,
    ) -> Result<APIKeyListResponse> {
        let options = QueryOptions {
            offset,
            limit,
            enabled_filter,
            include_expired,
        };

        let result = self.repository.find_by_user(user_id, options).await?;

        let items = result
            .items
            .into_iter()
            .map(|api_key| APIKeyDTO {
                id: api_key.id.0,
                name: api_key.name,
                permission_scope: PermissionScopeDTO {
                    agent_ids: api_key.permission_scope.agent_ids,
                    flow_ids: api_key.permission_scope.flow_ids,
                    mcp_tool_ids: api_key.permission_scope.mcp_tool_ids,
                    vector_store_ids: api_key.permission_scope.vector_store_ids,
                },
                enabled: api_key.enabled,
                expires_at: api_key.expires_at,
                last_used_at: api_key.last_used_at,
                created_at: api_key.created_at,
                updated_at: api_key.updated_at,
            })
            .collect();

        Ok(APIKeyListResponse {
            items,
            total: result.total,
            offset: result.offset,
            limit: result.limit,
        })
    }

    /// Get a single API key by ID
    pub async fn get_api_key(&self, id: APIKeyId, user_id: UserId) -> Result<APIKeyDTO> {
        let api_key = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("API key not found".to_string()))?;

        // Verify ownership
        if !api_key.belongs_to_user(&user_id) {
            return Err(PlatformError::Forbidden(
                "You do not have permission to access this API key".to_string(),
            ));
        }

        Ok(APIKeyDTO {
            id: api_key.id.0,
            name: api_key.name,
            permission_scope: PermissionScopeDTO {
                agent_ids: api_key.permission_scope.agent_ids,
                flow_ids: api_key.permission_scope.flow_ids,
                mcp_tool_ids: api_key.permission_scope.mcp_tool_ids,
                vector_store_ids: api_key.permission_scope.vector_store_ids,
            },
            enabled: api_key.enabled,
            expires_at: api_key.expires_at,
            last_used_at: api_key.last_used_at,
            created_at: api_key.created_at,
            updated_at: api_key.updated_at,
        })
    }

    /// Update an API key
    pub async fn update_api_key(
        &self,
        id: APIKeyId,
        user_id: UserId,
        request: UpdateAPIKeyRequest,
        context: Option<AuditContext>,
    ) -> Result<APIKeyDTO> {
        let mut api_key = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("API key not found".to_string()))?;

        // Verify ownership
        if !api_key.belongs_to_user(&user_id) {
            return Err(PlatformError::Forbidden(
                "You do not have permission to update this API key".to_string(),
            ));
        }

        let mut changes = Vec::new();

        // Update name if provided
        if let Some(name) = request.name {
            api_key.update_name(name.clone())?;
            changes.push(format!("name: {}", name));
        }

        // Update enabled status if provided
        if let Some(enabled) = request.enabled {
            if enabled {
                api_key.enable();
                changes.push("enabled: true".to_string());
            } else {
                api_key.disable();
                changes.push("enabled: false".to_string());
            }
        }

        // Update expiration if provided
        if let Some(expires_at) = request.expires_at {
            api_key.update_expiration(Some(expires_at))?;
            changes.push(format!("expires_at: {}", expires_at));
        }

        // Save the updated API key
        self.repository.update(&api_key).await?;

        // Log the update event
        let details = json!({
            "api_key_id": api_key.id.0,
            "changes": changes,
        });

        self.audit_service
            .log_event(
                api_key.tenant_id.0,
                Some(user_id.0),
                AuditAction::Update,
                AuditResourceType::Custom("api_key".to_string()),
                Some(api_key.id.0),
                Some(details),
                context,
            )
            .await?;

        Ok(APIKeyDTO {
            id: api_key.id.0,
            name: api_key.name,
            permission_scope: PermissionScopeDTO {
                agent_ids: api_key.permission_scope.agent_ids,
                flow_ids: api_key.permission_scope.flow_ids,
                mcp_tool_ids: api_key.permission_scope.mcp_tool_ids,
                vector_store_ids: api_key.permission_scope.vector_store_ids,
            },
            enabled: api_key.enabled,
            expires_at: api_key.expires_at,
            last_used_at: api_key.last_used_at,
            created_at: api_key.created_at,
            updated_at: api_key.updated_at,
        })
    }

    /// Delete an API key
    pub async fn delete_api_key(
        &self,
        id: APIKeyId,
        user_id: UserId,
        context: Option<AuditContext>,
    ) -> Result<()> {
        let api_key = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("API key not found".to_string()))?;

        // Verify ownership
        if !api_key.belongs_to_user(&user_id) {
            return Err(PlatformError::Forbidden(
                "You do not have permission to delete this API key".to_string(),
            ));
        }

        // Delete the API key
        self.repository.delete(id).await?;

        // Log the deletion event
        let details = json!({
            "api_key_id": api_key.id.0,
            "name": api_key.name,
        });

        self.audit_service
            .log_event(
                api_key.tenant_id.0,
                Some(user_id.0),
                AuditAction::Delete,
                AuditResourceType::Custom("api_key".to_string()),
                Some(api_key.id.0),
                Some(details),
                context,
            )
            .await?;

        Ok(())
    }

    /// Validate an API key token and return auth context
    pub async fn validate_api_key(
        &self,
        token: &str,
        context: Option<AuditContext>,
    ) -> Result<APIKeyAuthContext> {
        // Validate and get the API key
        let api_key = self.api_key_service.validate_and_get_key(token).await;

        match &api_key {
            Ok(key) => {
                // Log successful authentication
                let details = json!({
                    "api_key_id": key.id.0,
                    "success": true,
                });

                let _ = self
                    .audit_service
                    .log_event(
                        key.tenant_id.0,
                        Some(key.user_id.0),
                        AuditAction::Custom("api_key_auth".to_string()),
                        AuditResourceType::Custom("api_key".to_string()),
                        Some(key.id.0),
                        Some(details),
                        context,
                    )
                    .await;
            }
            Err(_) => {
                // Log failed authentication (without tenant/user info)
                let details = json!({
                    "success": false,
                    "reason": "invalid_token",
                });

                // Use a dummy tenant ID for failed auth attempts
                let _ = self
                    .audit_service
                    .log_event(
                        Uuid::nil(),
                        None,
                        AuditAction::Custom("api_key_auth_failed".to_string()),
                        AuditResourceType::Custom("api_key".to_string()),
                        None,
                        Some(details),
                        context,
                    )
                    .await;
            }
        }

        let api_key = api_key?;

        Ok(APIKeyAuthContext {
            api_key_id: api_key.id.0,
            tenant_id: api_key.tenant_id.0,
            user_id: api_key.user_id.0,
            permission_scope: PermissionScopeDTO {
                agent_ids: api_key.permission_scope.agent_ids,
                flow_ids: api_key.permission_scope.flow_ids,
                mcp_tool_ids: api_key.permission_scope.mcp_tool_ids,
                vector_store_ids: api_key.permission_scope.vector_store_ids,
            },
        })
    }

    /// Check if an API key has permission to access a resource
    pub async fn check_permission(
        &self,
        api_key_id: APIKeyId,
        resource_type: ResourceType,
        resource_id: Uuid,
        context: Option<AuditContext>,
    ) -> Result<bool> {
        let api_key = self
            .repository
            .find_by_id(api_key_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("API key not found".to_string()))?;

        let has_permission = self
            .api_key_service
            .check_resource_permission(&api_key, resource_type, resource_id)
            .await?;

        // Log authorization failure
        if !has_permission {
            let details = json!({
                "api_key_id": api_key.id.0,
                "resource_type": resource_type.as_str(),
                "resource_id": resource_id,
                "result": "denied",
            });

            let _ = self
                .audit_service
                .log_event(
                    api_key.tenant_id.0,
                    Some(api_key.user_id.0),
                    AuditAction::Custom("api_key_authz_failed".to_string()),
                    AuditResourceType::Custom("api_key".to_string()),
                    Some(api_key.id.0),
                    Some(details),
                    context,
                )
                .await;
        }

        Ok(has_permission)
    }

    /// Update the last used timestamp for an API key
    pub async fn update_last_used(&self, api_key_id: APIKeyId) -> Result<()> {
        let mut api_key = self
            .repository
            .find_by_id(api_key_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("API key not found".to_string()))?;

        api_key.update_last_used();
        self.repository.update(&api_key).await?;

        Ok(())
    }
}
