use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::PlatformError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: ModelProvider,
    pub model_name: String,
    pub parameters: ModelParameters,
    pub credentials: ModelCredentials,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    OpenAI,
    Claude,
    LocalLLM,
    Ollama,
    HuggingFace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCredentials {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub organization: Option<String>,
    pub custom_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorConfig {
    pub provider: VectorProvider,
    pub connection_config: VectorConnectionConfig,
    pub index_config: VectorIndexConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorProvider {
    Pinecone,
    Weaviate,
    ChromaDB,
    Qdrant,
    Milvus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorConnectionConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub custom_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    pub index_name: String,
    pub dimension: u32,
    pub metric: VectorMetric,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

impl ModelConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.model_name.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        if let Some(temperature) = self.parameters.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err("Temperature must be between 0.0 and 2.0".to_string());
            }
        }

        if let Some(top_p) = self.parameters.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err("Top-p must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(max_tokens) = self.parameters.max_tokens {
            if max_tokens == 0 {
                return Err("Max tokens must be greater than 0".to_string());
            }
        }

        Ok(())
    }

    pub fn supports_streaming(&self) -> bool {
        matches!(self.provider, ModelProvider::OpenAI | ModelProvider::Claude | ModelProvider::LocalLLM)
    }
}

impl VectorConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.connection_config.endpoint.trim().is_empty() {
            return Err("Vector database endpoint cannot be empty".to_string());
        }

        if self.index_config.index_name.trim().is_empty() {
            return Err("Index name cannot be empty".to_string());
        }

        if self.index_config.dimension == 0 {
            return Err("Vector dimension must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl Default for ModelParameters {
    fn default() -> Self {
        ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stop_sequences: None,
            custom_parameters: HashMap::new(),
        }
    }
}

impl Default for ModelCredentials {
    fn default() -> Self {
        ModelCredentials {
            api_key: None,
            api_base: None,
            organization: None,
            custom_headers: HashMap::new(),
        }
    }
}

impl ModelProvider {
    pub fn parse_model_provider(provider: &str) -> Result<ModelProvider, PlatformError> {
        match provider.to_lowercase().as_str() {
            "openai" => Ok(ModelProvider::OpenAI),
            "claude" | "anthropic" => Ok(ModelProvider::Claude),
            _ => Err(PlatformError::ValidationError(format!(
                "Unknown provider: {}",
                provider
            ))),
        }
    }
}