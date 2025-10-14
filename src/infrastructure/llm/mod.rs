pub mod providers;
pub mod error_handling;
pub mod streaming;

pub use providers::*;
pub use error_handling::*;


use crate::domain::services::llm_service::{LLMProvider, LLMError, ConnectionTestResult};
use std::collections::HashMap;
use std::sync::Arc;

/// LLM Provider Registry for managing multiple providers
pub struct LLMProviderRegistry {
    providers: HashMap<String, Arc<dyn LLMProvider>>,
}

impl LLMProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register_provider(&mut self, name: String, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(name, provider);
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn LLMProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub async fn test_all_connections(&self) -> HashMap<String, ConnectionTestResult> {
        let mut results = HashMap::new();
        
        for (name, provider) in &self.providers {
            let result = provider.test_connection().await;
            match result {
                Ok(test_result) => {
                    results.insert(name.clone(), test_result);
                }
                Err(e) => {
                    results.insert(name.clone(), ConnectionTestResult {
                        success: false,
                        response_time_ms: 0,
                        error_message: Some(e.to_string()),
                        model_info: None,
                    });
                }
            }
        }
        
        results
    }
}

impl Default for LLMProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating LLM providers
pub struct LLMProviderFactory;

impl LLMProviderFactory {
    pub fn create_openai_provider(api_key: String, base_url: Option<String>) -> Result<Arc<dyn LLMProvider>, LLMError> {
        Ok(Arc::new(OpenAIProvider::new(api_key, base_url)?))
    }

    pub fn create_claude_provider(api_key: String) -> Result<Arc<dyn LLMProvider>, LLMError> {
        Ok(Arc::new(ClaudeProvider::new(api_key)?))
    }

    pub fn create_local_llm_provider(base_url: String) -> Result<Arc<dyn LLMProvider>, LLMError> {
        Ok(Arc::new(LocalLLMProvider::new(base_url)?))
    }

    pub fn create_registry_with_defaults() -> LLMProviderRegistry {
        let mut registry = LLMProviderRegistry::new();
        
        // Add default providers if environment variables are set
        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            if let Ok(provider) = Self::create_openai_provider(openai_key, None) {
                registry.register_provider("openai".to_string(), provider);
            }
        }

        if let Ok(claude_key) = std::env::var("ANTHROPIC_API_KEY") {
            if let Ok(provider) = Self::create_claude_provider(claude_key) {
                registry.register_provider("claude".to_string(), provider);
            }
        }

        if let Ok(local_url) = std::env::var("LOCAL_LLM_URL") {
            if let Ok(provider) = Self::create_local_llm_provider(local_url) {
                registry.register_provider("local_llm".to_string(), provider);
            }
        }

        registry
    }
}