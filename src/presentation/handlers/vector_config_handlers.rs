use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;


use crate::application::services::VectorApplicationService;
use crate::domain::value_objects::ConfigId;
use crate::error::PlatformError;
use crate::infrastructure::vector::VectorProvider;
use crate::presentation::extractors::AuthenticatedUser;

/// Request to create a new vector configuration
#[derive(Debug, Deserialize)]
pub struct CreateVectorConfigRequest {
    pub name: String,
    pub provider: String,
    pub connection_params: HashMap<String, String>,
}

/// Request to update a vector configuration
#[derive(Debug, Deserialize)]
pub struct UpdateVectorConfigRequest {
    pub name: Option<String>,
    pub connection_params: Option<HashMap<String, String>>,
}

/// Response for vector configuration
#[derive(Debug, Serialize)]
pub struct VectorConfigResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub provider: String,
    pub connection_params: HashMap<String, String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Response for vector configuration list
#[derive(Debug, Serialize)]
pub struct VectorConfigListResponse {
    pub configs: Vec<VectorConfigResponse>,
    pub total: usize,
}

/// Response for connection test
#[derive(Debug, Serialize)]
pub struct ConnectionTestResponse {
    pub success: bool,
    pub message: String,
}

/// Response for health status
#[derive(Debug, Serialize)]
pub struct HealthStatusResponse {
    pub status: HashMap<String, bool>,
}

/// Query parameters for listing configurations
#[derive(Debug, Deserialize)]
pub struct ListConfigsQuery {
    pub provider: Option<String>,
}

/// Request to set default configuration
#[derive(Debug, Deserialize)]
pub struct SetDefaultRequest {
    pub config_id: String,
}

impl From<crate::domain::entities::VectorConfigEntity> for VectorConfigResponse {
    fn from(config: crate::domain::entities::VectorConfigEntity) -> Self {
        // Sanitize sensitive data for response
        let sanitized_config = config.sanitized();
        
        VectorConfigResponse {
            id: config.id.to_string(),
            tenant_id: config.tenant_id.to_string(),
            name: config.name,
            provider: config.provider.as_str().to_string(),
            connection_params: sanitized_config.connection_params,
            is_default: config.is_default,
            created_at: config.created_at.to_rfc3339(),
            updated_at: config.updated_at.to_rfc3339(),
        }
    }
}

/// Create a new vector configuration
pub async fn create_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<CreateVectorConfigRequest>,
) -> Result<Json<VectorConfigResponse>, PlatformError> {
    // Parse provider
    let provider = VectorProvider::from_str(&request.provider)?;
    
    // Create configuration
    let config = service
        .create_config(
            user.tenant_id,
            request.name,
            provider,
            request.connection_params,
        )
        .await?;
    
    Ok(Json(config.into()))
}

/// Update an existing vector configuration
pub async fn update_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<String>,
    Json(request): Json<UpdateVectorConfigRequest>,
) -> Result<Json<VectorConfigResponse>, PlatformError> {
    let config_id = ConfigId::from_string(&config_id)
        .map_err(|_| PlatformError::ValidationError("Invalid config ID format".to_string()))?;
    
    let config = service
        .update_config(config_id, request.name, request.connection_params)
        .await?;
    
    Ok(Json(config.into()))
}

/// Delete a vector configuration
pub async fn delete_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<String>,
) -> Result<StatusCode, PlatformError> {
    let config_id = ConfigId::from_string(&config_id)
        .map_err(|_| PlatformError::ValidationError("Invalid config ID format".to_string()))?;
    
    service.delete_config(config_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// Get a vector configuration by ID
pub async fn get_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<String>,
) -> Result<Json<VectorConfigResponse>, PlatformError> {
    let config_id = ConfigId::from_string(&config_id)
        .map_err(|_| PlatformError::ValidationError("Invalid config ID format".to_string()))?;
    
    let config = service.get_config(config_id).await?;
    
    // Verify tenant access
    if config.tenant_id != user.tenant_id {
        return Err(PlatformError::AuthorizationFailed(
            "Access denied to this configuration".to_string(),
        ));
    }
    
    Ok(Json(config.into()))
}

/// List vector configurations for the authenticated user's tenant
pub async fn list_vector_configs(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<ListConfigsQuery>,
) -> Result<Json<VectorConfigListResponse>, PlatformError> {
    let configs = if let Some(provider_str) = query.provider {
        let provider = VectorProvider::from_str(&provider_str)?;
        service.get_configs_by_provider(user.tenant_id, provider).await?
    } else {
        service.get_configs_by_tenant(user.tenant_id).await?
    };
    
    let config_responses: Vec<VectorConfigResponse> = configs
        .into_iter()
        .map(|config| config.into())
        .collect();
    
    let total = config_responses.len();
    
    Ok(Json(VectorConfigListResponse {
        configs: config_responses,
        total,
    }))
}

/// Get the default vector configuration for the tenant
pub async fn get_default_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
) -> Result<Json<VectorConfigResponse>, PlatformError> {
    let config = service.get_default_config(user.tenant_id).await?;
    
    Ok(Json(config.into()))
}

/// Set a configuration as default for the tenant
pub async fn set_default_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<SetDefaultRequest>,
) -> Result<StatusCode, PlatformError> {
    let config_id = ConfigId::from_string(&request.config_id)
        .map_err(|_| PlatformError::ValidationError("Invalid config ID format".to_string()))?;
    
    service.set_as_default(config_id, user.tenant_id).await?;
    
    Ok(StatusCode::OK)
}

/// Test connection to a vector configuration
pub async fn test_vector_config_connection(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<String>,
) -> Result<Json<ConnectionTestResponse>, PlatformError> {
    let config_id = ConfigId::from_string(&config_id)
        .map_err(|_| PlatformError::ValidationError("Invalid config ID format".to_string()))?;
    
    match service.test_connection_by_id(config_id).await {
        Ok(_) => Ok(Json(ConnectionTestResponse {
            success: true,
            message: "Connection successful".to_string(),
        })),
        Err(e) => Ok(Json(ConnectionTestResponse {
            success: false,
            message: e.to_string(),
        })),
    }
}

/// Get health status of all configurations for the tenant
pub async fn get_vector_configs_health(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
) -> Result<Json<HealthStatusResponse>, PlatformError> {
    let health_status = service.get_health_status(user.tenant_id).await?;
    
    Ok(Json(HealthStatusResponse {
        status: health_status,
    }))
}

/// Get required parameters for a vector provider
pub async fn get_provider_params(
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>, PlatformError> {
    let provider_enum = VectorProvider::from_str(&provider)?;
    
    let required_params = VectorApplicationService::get_required_params(provider_enum.clone());
    let optional_params = VectorApplicationService::get_optional_params(provider_enum);
    
    Ok(Json(serde_json::json!({
        "provider": provider,
        "required_params": required_params,
        "optional_params": optional_params
    })))
}

/// Validate provider parameters
pub async fn validate_provider_params(
    Path(provider): Path<String>,
    Json(params): Json<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, PlatformError> {
    let provider_enum = VectorProvider::from_str(&provider)?;
    
    match VectorApplicationService::validate_provider_params(provider_enum, &params) {
        Ok(_) => Ok(Json(serde_json::json!({
            "valid": true,
            "message": "Parameters are valid"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "valid": false,
            "message": e.to_string()
        }))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::VectorConfigEntity;
    use crate::domain::value_objects::{TenantId, ConfigId};
    use std::collections::HashMap;

    #[test]
    fn test_vector_config_response_conversion() {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "secret-key".to_string());
        params.insert("environment".to_string(), "test-env".to_string());
        
        let config = VectorConfigEntity::new(
            TenantId::new(),
            "Test Config".to_string(),
            VectorProvider::Pinecone,
            params,
        );
        
        let response: VectorConfigResponse = config.into();
        
        assert_eq!(response.name, "Test Config");
        assert_eq!(response.provider, "pinecone");
        // Sensitive data should be sanitized
        assert_eq!(response.connection_params.get("api_key").unwrap(), "***");
        assert_eq!(response.connection_params.get("environment").unwrap(), "test-env");
    }
    
    #[test]
    fn test_create_request_validation() {
        let request = CreateVectorConfigRequest {
            name: "Test Config".to_string(),
            provider: "pinecone".to_string(),
            connection_params: HashMap::new(),
        };
        
        assert_eq!(request.name, "Test Config");
        assert_eq!(request.provider, "pinecone");
    }
}