use crate::domain::value_objects::{ConfigId, TenantId, ModelConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


/// LLM Configuration domain entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LLMConfig {
    pub id: ConfigId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub model_config: ModelConfig,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LLMConfig {
    pub fn new(
        tenant_id: TenantId,
        name: String,
        model_config: ModelConfig,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ConfigId::new(),
            tenant_id,
            name,
            description: None,
            model_config,
            is_default: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn deactivate(mut self) -> Self {
        self.is_active = false;
        self.updated_at = Utc::now();
        self
    }

    pub fn activate(mut self) -> Self {
        self.is_active = true;
        self.updated_at = Utc::now();
        self
    }

    pub fn update_config(mut self, model_config: ModelConfig) -> Result<Self, String> {
        // Validate the new configuration
        model_config.validate()?;
        
        self.model_config = model_config;
        self.updated_at = Utc::now();
        Ok(self)
    }

    pub fn update_name(mut self, name: String) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Configuration name cannot be empty".to_string());
        }
        
        if name.len() > 255 {
            return Err("Configuration name cannot exceed 255 characters".to_string());
        }
        
        self.name = name.trim().to_string();
        self.updated_at = Utc::now();
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Configuration name cannot be empty".to_string());
        }

        if self.name.len() > 255 {
            return Err("Configuration name cannot exceed 255 characters".to_string());
        }

        // Validate the model configuration
        self.model_config.validate()?;

        Ok(())
    }

    pub fn is_usable(&self) -> bool {
        self.is_active && self.validate().is_ok()
    }

    pub fn provider_name(&self) -> String {
        format!("{:?}", self.model_config.provider).to_lowercase()
    }

    pub fn model_name(&self) -> &str {
        &self.model_config.model_name
    }

    pub fn supports_streaming(&self) -> bool {
        self.model_config.supports_streaming()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{ModelProvider, ModelParameters, ModelCredentials};

    fn create_test_model_config() -> ModelConfig {
        ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        }
    }

    #[test]
    fn test_new_llm_config() {
        let tenant_id = TenantId::new();
        let name = "Test Config".to_string();
        let model_config = create_test_model_config();

        let config = LLMConfig::new(tenant_id, name.clone(), model_config.clone());

        assert_eq!(config.tenant_id, tenant_id);
        assert_eq!(config.name, name);
        assert_eq!(config.model_config, model_config);
        assert!(!config.is_default);
        assert!(config.is_active);
        assert!(config.description.is_none());
    }

    #[test]
    fn test_with_description() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        ).with_description("Test description".to_string());

        assert_eq!(config.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_set_as_default() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        ).set_as_default();

        assert!(config.is_default);
    }

    #[test]
    fn test_deactivate_activate() {
        let mut config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        assert!(config.is_active);

        config = config.deactivate();
        assert!(!config.is_active);

        config = config.activate();
        assert!(config.is_active);
    }

    #[test]
    fn test_update_config() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        let mut new_model_config = create_test_model_config();
        new_model_config.model_name = "gpt-4".to_string();

        let updated_config = config.update_config(new_model_config.clone()).unwrap();
        assert_eq!(updated_config.model_config.model_name, "gpt-4");
    }

    #[test]
    fn test_update_name() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        let updated_config = config.update_name("New Name".to_string()).unwrap();
        assert_eq!(updated_config.name, "New Name");

        // Test empty name
        let result = updated_config.clone().update_name("".to_string());
        assert!(result.is_err());

        // Test too long name
        let long_name = "a".repeat(256);
        let result = updated_config.update_name(long_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_is_usable() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        assert!(config.is_usable());

        let deactivated_config = config.deactivate();
        assert!(!deactivated_config.is_usable());
    }

    #[test]
    fn test_provider_name() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        assert_eq!(config.provider_name(), "openai");
    }

    #[test]
    fn test_model_name() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        assert_eq!(config.model_name(), "gpt-3.5-turbo");
    }

    #[test]
    fn test_supports_streaming() {
        let config = LLMConfig::new(
            TenantId::new(),
            "Test".to_string(),
            create_test_model_config(),
        );

        assert!(config.supports_streaming());
    }
}