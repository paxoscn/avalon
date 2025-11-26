// Integration tests for API Key Management
// These tests verify the end-to-end functionality of:
// - API key CRUD operations
// - Authentication and authorization flows
// - MCP server functionality with API key authentication
//
// Requirements tested:
// - 1.1-1.5: API key creation with permissions and expiration
// - 2.1-2.5: Permission scope configuration
// - 3.1-3.4: Enable/disable functionality
// - 4.1-4.2: Expiration handling
// - 5.1-5.5: Authentication
// - 6.1-6.4: Authorization and resource access
// - 7.1-7.5: Listing and viewing API keys
// - 8.1-8.4: API key deletion
// - 9.1-9.5: MCP server integration
// - 10.1-10.4: MCP tool metadata and schemas
// - 11.1-11.5: Audit logging

use agent_platform::config::AppConfig;
use agent_platform::infrastructure::database;
use agent_platform::infrastructure::RedisCache;
use agent_platform::presentation::server::Server;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sea_orm::{Database, DatabaseConnection};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;
use std::sync::Arc;

#[cfg(test)]
mod test_helpers {
    use super::*;
    use bcrypt::{hash, DEFAULT_COST};
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    use agent_platform::infrastructure::database::entities::{tenant, user, agent, flow, mcp_tool};

    pub struct TestContext {
        pub db: Arc<DatabaseConnection>,
        pub server: Server,
        pub tenant_id: Uuid,
        pub user_id: Uuid,
        pub auth_token: String,
        pub tenant2_id: Uuid,
        pub user2_id: Uuid,
        pub auth_token2: String,
        pub agent_id: Uuid,
        pub flow_id: Uuid,
        pub mcp_tool_id: Uuid,
    }

    impl TestContext {
        pub async fn setup() -> Self {
            // Load test configuration
            let config = AppConfig::load().expect("Failed to load config");
            
            // Connect to test database
            let db = Database::connect(&config.database_url)
                .await
                .expect("Failed to connect to test database");

            let database = database::Database::new(&config.database_url)
                .await
                .expect("Failed to load database");
            let cache = RedisCache::new(&config.redis_url)
                .await
                .expect("Failed to load redis");
            let server = Server::new(config, Arc::new(database), Arc::new(cache));

            // Create test tenant 1
            let tenant_id = Uuid::new_v4();
            let tenant_model = tenant::ActiveModel {
                id: Set(tenant_id),
                name: Set("Test Tenant 1".to_string()),
                ..Default::default()
            };
            tenant_model.insert(&db).await.expect("Failed to create test tenant");

            // Create test user 1
            let user_id = Uuid::new_v4();
            let password_hash = hash("password123", DEFAULT_COST).expect("Failed to hash password");
            let user_model = user::ActiveModel {
                id: Set(user_id),
                tenant_id: Set(tenant_id),
                username: Set("testuser1".to_string()),
                nickname: Set(Some("Test User 1".to_string())),
                password_hash: Set(password_hash),
                ..Default::default()
            };
            user_model.insert(&db).await.expect("Failed to create test user");

            // Create test tenant 2 (for multi-tenant isolation tests)
            let tenant2_id = Uuid::new_v4();
            let tenant2_model = tenant::ActiveModel {
                id: Set(tenant2_id),
                name: Set("Test Tenant 2".to_string()),
                ..Default::default()
            };
            tenant2_model.insert(&db).await.expect("Failed to create test tenant 2");

            // Create test user 2
            let user2_id = Uuid::new_v4();
            let password_hash2 = hash("password456", DEFAULT_COST).expect("Failed to hash password");
            let user2_model = user::ActiveModel {
                id: Set(user2_id),
                tenant_id: Set(tenant2_id),
                username: Set("testuser2".to_string()),
                nickname: Set(Some("Test User 2".to_string())),
                password_hash: Set(password_hash2),
                ..Default::default()
            };
            user2_model.insert(&db).await.expect("Failed to create test user 2");

            // Create test agent for permission testing
            let agent_id = Uuid::new_v4();
            let agent_model = agent::ActiveModel {
                id: Set(agent_id),
                tenant_id: Set(tenant_id),
                name: Set("Test Agent".to_string()),
                system_prompt: Set("Test system prompt".to_string()),
                creator_id: Set(user_id),
                ..Default::default()
            };
            agent_model.insert(&db).await.expect("Failed to create test agent");

            // Create test flow for permission testing
            let flow_id = Uuid::new_v4();
            let flow_model = flow::ActiveModel {
                id: Set(flow_id),
                tenant_id: Set(tenant_id),
                name: Set("Test Flow".to_string()),
                description: Set(Some("Test flow for API key tests".to_string())),
                created_by: Set(user_id),
                ..Default::default()
            };
            flow_model.insert(&db).await.expect("Failed to create test flow");

            // Create test MCP tool for permission testing
            let mcp_tool_id = Uuid::new_v4();
            let mcp_tool_model = mcp_tool::ActiveModel {
                id: Set(mcp_tool_id),
                tenant_id: Set(tenant_id),
                name: Set("test-tool".to_string()),
                description: Set(Some("Test MCP tool".to_string())),
                created_by: Set(user_id),
                ..Default::default()
            };
            mcp_tool_model.insert(&db).await.expect("Failed to create test MCP tool");

            // Generate auth tokens (simplified - in real implementation, call login endpoint)
            let auth_token = format!("test_token_{}", user_id);
            let auth_token2 = format!("test_token_{}", user2_id);

            Self {
                db: Arc::new(db),
                server,
                tenant_id,
                user_id,
                auth_token,
                tenant2_id,
                user2_id,
                auth_token2,
                agent_id,
                flow_id,
                mcp_tool_id,
            }
        }

        pub async fn cleanup(&self) {
            // Clean up test data in reverse order of creation
            use agent_platform::infrastructure::database::entities::api_key;
            let _ = api_key::Entity::delete_many().exec(self.db.as_ref()).await;
            let _ = mcp_tool::Entity::delete_many().exec(self.db.as_ref()).await;
            let _ = flow::Entity::delete_many().exec(self.db.as_ref()).await;
            let _ = agent::Entity::delete_many().exec(self.db.as_ref()).await;
            let _ = user::Entity::delete_many().exec(self.db.as_ref()).await;
            let _ = tenant::Entity::delete_many().exec(self.db.as_ref()).await;
        }
    }

    pub async fn make_request(
        app: &axum::Router,
        method: &str,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path);

        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let request = if let Some(body) = body {
            request
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap()
        } else {
            request.body(Body::empty()).unwrap()
        };

        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        
        let json: Value = if body.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body).unwrap_or(json!({}))
        };

        (status, json)
    }
}

#[cfg(test)]
mod api_key_crud_tests {
    use super::*;
    use test_helpers::*;
    use chrono::{Duration, Utc};

    #[tokio::test]
    #[ignore] // Remove when test database is configured
    async fn test_create_api_key_and_verify_token_returned_once() {
        // Requirement 1.1, 1.2, 1.5: Create API key and verify token returned once
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Test API Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        assert!(response["id"].is_string());
        assert!(response["token"].is_string());
        assert_eq!(response["name"], "Test API Key");
        assert!(response["token"].as_str().unwrap().starts_with("pk_"));

        let api_key_id = response["id"].as_str().unwrap();
        let _token = response["token"].as_str().unwrap().to_string();

        // Get the API key details - token should NOT be returned
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["token"].is_null());
        assert_eq!(response["name"], "Test API Key");

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_api_keys_with_pagination() {
        // Requirement 7.1, 7.2, 7.4: List API keys with pagination
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create multiple API keys
        for i in 1..=5 {
            let _ = make_request(
                &app,
                "POST",
                "/api/v1/api-keys",
                Some(&ctx.auth_token),
                Some(json!({
                    "name": format!("API Key {}", i),
                    "permission_scope": {
                        "agent_ids": [ctx.agent_id],
                        "flow_ids": [],
                        "mcp_tool_ids": [],
                        "vector_store_ids": []
                    },
                    "expires_at": null
                })),
            ).await;
        }

        // List with pagination
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/v1/api-keys?offset=0&limit=3",
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["items"].is_array());
        let items = response["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(response["total"].as_u64().unwrap(), 5);
        assert_eq!(response["offset"].as_u64().unwrap(), 0);
        assert_eq!(response["limit"].as_u64().unwrap(), 3);

        // Verify tokens are not included in list
        for item in items {
            assert!(item["token"].is_null());
        }

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_api_keys_with_filtering() {
        // Requirement 7.5: Filter API keys by enabled status
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create enabled API key
        let (_, _response1) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Enabled Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        // Create and disable another API key
        let (_, response2) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Disabled Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let disabled_key_id = response2["id"].as_str().unwrap();
        let _ = make_request(
            &app,
            "PATCH",
            &format!("/api/v1/api-keys/{}", disabled_key_id),
            Some(&ctx.auth_token),
            Some(json!({"enabled": false})),
        ).await;

        // Filter for enabled keys only
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/v1/api-keys?enabled=true",
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let items = response["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["name"], "Enabled Key");

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_single_api_key_details() {
        // Requirement 7.2, 7.3: Get single API key details without token
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Detail Test Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [ctx.flow_id],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();

        // Get the API key details
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["id"], api_key_id);
        assert_eq!(response["name"], "Detail Test Key");
        assert!(response["token"].is_null()); // Token should not be returned
        assert!(response["enabled"].as_bool().unwrap());
        assert!(response["permission_scope"]["agent_ids"].as_array().unwrap().contains(&json!(ctx.agent_id)));
        assert!(response["permission_scope"]["flow_ids"].as_array().unwrap().contains(&json!(ctx.flow_id)));

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_api_key_properties() {
        // Requirement 1.3, 3.1, 3.2: Update API key properties
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Original Name",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();

        // Update name
        let (status, response) = make_request(
            &app,
            "PATCH",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            Some(json!({"name": "Updated Name"})),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["name"], "Updated Name");

        // Update enabled status
        let (status, response) = make_request(
            &app,
            "PATCH",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            Some(json!({"enabled": false})),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["enabled"], false);

        // Update expiration
        let future_date = Utc::now() + Duration::days(30);
        let (status, response) = make_request(
            &app,
            "PATCH",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            Some(json!({"expires_at": future_date.to_rfc3339()})),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["expires_at"].is_string());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_api_key_and_verify_access_denied() {
        // Requirement 8.1, 8.2, 8.3: Delete API key and verify subsequent access denied
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "To Be Deleted",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();
        let token = create_response["token"].as_str().unwrap();

        // Verify the key exists
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            None,
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Delete the API key
        let (status, _) = make_request(
            &app,
            "DELETE",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            None,
        ).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Verify the key no longer exists
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            None,
        ).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Verify authentication with deleted key fails
        let (status, _) = make_request(
            &app,
            "GET",
            "/api/v1/agents",
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        ctx.cleanup().await;
    }
}


#[cfg(test)]
mod authentication_authorization_tests {
    use super::*;
    use test_helpers::*;
    use chrono::{Duration, Utc};

    #[tokio::test]
    #[ignore]
    async fn test_successful_authentication_with_valid_api_key() {
        // Requirement 5.1, 5.2, 5.3, 5.4: Successful authentication with valid API key
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Valid Auth Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Use the API key to authenticate and access a resource
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["id"], ctx.agent_id.to_string());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_authentication_failure_with_invalid_token() {
        // Requirement 5.2, 5.5: Authentication failure with invalid token
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Try to authenticate with an invalid token
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some("pk_invalid_token_12345"),
            None,
        ).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_authentication_failure_with_expired_key() {
        // Requirement 4.1, 4.2, 5.4: Authentication failure with expired key
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key that expires in 1 second
        let expires_at = Utc::now() + Duration::seconds(1);
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Expiring Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": expires_at.to_rfc3339()
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Wait for the key to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Try to authenticate with the expired key
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_authentication_failure_with_disabled_key() {
        // Requirement 3.3, 5.3: Authentication failure with disabled key
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "To Be Disabled",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();
        let token = create_response["token"].as_str().unwrap();

        // Verify authentication works initially
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Disable the API key
        let (status, _) = make_request(
            &app,
            "PATCH",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            Some(json!({"enabled": false})),
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Try to authenticate with the disabled key
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_authorization_success_for_permitted_resources() {
        // Requirement 6.1, 6.3: Authorization success for permitted resources
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key with access to specific resources
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Multi-Resource Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [ctx.flow_id],
                    "mcp_tool_ids": [ctx.mcp_tool_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Access agent - should succeed
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Access flow - should succeed
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/flows/{}", ctx.flow_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Access MCP tool - should succeed
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/mcp-tools/{}", ctx.mcp_tool_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::OK);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_authorization_failure_for_non_permitted_resources() {
        // Requirement 6.2: Authorization failure for non-permitted resources
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key with access to only one agent
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Limited Access Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Try to access a flow (not in permission scope)
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/flows/{}", ctx.flow_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // Try to access an MCP tool (not in permission scope)
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/mcp-tools/{}", ctx.mcp_tool_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_tenant_isolation_enforcement() {
        // Requirement 6.4: Test tenant isolation enforcement
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key for tenant 1
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Tenant 1 Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Create an agent in tenant 2
        let agent2_id = Uuid::new_v4();
        use agent_platform::infrastructure::database::entities::agent;
        use sea_orm::{ActiveModelTrait, Set};
        let agent2_model = agent::ActiveModel {
            id: Set(agent2_id),
            tenant_id: Set(ctx.tenant2_id),
            name: Set("Tenant 2 Agent".to_string()),
            system_prompt: Set("Test system prompt".to_string()),
            creator_id: Set(ctx.user2_id),
            ..Default::default()
        };
        agent2_model.insert(ctx.db.as_ref()).await.expect("Failed to create tenant 2 agent");

        // Try to access tenant 2's agent with tenant 1's API key
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", agent2_id),
            Some(token),
            None,
        ).await;

        // Should be forbidden or not found (tenant isolation)
        assert!(status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_authorization_with_empty_permission_scope() {
        // Requirement 2.5: Empty permission scope denies all access
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Try to create an API key with empty permission scope (should fail validation)
        let (status, _) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Empty Scope Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        // Should fail validation
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

        ctx.cleanup().await;
    }
}


#[cfg(test)]
mod mcp_server_tests {
    use super::*;
    use test_helpers::*;

    #[tokio::test]
    #[ignore]
    async fn test_list_tools_filtered_by_api_key_permissions() {
        // Requirement 9.3, 10.1, 10.2: List tools filtered by API key permissions
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create additional MCP tools
        use agent_platform::infrastructure::database::entities::mcp_tool;
        use sea_orm::{ActiveModelTrait, Set};
        
        let tool2_id = Uuid::new_v4();
        let tool2_model = mcp_tool::ActiveModel {
            id: Set(tool2_id),
            tenant_id: Set(ctx.tenant_id),
            name: Set("test-tool-2".to_string()),
            description: Set(Some("Second test tool".to_string())),
            created_by: Set(ctx.user_id),
            ..Default::default()
        };
        tool2_model.insert(ctx.db.as_ref()).await.expect("Failed to create tool 2");

        let tool3_id = Uuid::new_v4();
        let tool3_model = mcp_tool::ActiveModel {
            id: Set(tool3_id),
            tenant_id: Set(ctx.tenant_id),
            name: Set("test-tool-3".to_string()),
            description: Set(Some("Third test tool".to_string())),
            created_by: Set(ctx.user_id),
            ..Default::default()
        };
        tool3_model.insert(ctx.db.as_ref()).await.expect("Failed to create tool 3");

        // Create an API key with access to only tool 1 and tool 2
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "MCP Tools Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [ctx.mcp_tool_id, tool2_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // List MCP tools using the API key
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/v1/mcp-tools",
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let tools = response["tools"].as_array().unwrap();
        
        // Should only see tools 1 and 2, not tool 3
        assert_eq!(tools.len(), 2);
        let tool_ids: Vec<String> = tools.iter()
            .map(|t| t["id"].as_str().unwrap().to_string())
            .collect();
        assert!(tool_ids.contains(&ctx.mcp_tool_id.to_string()));
        assert!(tool_ids.contains(&tool2_id.to_string()));
        assert!(!tool_ids.contains(&tool3_id.to_string()));

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_tool_invocation_with_valid_permissions() {
        // Requirement 9.4, 9.5: Tool invocation with valid permissions
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key with access to the MCP tool
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Tool Invocation Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [ctx.mcp_tool_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Invoke the MCP tool
        let (status, _response) = make_request(
            &app,
            "POST",
            &format!("/api/v1/mcp-tools/{}/invoke", ctx.mcp_tool_id),
            Some(token),
            Some(json!({
                "parameters": {
                    "query": "test"
                }
            })),
        ).await;

        // Should succeed (or return appropriate error if tool execution fails)
        assert!(status == StatusCode::OK || status == StatusCode::INTERNAL_SERVER_ERROR);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_tool_invocation_denial_without_permissions() {
        // Requirement 9.4: Tool invocation denial without permissions
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key WITHOUT access to the MCP tool
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "No Tool Access Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Try to invoke the MCP tool
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/v1/mcp-tools/{}/invoke", ctx.mcp_tool_id),
            Some(token),
            Some(json!({
                "parameters": {
                    "query": "test"
                }
            })),
        ).await;

        // Should be forbidden
        assert_eq!(status, StatusCode::FORBIDDEN);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_mcp_protocol_error_handling() {
        // Requirement 9.5: MCP protocol error handling
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key with access to the MCP tool
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Error Test Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [ctx.mcp_tool_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Try to invoke with invalid parameters
        let (status, _response) = make_request(
            &app,
            "POST",
            &format!("/api/v1/mcp-tools/{}/invoke", ctx.mcp_tool_id),
            Some(token),
            Some(json!({
                "parameters": "invalid"
            })),
        ).await;

        // Should return an error (validation or bad request)
        assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_api_key_authentication_in_mcp_connections() {
        // Requirement 9.2: API key authentication in MCP connections
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "MCP Connection Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [ctx.mcp_tool_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Try to access MCP tools without authentication
        let (status, _) = make_request(
            &app,
            "GET",
            "/api/v1/mcp-tools",
            None,
            None,
        ).await;

        // Should require authentication
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        // Access with valid API key
        let (status, _) = make_request(
            &app,
            "GET",
            "/api/v1/mcp-tools",
            Some(token),
            None,
        ).await;

        // Should succeed
        assert_eq!(status, StatusCode::OK);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_tool_metadata_and_schemas() {
        // Requirement 10.1, 10.3, 10.4: Tool metadata and schemas
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key with access to the MCP tool
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Schema Test Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [ctx.mcp_tool_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Get tool details including schema
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/mcp-tools/{}", ctx.mcp_tool_id),
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["id"], ctx.mcp_tool_id.to_string());
        assert_eq!(response["name"], "test-tool");
        assert!(response["description"].is_string());
        assert!(response["config"].is_object());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_tool_not_accessible_excluded_from_list() {
        // Requirement 10.4: Tools not accessible via API key are excluded from list
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create additional MCP tool
        use agent_platform::infrastructure::database::entities::mcp_tool;
        use sea_orm::{ActiveModelTrait, Set};
        
        let tool2_id = Uuid::new_v4();
        let tool2_model = mcp_tool::ActiveModel {
            id: Set(tool2_id),
            tenant_id: Set(ctx.tenant_id),
            name: Set("inaccessible-tool".to_string()),
            description: Set(Some("Tool not in permission scope".to_string())),
            created_by: Set(ctx.user_id),
            ..Default::default()
        };
        tool2_model.insert(ctx.db.as_ref()).await.expect("Failed to create tool 2");

        // Create an API key with access to only the first tool
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Single Tool Key",
                "permission_scope": {
                    "agent_ids": [],
                    "flow_ids": [],
                    "mcp_tool_ids": [ctx.mcp_tool_id],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // List MCP tools
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/v1/mcp-tools",
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let tools = response["tools"].as_array().unwrap();
        
        // Should only see the first tool
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["id"], ctx.mcp_tool_id.to_string());

        // Try to access the inaccessible tool directly
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/mcp-tools/{}", tool2_id),
            Some(token),
            None,
        ).await;

        // Should be forbidden
        assert_eq!(status, StatusCode::FORBIDDEN);

        ctx.cleanup().await;
    }
}


#[cfg(test)]
mod audit_logging_tests {
    use super::*;
    use test_helpers::*;

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_api_key_creation() {
        // Requirement 11.1: Log API key creation events
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (status, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Audit Test Key",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let api_key_id = create_response["id"].as_str().unwrap();

        // Query audit logs
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&resource_type=api_key", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Find the creation log
        let creation_log = logs.iter().find(|log| {
            log["action"].as_str() == Some("create") &&
            log["resource_id"].as_str() == Some(api_key_id)
        });

        assert!(creation_log.is_some());
        let log = creation_log.unwrap();
        assert_eq!(log["user_id"], ctx.user_id.to_string());
        assert_eq!(log["tenant_id"], ctx.tenant_id.to_string());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_api_key_state_changes() {
        // Requirement 11.3: Log API key state changes
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "State Change Test",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();

        // Disable the API key
        let (status, _) = make_request(
            &app,
            "PATCH",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            Some(json!({"enabled": false})),
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Query audit logs for update action
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&resource_type=api_key&action=update", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Find the update log
        let update_log = logs.iter().find(|log| {
            log["resource_id"].as_str() == Some(api_key_id)
        });

        assert!(update_log.is_some());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_authentication_attempts() {
        // Requirement 11.2: Log authentication attempts
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Auth Audit Test",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Use the API key to authenticate
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/agents/{}", ctx.agent_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::OK);

        // Query audit logs for authentication
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&action=api_key_auth", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Should have at least one authentication log
        assert!(!logs.is_empty());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_authorization_failures() {
        // Requirement 11.4: Log authorization failures
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key without access to flows
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Authz Failure Test",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();

        // Try to access a flow (should fail authorization)
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/v1/flows/{}", ctx.flow_id),
            Some(token),
            None,
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // Query audit logs for authorization failures
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&action=api_key_authz_failed", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Should have at least one authorization failure log
        assert!(!logs.is_empty());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_deletion() {
        // Requirement 11.3: Log API key deletion
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Delete Audit Test",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();

        // Delete the API key
        let (status, _) = make_request(
            &app,
            "DELETE",
            &format!("/api/v1/api-keys/{}", api_key_id),
            Some(&ctx.auth_token),
            None,
        ).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Query audit logs for deletion
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&resource_type=api_key&action=delete", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Find the deletion log
        let deletion_log = logs.iter().find(|log| {
            log["resource_id"].as_str() == Some(api_key_id)
        });

        assert!(deletion_log.is_some());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_includes_timestamps_and_ip() {
        // Requirement 11.5: Include timestamps and IP addresses in audit logs
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Timestamp Test",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let api_key_id = create_response["id"].as_str().unwrap();

        // Query audit logs
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&resource_type=api_key", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Find the creation log
        let creation_log = logs.iter().find(|log| {
            log["action"].as_str() == Some("create") &&
            log["resource_id"].as_str() == Some(api_key_id)
        });

        assert!(creation_log.is_some());
        let log = creation_log.unwrap();
        
        // Verify timestamp is present
        assert!(log["timestamp"].is_string() || log["created_at"].is_string());
        
        // Verify IP address is present (if tracked)
        // Note: IP address tracking may be in metadata or separate field
        assert!(log["ip_address"].is_string() || log["metadata"].is_object());

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_log_never_contains_tokens() {
        // Requirement 11.5: Ensure tokens and hashes are never logged
        let ctx = TestContext::setup().await;
        let app = ctx.server.create_app();

        // Create an API key
        let (_, create_response) = make_request(
            &app,
            "POST",
            "/api/v1/api-keys",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Token Security Test",
                "permission_scope": {
                    "agent_ids": [ctx.agent_id],
                    "flow_ids": [],
                    "mcp_tool_ids": [],
                    "vector_store_ids": []
                },
                "expires_at": null
            })),
        ).await;

        let token = create_response["token"].as_str().unwrap();
        let api_key_id = create_response["id"].as_str().unwrap();

        // Query audit logs
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/v1/audit?tenant_id={}&resource_type=api_key", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let logs = response["logs"].as_array().unwrap();
        
        // Find the creation log
        let creation_log = logs.iter().find(|log| {
            log["action"].as_str() == Some("create") &&
            log["resource_id"].as_str() == Some(api_key_id)
        });

        assert!(creation_log.is_some());
        let log = creation_log.unwrap();
        
        // Convert log to string and verify token is not present
        let log_str = serde_json::to_string(&log).unwrap();
        assert!(!log_str.contains(token));
        assert!(!log_str.contains("pk_")); // Token prefix should not appear

        ctx.cleanup().await;
    }
}
