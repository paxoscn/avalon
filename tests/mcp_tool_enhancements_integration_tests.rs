use serde_json::json;
use std::collections::HashMap;

// Integration tests for MCP Tool Enhancements
// These tests verify the end-to-end functionality of:
// - Path parameters in URLs
// - Header parameters
// - Body parameters
// - Response template rendering
// - MCP Server interfaces

#[cfg(test)]
mod mcp_tool_enhancements_tests {
    use super::*;
    use agent_platform::domain::entities::MCPTool;
    use agent_platform::domain::value_objects::{
        ids::{TenantId, UserId},
        tool_config::{
            HTTPToolConfig, HttpMethod, ParameterPosition, ParameterSchema, ParameterType,
            ToolConfig,
        },
    };
    use agent_platform::infrastructure::mcp::{
        http_converter::HTTPToMCPConverter,
        mcp_protocol::{parameters_to_json_schema, tool_to_mcp_format},
        template_engine::ResponseTemplateEngine,
    };

    #[test]
    fn test_create_tool_with_path_parameters() {
        // Test creating a tool with path parameters
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}/orders/{orderId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
                .with_description("User ID".to_string()),
        )
        .with_parameter(
            ParameterSchema::new("orderId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
                .with_description("Order ID".to_string()),
        );

        // Validate the configuration
        assert!(config.validate().is_ok());

        // Create the tool
        let tool = MCPTool::new(
            TenantId::new(),
            "get-user-order".to_string(),
            Some("Get a specific order for a user".to_string()),
            ToolConfig::HTTP(config),
            UserId::new(),
        );

        assert_eq!(tool.name, "get-user-order");
    }

    #[test]
    fn test_create_tool_with_mixed_parameter_positions() {
        // Test creating a tool with parameters in different positions
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}".to_string(),
            HttpMethod::POST,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        )
        .with_parameter(
            ParameterSchema::new("Authorization".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header),
        )
        .with_parameter(
            ParameterSchema::new("name".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Body),
        )
        .with_parameter(
            ParameterSchema::new("email".to_string(), ParameterType::String, false)
                .with_position(ParameterPosition::Body),
        );

        // Validate the configuration
        assert!(config.validate().is_ok());

        // Validate call parameters
        let params = json!({
            "userId": "123",
            "Authorization": "Bearer token",
            "name": "John Doe",
            "email": "john@example.com"
        });

        assert!(config.validate_call_parameters(&params).is_ok());
    }

    #[test]
    fn test_response_template_rendering() {
        // Test response template rendering
        let template_engine = ResponseTemplateEngine::new();

        let template = r#"Order ID: {{ orderId }}
Status: {{ status }}
Items:
{{#each items}}
- {{ name }}: ${{ price }}
{{/each}}
Total: ${{ total }}"#;

        let response_data = json!({
            "orderId": "ORD-12345",
            "status": "Completed",
            "items": [
                {"name": "Product A", "price": 29.99},
                {"name": "Product B", "price": 49.99}
            ],
            "total": 79.98
        });

        let rendered = template_engine
            .render("test-tool", template, &response_data)
            .unwrap();

        assert!(rendered.contains("Order ID: ORD-12345"));
        assert!(rendered.contains("Status: Completed"));
        assert!(rendered.contains("Product A: $29.99"));
        assert!(rendered.contains("Product B: $49.99"));
        assert!(rendered.contains("Total: $79.98"));
    }

    #[test]
    fn test_mcp_format_conversion() {
        // Test converting a tool to MCP format
        let config = HTTPToolConfig::new(
            "https://api.example.com/search".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("query".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Body)
                .with_description("Search query".to_string()),
        )
        .with_parameter(
            ParameterSchema::new("limit".to_string(), ParameterType::Number, false)
                .with_position(ParameterPosition::Body)
                .with_description("Result limit".to_string())
                .with_default(json!(10)),
        );

        let tool = MCPTool::new(
            TenantId::new(),
            "search-api".to_string(),
            Some("Search API".to_string()),
            ToolConfig::HTTP(config),
            UserId::new(),
        );

        let mcp_descriptor = tool_to_mcp_format(&tool);

        assert_eq!(mcp_descriptor.name, "search-api");
        assert_eq!(mcp_descriptor.description, Some("Search API".to_string()));
        assert_eq!(mcp_descriptor.input_schema["type"], "object");
        assert!(mcp_descriptor.input_schema["properties"]["query"].is_object());
        assert_eq!(
            mcp_descriptor.input_schema["properties"]["query"]["type"],
            "string"
        );
        assert_eq!(
            mcp_descriptor.input_schema["properties"]["limit"]["default"],
            10
        );

        let required = mcp_descriptor.input_schema["required"]
            .as_array()
            .unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "query");
    }

    #[test]
    fn test_json_schema_excludes_path_parameters() {
        // Test that path parameters are excluded from JSON Schema
        let parameters = vec![
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
            ParameterSchema::new("name".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Body),
            ParameterSchema::new("X-API-Key".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header),
        ];

        let schema = parameters_to_json_schema(&parameters);

        // Path parameter should not be in schema
        assert!(schema["properties"]["userId"].is_null());

        // Body and header parameters should be in schema
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["properties"]["X-API-Key"].is_object());
    }

    #[test]
    fn test_error_missing_path_parameter() {
        // Test error when path parameter is missing
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        );

        // Missing userId parameter
        let params = json!({});

        let result = config.validate_call_parameters(&params);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("userId"));
        assert!(error_msg.contains("required") || error_msg.contains("missing"));
    }

    #[test]
    fn test_error_template_syntax() {
        // Test error when template has syntax error
        let template_engine = ResponseTemplateEngine::new();

        let invalid_template = "Hello {{ name }"; // Missing closing brace
        let data = json!({"name": "World"});

        let result = template_engine.render("test-tool", invalid_template, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_path_parameter_mismatch() {
        // Test error when path parameter doesn't match endpoint
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        )
        .with_parameter(
            ParameterSchema::new("orderId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        );

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("orderId"));
    }

    #[test]
    fn test_error_invalid_header_name() {
        // Test error when header parameter has invalid name
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("Invalid Header!".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header),
        );

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Invalid Header!"));
        assert!(error_msg.contains("invalid name"));
    }

    #[test]
    fn test_url_special_characters_encoding() {
        // Test that special characters in path parameters are properly encoded
        // by validating the configuration accepts special characters
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{email}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("email".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        );

        // Configuration should be valid
        assert!(config.validate().is_ok());

        // Parameters with special characters should be accepted
        let params = json!({"email": "user@example.com"});
        assert!(config.validate_call_parameters(&params).is_ok());
    }

    #[test]
    fn test_template_with_conditional() {
        // Test template with conditional logic
        let template_engine = ResponseTemplateEngine::new();

        let template = "Status: {{#if active}}Active{{else}}Inactive{{/if}}";

        let active_data = json!({"active": true});
        let result = template_engine
            .render("test-tool", template, &active_data)
            .unwrap();
        assert_eq!(result, "Status: Active");

        let inactive_data = json!({"active": false});
        let result = template_engine
            .render("test-tool", template, &inactive_data)
            .unwrap();
        assert_eq!(result, "Status: Inactive");
    }

    #[test]
    fn test_template_with_nested_data() {
        // Test template with nested data access
        let template_engine = ResponseTemplateEngine::new();

        let template = "User: {{ user.name }}, Email: {{ user.contact.email }}";

        let data = json!({
            "user": {
                "name": "John Doe",
                "contact": {
                    "email": "john@example.com"
                }
            }
        });

        let result = template_engine.render("test-tool", template, &data).unwrap();
        assert_eq!(result, "User: John Doe, Email: john@example.com");
    }

    #[test]
    fn test_parameter_with_default_value() {
        // Test parameter with default value
        let config = HTTPToolConfig::new(
            "https://api.example.com/search".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("limit".to_string(), ParameterType::Number, false)
                .with_position(ParameterPosition::Body)
                .with_default(json!(10)),
        );

        // Call without providing the optional parameter
        let params = json!({});

        // Should use default value
        assert!(config.validate_call_parameters(&params).is_ok());
    }

    #[test]
    fn test_parameter_with_enum_values() {
        // Test parameter with enum values
        let config = HTTPToolConfig::new(
            "https://api.example.com/users".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("status".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Body)
                .with_enum_values(vec![json!("active"), json!("inactive"), json!("pending")]),
        );

        // Valid enum value
        let valid_params = json!({"status": "active"});
        assert!(config.validate_call_parameters(&valid_params).is_ok());

        // Invalid enum value
        let invalid_params = json!({"status": "unknown"});
        assert!(config.validate_call_parameters(&invalid_params).is_err());
    }

    #[test]
    fn test_multiple_path_parameters() {
        // Test configuration with multiple path parameters
        let config = HTTPToolConfig::new(
            "https://api.example.com/tenants/{tenantId}/users/{userId}/orders/{orderId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("tenantId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        )
        .with_parameter(
            ParameterSchema::new("orderId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path),
        );

        // Configuration should be valid
        assert!(config.validate().is_ok());

        // All path parameters should be provided
        let params = json!({
            "tenantId": "tenant-123",
            "userId": "user-456",
            "orderId": "order-789"
        });
        assert!(config.validate_call_parameters(&params).is_ok());
    }

    #[test]
    fn test_template_cache_functionality() {
        // Test that template caching works correctly
        let template_engine = ResponseTemplateEngine::new();
        let template = "Hello {{ name }}!";
        let data = json!({"name": "World"});

        // First render - should compile and cache
        let result1 = template_engine.render("cache-test", template, &data).unwrap();
        assert_eq!(result1, "Hello World!");

        // Second render - should use cache
        let result2 = template_engine.render("cache-test", template, &data).unwrap();
        assert_eq!(result2, "Hello World!");

        // Clear cache
        template_engine.clear_cache("cache-test");

        // Third render - should recompile
        let result3 = template_engine.render("cache-test", template, &data).unwrap();
        assert_eq!(result3, "Hello World!");
    }
}
