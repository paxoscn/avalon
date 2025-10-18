use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::services::{LLMApplicationService, VectorApplicationService},
    domain::value_objects::{
        ConfigId, ModelConfig, ModelCredentials, ModelParameters, ModelProvider,
    },
    domain::LLMConfig,
    error::Result,
    infrastructure::vector::VectorProvider,
    presentation::extractors::AuthenticatedUser,
};

// LLM Configuration DTOs
#[derive(Debug, Deserialize)]
pub struct CreateLLMConfigRequest {
    pub name: String,
    pub provider: String,
    pub model_name: String,
    pub parameters: Option<Value>,
    pub credentials: Option<Value>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLLMConfigRequest {
    pub name: Option<String>,
    pub model_name: Option<String>,
    pub parameters: Option<Value>,
    pub credentials: Option<Value>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LLMConfigResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub provider: String,
    pub model_name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub config: LLMConfig,
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResponse {
    pub success: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub model_info: Option<Value>,
    pub response: Option<String>,
    pub usage: Option<TokenUsageResponse>,
}

#[derive(Debug, Serialize)]
pub struct TokenUsageResponse {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct TestLLMConnectionRequest {
    pub system_prompt: Option<String>,
    pub user_prompt: String,
}

// Vector Configuration DTOs
#[derive(Debug, Deserialize)]
pub struct CreateVectorConfigRequest {
    pub name: String,
    pub provider: String,
    pub connection_params: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVectorConfigRequest {
    pub name: Option<String>,
    pub connection_params: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct VectorConfigResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub provider: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListConfigsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub provider: Option<String>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    20
}

#[derive(Debug, Serialize)]
pub struct PaginatedLLMConfigResponse {
    pub data: Vec<LLMConfigResponse>,
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct PaginatedVectorConfigResponse {
    pub data: Vec<VectorConfigResponse>,
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct ProviderParamsResponse {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

// LLM Configuration Handlers
pub async fn create_llm_config(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Json(req): Json<CreateLLMConfigRequest>,
) -> Result<impl IntoResponse> {
    let provider = ModelProvider::parse_model_provider(&req.provider)?;
    let parameters = parse_model_parameters(req.parameters)?;
    let credentials = parse_model_credentials(req.credentials)?;

    let model_config = ModelConfig {
        provider,
        model_name: req.model_name,
        parameters,
        credentials,
    };

    let config = service
        .create_config(user.tenant_id, req.name, model_config, req.description)
        .await?;

    Ok((StatusCode::CREATED, Json(llm_config_to_response(&config))))
}

pub async fn get_llm_config(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let config = service
        .get_config(ConfigId(config_id), user.tenant_id)
        .await?;
    Ok(Json(llm_config_to_response(&config)))
}

pub async fn list_llm_configs(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<ListConfigsQuery>,
) -> Result<impl IntoResponse> {
    // Convert from 1-based (API) to 0-based (internal)
    let page = query.page.saturating_sub(1);
    let limit = query.limit;

    let (configs, total) = if let Some(provider) = query.provider {
        // For filtered queries, still need pagination
        let all_configs = service
            .get_configs_by_provider(user.tenant_id, &provider)
            .await?;
        let total = all_configs.len() as u64;
        let offset = (page * limit) as usize;
        let paginated: Vec<_> = all_configs
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();
        (paginated, total)
    } else {
        service
            .list_configs_paginated(user.tenant_id, page, limit)
            .await?
    };

    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };

    let response = PaginatedLLMConfigResponse {
        data: configs.iter().map(llm_config_to_response).collect(),
        page: page + 1, // Convert back to 1-based for API
        limit,
        total,
        total_pages,
    };

    Ok(Json(response))
}

pub async fn update_llm_config(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
    Json(req): Json<UpdateLLMConfigRequest>,
) -> Result<impl IntoResponse> {
    let model_config =
        if req.model_name.is_some() || req.parameters.is_some() || req.credentials.is_some() {
            // Get existing config to merge changes
            let existing = service
                .get_config(ConfigId(config_id), user.tenant_id)
                .await?;

            let parameters = if let Some(p) = req.parameters {
                parse_model_parameters(Some(p))?
            } else {
                existing.model_config.parameters
            };

            let credentials = if let Some(c) = req.credentials {
                parse_model_credentials(Some(c))?
            } else {
                existing.model_config.credentials
            };

            Some(ModelConfig {
                provider: existing.model_config.provider,
                model_name: req.model_name.unwrap_or(existing.model_config.model_name),
                parameters,
                credentials,
            })
        } else {
            None
        };

    let config = service
        .update_config(
            ConfigId(config_id),
            user.tenant_id,
            req.name,
            model_config,
            req.description,
        )
        .await?;

    Ok(Json(llm_config_to_response(&config)))
}

pub async fn delete_llm_config(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service
        .delete_config(ConfigId(config_id), user.tenant_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default_llm_config(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service
        .set_default_config(ConfigId(config_id), user.tenant_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_llm_connection(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
    Json(req): Json<TestLLMConnectionRequest>,
) -> Result<impl IntoResponse> {
    use crate::domain::value_objects::ChatMessage;

    let start_time = std::time::Instant::now();

    // Build messages from prompts
    let mut messages = Vec::new();

    if let Some(system_prompt) = req.system_prompt {
        if !system_prompt.trim().is_empty() {
            messages.push(ChatMessage::new_system_message(system_prompt));
        }
    }

    messages.push(ChatMessage::new_user_message(req.user_prompt));

    // Call the LLM with the prompts
    match service
        .test_connection_with_prompts(ConfigId(config_id), user.tenant_id, messages)
        .await
    {
        Ok(chat_response) => {
            let response_time = start_time.elapsed().as_millis() as u64;

            let response = ConnectionTestResponse {
                success: true,
                response_time_ms: response_time,
                error_message: None,
                model_info: Some(serde_json::json!({
                    "model_used": chat_response.model_used,
                })),
                response: Some(chat_response.content),
                usage: Some(TokenUsageResponse {
                    prompt_tokens: chat_response.usage.prompt_tokens,
                    completion_tokens: chat_response.usage.completion_tokens,
                    total_tokens: chat_response.usage.total_tokens,
                }),
            };

            Ok(Json(response))
        }
        Err(e) => {
            let response_time = start_time.elapsed().as_millis() as u64;

            let response = ConnectionTestResponse {
                success: false,
                response_time_ms: response_time,
                error_message: Some(e.to_string()),
                model_info: None,
                response: None,
                usage: None,
            };

            Ok(Json(response))
        }
    }
}

pub async fn get_available_models(
    State(service): State<Arc<dyn LLMApplicationService>>,
    _user: AuthenticatedUser,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse> {
    let models = service.get_available_models(&provider).await?;
    Ok(Json(models))
}

// Vector Configuration Handlers
pub async fn create_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Json(req): Json<CreateVectorConfigRequest>,
) -> Result<impl IntoResponse> {
    let provider = parse_vector_provider(&req.provider)?;

    let config = service
        .create_config(user.tenant_id, req.name, provider, req.connection_params)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(vector_config_to_response(&config)),
    ))
}

pub async fn get_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    _user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let config = service.get_config(ConfigId(config_id)).await?;
    Ok(Json(vector_config_to_response(&config)))
}

pub async fn list_vector_configs(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<ListConfigsQuery>,
) -> Result<impl IntoResponse> {
    // Convert from 1-based (API) to 0-based (internal)
    let page = query.page.saturating_sub(1);
    let limit = query.limit;

    let (configs, total) = if let Some(provider_str) = query.provider {
        // For filtered queries, still need pagination
        let provider = parse_vector_provider(&provider_str)?;
        let all_configs = service
            .get_configs_by_provider(user.tenant_id, provider)
            .await?;
        let total = all_configs.len() as u64;
        let offset = (page * limit) as usize;
        let paginated: Vec<_> = all_configs
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();
        (paginated, total)
    } else {
        service.list_configs(user.tenant_id, page, limit).await?
    };

    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };

    let response = PaginatedVectorConfigResponse {
        data: configs.iter().map(vector_config_to_response).collect(),
        page: page + 1, // Convert back to 1-based for API
        limit,
        total,
        total_pages,
    };

    Ok(Json(response))
}

pub async fn update_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    _user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
    Json(req): Json<UpdateVectorConfigRequest>,
) -> Result<impl IntoResponse> {
    let config = service
        .update_config(ConfigId(config_id), req.name, req.connection_params)
        .await?;

    Ok(Json(vector_config_to_response(&config)))
}

pub async fn delete_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    _user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.delete_config(ConfigId(config_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default_vector_config(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service
        .set_as_default(ConfigId(config_id), user.tenant_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_vector_connection(
    State(service): State<Arc<VectorApplicationService>>,
    _user: AuthenticatedUser,
    Path(config_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.test_connection_by_id(ConfigId(config_id)).await?;

    let response = ConnectionTestResponse {
        success: true,
        response_time_ms: 0,
        error_message: None,
        model_info: None,
        response: None,
        usage: None,
    };

    Ok(Json(response))
}

pub async fn get_vector_provider_params(
    _user: AuthenticatedUser,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse> {
    let provider_enum = parse_vector_provider(&provider)?;

    let required = VectorApplicationService::get_required_params(provider_enum.clone());
    let optional = VectorApplicationService::get_optional_params(provider_enum);

    let response = ProviderParamsResponse { required, optional };

    Ok(Json(response))
}

pub async fn get_vector_health_status(
    State(service): State<Arc<VectorApplicationService>>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    let health = service.get_health_status(user.tenant_id).await?;
    Ok(Json(health))
}

// Helper functions
fn llm_config_to_response(config: &crate::domain::entities::LLMConfig) -> LLMConfigResponse {
    LLMConfigResponse {
        id: config.id.0.to_string(),
        tenant_id: config.tenant_id.0.to_string(),
        name: config.name.clone(),
        provider: format!("{:?}", config.model_config.provider),
        model_name: config.model_config.model_name.clone(),
        is_default: config.is_default,
        is_active: config.is_active,
        description: config.description.clone(),
        created_at: config.created_at.to_rfc3339(),
        updated_at: config.updated_at.to_rfc3339(),
        config: config.clone(),
    }
}

fn vector_config_to_response(
    config: &crate::domain::entities::VectorConfigEntity,
) -> VectorConfigResponse {
    VectorConfigResponse {
        id: config.id.0.to_string(),
        tenant_id: config.tenant_id.0.to_string(),
        name: config.name.clone(),
        provider: config.provider.as_str().to_string(),
        is_default: config.is_default,
        created_at: config.created_at.to_rfc3339(),
        updated_at: config.updated_at.to_rfc3339(),
    }
}

fn parse_vector_provider(provider: &str) -> Result<VectorProvider> {
    VectorProvider::from_str(provider)
}

fn parse_model_parameters(value: Option<Value>) -> Result<ModelParameters> {
    if let Some(v) = value {
        serde_json::from_value(v).map_err(|e| {
            crate::error::PlatformError::ValidationError(format!("Invalid model parameters: {}", e))
        })
    } else {
        Ok(ModelParameters::default())
    }
}

fn parse_model_credentials(value: Option<Value>) -> Result<ModelCredentials> {
    if let Some(v) = value {
        serde_json::from_value(v).map_err(|e| {
            crate::error::PlatformError::ValidationError(format!(
                "Invalid model credentials: {}",
                e
            ))
        })
    } else {
        Ok(ModelCredentials::default())
    }
}
