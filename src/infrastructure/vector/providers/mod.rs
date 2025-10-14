pub mod pinecone;
pub mod chromadb;
pub mod weaviate;
pub mod qdrant;
pub mod milvus;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::error::PlatformError;
use crate::infrastructure::vector::VectorStoreConfig;

/// Common HTTP client configuration for vector store providers
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            user_agent: "agent-platform/0.1.0".to_string(),
            default_headers: HashMap::new(),
        }
    }
}

/// HTTP client wrapper for vector store operations
pub struct VectorHttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl VectorHttpClient {
    pub fn new(config: HttpClientConfig) -> Result<Self, PlatformError> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Add default headers
        for (key, value) in &config.default_headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| PlatformError::ValidationError(format!("Invalid header name '{}': {}", key, e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(value)
                .map_err(|e| PlatformError::ValidationError(format!("Invalid header value '{}': {}", value, e)))?;
            headers.insert(header_name, header_value);
        }
        
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .default_headers(headers)
            .build()
            .map_err(|e| PlatformError::InternalError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self { client, config })
    }
    
    /// Send a POST request with JSON body
    pub async fn post_json<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HashMap<String, String>>,
    ) -> Result<R, PlatformError> {
        let mut request = self.client.post(url).json(body);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await
            .map_err(|e| PlatformError::VectorStoreError(format!("HTTP request failed: {}", e)))?;
        
        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| PlatformError::VectorStoreError(format!("Failed to read response: {}", e)))?;
        
        if status.is_success() {
            serde_json::from_str(&response_text)
                .map_err(|e| PlatformError::VectorStoreError(format!("Failed to parse response: {}", e)))
        } else {
            Err(PlatformError::VectorStoreError(
                format!("HTTP error {}: {}", status, response_text)
            ))
        }
    }
    
    /// Send a GET request
    pub async fn get<R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<R, PlatformError> {
        let mut request = self.client.get(url);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await
            .map_err(|e| PlatformError::VectorStoreError(format!("HTTP request failed: {}", e)))?;
        
        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| PlatformError::VectorStoreError(format!("Failed to read response: {}", e)))?;
        
        if status.is_success() {
            serde_json::from_str(&response_text)
                .map_err(|e| PlatformError::VectorStoreError(format!("Failed to parse response: {}", e)))
        } else {
            Err(PlatformError::VectorStoreError(
                format!("HTTP error {}: {}", status, response_text)
            ))
        }
    }
    
    /// Send a DELETE request
    pub async fn delete(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<(), PlatformError> {
        let mut request = self.client.delete(url);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await
            .map_err(|e| PlatformError::VectorStoreError(format!("HTTP request failed: {}", e)))?;
        
        let status = response.status();
        
        if status.is_success() {
            Ok(())
        } else {
            let response_text = response.text().await
                .map_err(|e| PlatformError::VectorStoreError(format!("Failed to read response: {}", e)))?;
            Err(PlatformError::VectorStoreError(
                format!("HTTP error {}: {}", status, response_text)
            ))
        }
    }
}

/// Common utilities for vector store providers
pub struct ProviderUtils;

impl ProviderUtils {
    /// Extract connection parameter from config
    pub fn get_connection_param(config: &VectorStoreConfig, key: &str) -> Result<String, PlatformError> {
        config.connection_params.get(key)
            .cloned()
            .ok_or_else(|| PlatformError::ValidationError(
                format!("Missing required connection parameter: {}", key)
            ))
    }
    
    /// Extract optional connection parameter from config
    pub fn get_optional_connection_param(config: &VectorStoreConfig, key: &str) -> Option<String> {
        config.connection_params.get(key).cloned()
    }
    
    /// Validate required connection parameters
    pub fn validate_required_params(config: &VectorStoreConfig, required_params: &[&str]) -> Result<(), PlatformError> {
        for param in required_params {
            if !config.connection_params.contains_key(*param) {
                return Err(PlatformError::ValidationError(
                    format!("Missing required connection parameter: {}", param)
                ));
            }
        }
        Ok(())
    }
    
    /// Create HTTP client with provider-specific configuration
    pub fn create_http_client(
        config: &VectorStoreConfig,
        additional_headers: HashMap<String, String>,
    ) -> Result<VectorHttpClient, PlatformError> {
        let mut http_config = HttpClientConfig::default();
        http_config.timeout = Duration::from_secs(config.timeout_seconds);
        http_config.default_headers.extend(additional_headers);
        
        VectorHttpClient::new(http_config)
    }
}

/// Standard response format for vector operations
#[derive(Debug, Serialize, Deserialize)]
pub struct StandardVectorResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Standard error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct StandardErrorResponse {
    pub error: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}