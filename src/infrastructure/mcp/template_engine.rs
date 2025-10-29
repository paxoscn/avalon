use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Template engine errors
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template syntax error: {0}")]
    SyntaxError(String),
    
    #[error("Template render error: {0}")]
    RenderError(String),
    
    #[error("Template compilation error: {0}")]
    CompilationError(String),
}

impl From<handlebars::RenderError> for TemplateError {
    fn from(error: handlebars::RenderError) -> Self {
        TemplateError::RenderError(error.to_string())
    }
}

impl From<handlebars::TemplateError> for TemplateError {
    fn from(error: handlebars::TemplateError) -> Self {
        TemplateError::CompilationError(error.to_string())
    }
}

/// Response template engine for transforming JSON responses to text
#[derive(Clone)]
pub struct ResponseTemplateEngine {
    handlebars: Arc<RwLock<Handlebars<'static>>>,
    template_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl ResponseTemplateEngine {
    /// Create a new template engine instance
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        
        // Configure handlebars for security and performance
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(handlebars::no_escape);
        
        Self {
            handlebars: Arc::new(RwLock::new(handlebars)),
            template_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Render a template with the given data
    /// 
    /// # Arguments
    /// * `tool_id` - Unique identifier for the tool (used for caching)
    /// * `template` - Template string
    /// * `data` - JSON data to render
    /// 
    /// # Returns
    /// Rendered text or error
    pub fn render(
        &self,
        tool_id: &str,
        template: &str,
        data: &Value,
    ) -> Result<String, TemplateError> {
        // Check if template is already compiled and cached
        let cache_key = format!("{}:{}", tool_id, Self::hash_template(template));
        
        {
            let cache = self.template_cache.read().unwrap();
            if cache.contains_key(&cache_key) {
                // Template is cached, render directly
                let hb = self.handlebars.read().unwrap();
                return hb.render_template(template, data)
                    .map_err(TemplateError::from);
            }
        }
        
        // Template not cached, compile and cache it
        self.compile_template(&cache_key, template)?;
        
        // Render the template
        let hb = self.handlebars.read().unwrap();
        hb.render_template(template, data)
            .map_err(TemplateError::from)
    }
    
    /// Compile and cache a template
    /// 
    /// # Arguments
    /// * `cache_key` - Cache key for the template
    /// * `template` - Template string to compile
    fn compile_template(
        &self,
        cache_key: &str,
        template: &str,
    ) -> Result<(), TemplateError> {
        // Validate template syntax by attempting to compile
        handlebars::Template::compile(template)
            .map_err(|e| TemplateError::SyntaxError(e.to_string()))?;
        
        // Cache the template
        let mut cache = self.template_cache.write().unwrap();
        cache.insert(cache_key.to_string(), template.to_string());
        
        Ok(())
    }
    
    /// Clear template cache for a specific tool
    /// 
    /// # Arguments
    /// * `tool_id` - Tool identifier whose templates should be cleared
    pub fn clear_cache(&self, tool_id: &str) {
        let mut cache = self.template_cache.write().unwrap();
        cache.retain(|key, _| !key.starts_with(&format!("{}:", tool_id)));
    }
    
    /// Clear all cached templates
    pub fn clear_all_cache(&self) {
        let mut cache = self.template_cache.write().unwrap();
        cache.clear();
    }
    
    /// Validate template syntax without caching
    /// 
    /// # Arguments
    /// * `template` - Template string to validate
    /// 
    /// # Returns
    /// Ok if template is valid, error otherwise
    pub fn validate_template(&self, template: &str) -> Result<(), TemplateError> {
        handlebars::Template::compile(template)
            .map_err(|e| TemplateError::SyntaxError(e.to_string()))?;
        Ok(())
    }
    
    /// Generate a simple hash for template content
    fn hash_template(template: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        template.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for ResponseTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_variable_substitution() {
        let engine = ResponseTemplateEngine::new();
        let template = "Hello {{ name }}!";
        let data = json!({
            "name": "World"
        });
        
        let result = engine.render("test-tool", template, &data).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_nested_variable_access() {
        let engine = ResponseTemplateEngine::new();
        let template = "User: {{ user.name }}, Email: {{ user.email }}";
        let data = json!({
            "user": {
                "name": "John Doe",
                "email": "john@example.com"
            }
        });
        
        let result = engine.render("test-tool", template, &data).unwrap();
        assert_eq!(result, "User: John Doe, Email: john@example.com");
    }
    
    #[test]
    fn test_loop_rendering() {
        let engine = ResponseTemplateEngine::new();
        let template = "Items:\n{{#each items}}{{@index}}. {{this.name}}: ${{this.price}}\n{{/each}}";
        let data = json!({
            "items": [
                {"name": "Apple", "price": 1.5},
                {"name": "Banana", "price": 0.8}
            ]
        });
        
        let result = engine.render("test-tool", template, &data).unwrap();
        assert_eq!(result, "Items:\n0. Apple: $1.5\n1. Banana: $0.8\n");
    }
    
    #[test]
    fn test_conditional_rendering() {
        let engine = ResponseTemplateEngine::new();
        let template = "{{#if active}}Status: Active{{else}}Status: Inactive{{/if}}";
        
        let data_active = json!({"active": true});
        let result = engine.render("test-tool", template, &data_active).unwrap();
        assert_eq!(result, "Status: Active");
        
        let data_inactive = json!({"active": false});
        let result = engine.render("test-tool", template, &data_inactive).unwrap();
        assert_eq!(result, "Status: Inactive");
    }
    
    #[test]
    fn test_template_caching() {
        let engine = ResponseTemplateEngine::new();
        let template = "Hello {{ name }}!";
        let data = json!({"name": "World"});
        
        // First render - should compile and cache
        let result1 = engine.render("test-tool", template, &data).unwrap();
        assert_eq!(result1, "Hello World!");
        
        // Second render - should use cache
        let result2 = engine.render("test-tool", template, &data).unwrap();
        assert_eq!(result2, "Hello World!");
        
        // Verify cache contains the template
        let cache = engine.template_cache.read().unwrap();
        assert!(!cache.is_empty());
    }
    
    #[test]
    fn test_clear_cache() {
        let engine = ResponseTemplateEngine::new();
        let template = "Hello {{ name }}!";
        let data = json!({"name": "World"});
        
        // Render to populate cache
        engine.render("test-tool", template, &data).unwrap();
        
        // Clear cache for specific tool
        engine.clear_cache("test-tool");
        
        // Cache should be empty
        let cache = engine.template_cache.read().unwrap();
        assert!(cache.is_empty());
    }
    
    #[test]
    fn test_template_syntax_error() {
        let engine = ResponseTemplateEngine::new();
        let template = "Hello {{ name }"; // Missing closing brace
        let data = json!({"name": "World"});
        
        let result = engine.render("test-tool", template, &data);
        assert!(result.is_err());
        
        match result {
            Err(TemplateError::SyntaxError(_)) => {},
            _ => panic!("Expected SyntaxError"),
        }
    }
    
    #[test]
    fn test_validate_template() {
        let engine = ResponseTemplateEngine::new();
        
        // Valid template
        let valid_template = "Hello {{ name }}!";
        assert!(engine.validate_template(valid_template).is_ok());
        
        // Invalid template
        let invalid_template = "Hello {{ name }";
        assert!(engine.validate_template(invalid_template).is_err());
    }
    
    #[test]
    fn test_complex_template() {
        let engine = ResponseTemplateEngine::new();
        let template = r#"Order ID: {{ orderId }}
Status: {{ status }}
Items:
{{#each items}}
- {{ name }}: ${{ price }}
{{/each}}
Total: ${{ total }}"#;
        
        let data = json!({
            "orderId": "12345",
            "status": "Completed",
            "items": [
                {"name": "Product A", "price": 29.99},
                {"name": "Product B", "price": 49.99}
            ],
            "total": 79.98
        });
        
        let result = engine.render("test-tool", template, &data).unwrap();
        assert!(result.contains("Order ID: 12345"));
        assert!(result.contains("Status: Completed"));
        assert!(result.contains("Product A: $29.99"));
        assert!(result.contains("Total: $79.98"));
    }
    
    #[test]
    fn test_rendering_performance() {
        let engine = ResponseTemplateEngine::new();
        let template = "Hello {{ name }}, you have {{ count }} messages.";
        let data = json!({
            "name": "User",
            "count": 5
        });
        
        // Warm up the cache
        engine.render("perf-test", template, &data).unwrap();
        
        // Measure rendering time
        let start = std::time::Instant::now();
        let _result = engine.render("perf-test", template, &data).unwrap();
        let duration = start.elapsed();
        
        // Should render in less than 1ms
        assert!(duration.as_micros() < 1000, 
            "Template rendering took {}μs, expected < 1000μs", 
            duration.as_micros());
    }
}
