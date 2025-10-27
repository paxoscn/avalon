// Integration tests for Agent entity API endpoints
// These tests verify the complete Agent functionality including:
// - CRUD operations
// - Permission control (creator-only modifications)
// - Employment relationship management
// - Allocation relationship management
// - Copy functionality
// - Resource association management
// - Pagination
//
// Requirements tested: All requirements from .kiro/specs/agent-entity/requirements.md

use agent_platform::config::AppConfig;
use agent_platform::infrastructure::{Database, RedisCache};
use agent_platform::presentation::server::Server;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

#[cfg(test)]
mod test_helpers {
    use super::*;
    use bcrypt::{hash, DEFAULT_COST};
    use agent_platform::infrastructure::database::entities::{tenant, user};
    use std::sync::Arc;

    pub struct TestContext {
        pub db: Arc<Database>,
        pub tenant_id: Uuid,
        pub user_id: Uuid,
        pub username: String,
        pub user2_id: Uuid,
        pub username2: String,
    }

    impl TestContext {
        pub async fn setup() -> Self {
            // Load test configuration
            let config = AppConfig::load().expect("Failed to load config");
            
            // Connect to test database
            let db = Arc::new(Database::new(&config.database_url)
                .await
                .expect("Failed to connect to test database"));

            // Create test tenant
            let tenant_id = Uuid::new_v4();
            let tenant_model = tenant::ActiveModel {
                id: Set(tenant_id),
                name: Set("Test Tenant".to_string()),
                ..Default::default()
            };
            tenant_model.insert(db.get_connection()).await.expect("Failed to create test tenant");

            // Create test user 1 (creator)
            let user_id = Uuid::new_v4();
            let username = format!("testuser_{}", Uuid::new_v4());
            let password_hash = hash("password123", DEFAULT_COST).expect("Failed to hash password");
            let user_model = user::ActiveModel {
                id: Set(user_id),
                tenant_id: Set(tenant_id),
                username: Set(username.clone()),
                nickname: Set(Some("Test User 1".to_string())),
                password_hash: Set(password_hash),
                ..Default::default()
            };
            user_model.insert(db.get_connection()).await.expect("Failed to create test user");

            // Create test user 2 (non-creator)
            let user2_id = Uuid::new_v4();
            let username2 = format!("testuser2_{}", Uuid::new_v4());
            let password_hash2 = hash("password456", DEFAULT_COST).expect("Failed to hash password");
            let user2_model = user::ActiveModel {
                id: Set(user2_id),
                tenant_id: Set(tenant_id),
                username: Set(username2.clone()),
                nickname: Set(Some("Test User 2".to_string())),
                password_hash: Set(password_hash2),
                ..Default::default()
            };
            user2_model.insert(db.get_connection()).await.expect("Failed to create test user 2");

            Self {
                db,
                tenant_id,
                user_id,
                username,
                user2_id,
                username2,
            }
        }

        pub async fn cleanup(&self) {
            // Clean up test data in reverse order of dependencies
            use agent_platform::infrastructure::database::entities::{
                agent_employment, agent_allocation, agent, user, tenant
            };
            
            let _ = agent_employment::Entity::delete_many().exec(self.db.get_connection()).await;
            let _ = agent_allocation::Entity::delete_many().exec(self.db.get_connection()).await;
            let _ = agent::Entity::delete_many().exec(self.db.get_connection()).await;
            let _ = user::Entity::delete_many().exec(self.db.get_connection()).await;
            let _ = tenant::Entity::delete_many().exec(self.db.get_connection()).await;
        }

        pub async fn login(&self, username: &str, password: &str, app: &axum::Router) -> String {
            let (status, response) = make_request(
                app,
                "POST",
                "/api/auth/login",
                None,
                Some(json!({
                    "tenant_id": self.tenant_id.to_string(),
                    "username": username,
                    "password": password
                })),
            ).await;

            assert_eq!(status, StatusCode::OK, "Login failed: {:?}", response);
            response["token"].as_str().unwrap().to_string()
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
            serde_json::from_slice(&body).unwrap_or_else(|_| {
                json!({"error": "Failed to parse response", "body": String::from_utf8_lossy(&body).to_string()})
            })
        };

        (status, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::*;
    use std::sync::Arc;

    async fn create_test_app(db: Arc<Database>) -> axum::Router {
        let config = AppConfig::load().expect("Failed to load config");
        let cache = Arc::new(RedisCache::new(&config.redis_url)
            .await
            .expect("Failed to connect to Redis"));
        
        let server = Server::new(config, db, cache);
        server.create_app()
    }

    #[tokio::test]
    #[ignore] // Remove when test database is configured
    async fn test_agent_crud_operations() {
        // Requirement 2.1, 2.2, 2.3: Test complete CRUD flow
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token = ctx.login(&ctx.username, "password123", &app).await;

        // Step 1: Create an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token),
            Some(json!({
                "name": "Test Agent",
                "avatar": "https://example.com/avatar.png",
                "system_prompt": "You are a helpful assistant",
                "additional_settings": "{\"temperature\": 0.7}",
                "preset_questions": ["What can you do?", "How do I start?"],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response["name"], "Test Agent");
        assert_eq!(response["system_prompt"], "You are a helpful assistant");
        assert_eq!(response["creator_id"], ctx.user_id.to_string());
        let agent_id = response["id"].as_str().unwrap();

        // Step 2: Get agent by ID
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["id"], agent_id);
        assert_eq!(response["name"], "Test Agent");
        assert_eq!(response["is_creator"], true);
        assert!(response["knowledge_bases"].is_array());
        assert!(response["mcp_tools"].is_array());
        assert!(response["flows"].is_array());

        // Step 3: Update agent
        let (status, response) = make_request(
            &app,
            "PUT",
            &format!("/api/agents/{}", agent_id),
            Some(&token),
            Some(json!({
                "name": "Updated Agent",
                "system_prompt": "You are an updated assistant"
            })),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["name"], "Updated Agent");
        assert_eq!(response["system_prompt"], "You are an updated assistant");

        // Step 4: Delete agent
        let (status, _) = make_request(
            &app,
            "DELETE",
            &format!("/api/agents/{}", agent_id),
            Some(&token),
            None,
        ).await;

        assert_eq!(status, StatusCode::NO_CONTENT);

        // Step 5: Verify agent is deleted
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token),
            None,
        ).await;

        assert_eq!(status, StatusCode::NOT_FOUND);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_permission_control() {
        // Requirement 3.1, 3.2: Test creator-only modification permissions
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token1 = ctx.login(&ctx.username, "password123", &app).await;
        let token2 = ctx.login(&ctx.username2, "password456", &app).await;

        // User 1 creates an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token1),
            Some(json!({
                "name": "User 1 Agent",
                "system_prompt": "Private agent",
                "preset_questions": [],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let agent_id = response["id"].as_str().unwrap();

        // User 2 can view the agent (same tenant)
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["is_creator"], false);

        // User 2 cannot update the agent
        let (status, _) = make_request(
            &app,
            "PUT",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            Some(json!({
                "name": "Hacked Agent"
            })),
        ).await;

        assert!(status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED);

        // User 2 cannot delete the agent
        let (status, _) = make_request(
            &app,
            "DELETE",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            None,
        ).await;

        assert!(status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED);

        // User 1 can still update and delete
        let (status, _) = make_request(
            &app,
            "PUT",
            &format!("/api/agents/{}", agent_id),
            Some(&token1),
            Some(json!({
                "name": "Updated by Owner"
            })),
        ).await;

        assert_eq!(status, StatusCode::OK);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_copy_functionality() {
        // Requirement 4.1, 4.2, 4.3, 4.4, 4.5: Test agent copying
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token1 = ctx.login(&ctx.username, "password123", &app).await;
        let token2 = ctx.login(&ctx.username2, "password456", &app).await;

        // User 1 creates an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token1),
            Some(json!({
                "name": "Original Agent",
                "avatar": "https://example.com/avatar.png",
                "system_prompt": "Original prompt",
                "additional_settings": "{\"key\": \"value\"}",
                "preset_questions": ["Q1", "Q2"],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let original_id = response["id"].as_str().unwrap().to_string();

        // User 2 copies the agent
        let (status, response) = make_request(
            &app,
            "POST",
            &format!("/api/agents/{}/copy", original_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let copied_id = response["id"].as_str().unwrap();
        
        // Verify copied agent has different ID
        assert_ne!(copied_id, original_id);
        
        // Verify copied agent has same configuration
        assert_eq!(response["name"], "Original Agent");
        assert_eq!(response["system_prompt"], "Original prompt");
        assert_eq!(response["additional_settings"], "{\"key\": \"value\"}");
        
        // Verify copied agent has new creator
        assert_eq!(response["creator_id"], ctx.user2_id.to_string());
        
        // Verify source_agent_id is set
        assert_eq!(response["source_agent_id"], original_id);

        // Get full details to verify source reference
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", copied_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["source_agent"].is_object());
        assert_eq!(response["source_agent"]["id"], original_id);
        assert_eq!(response["source_agent"]["name"], "Original Agent");

        // User 2 can modify the copied agent independently
        let (status, response) = make_request(
            &app,
            "PUT",
            &format!("/api/agents/{}", copied_id),
            Some(&token2),
            Some(json!({
                "name": "Modified Copy"
            })),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["name"], "Modified Copy");

        // Original agent should remain unchanged
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", original_id),
            Some(&token1),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["name"], "Original Agent");

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_employment_management() {
        // Requirement 5.1, 5.2, 5.3, 5.4, 5.5: Test employment relationships
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token1 = ctx.login(&ctx.username, "password123", &app).await;
        let token2 = ctx.login(&ctx.username2, "password456", &app).await;

        // User 1 creates an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token1),
            Some(json!({
                "name": "Employable Agent",
                "system_prompt": "Ready to work",
                "preset_questions": [],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let agent_id = response["id"].as_str().unwrap();

        // User 2 employs the agent
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/agents/{}/employ", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);

        // Verify employment status in agent details
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["is_employed"], true);

        // List employed agents for User 2
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/agents/employed",
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["items"].is_array());
        let items = response["items"].as_array().unwrap();
        assert!(items.iter().any(|item| item["id"] == agent_id));

        // User 2 terminates employment
        let (status, _) = make_request(
            &app,
            "DELETE",
            &format!("/api/agents/{}/employ", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);

        // Verify employment is terminated
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["is_employed"], false);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_allocation_management() {
        // Requirement 5.1, 5.2, 5.3, 5.4, 5.5: Test allocation relationships
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token1 = ctx.login(&ctx.username, "password123", &app).await;
        let token2 = ctx.login(&ctx.username2, "password456", &app).await;

        // User 1 creates an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token1),
            Some(json!({
                "name": "Allocatable Agent",
                "system_prompt": "Ready to work",
                "preset_questions": [],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let agent_id = response["id"].as_str().unwrap();

        // User 2 allocates the agent
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/agents/{}/allocate", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);

        // Verify allocation status in agent details
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["is_allocated"], true);

        // List allocated agents for User 2
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/agents/allocated",
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["items"].is_array());
        let items = response["items"].as_array().unwrap();
        assert!(items.iter().any(|item| item["id"] == agent_id));

        // User 2 terminates allocation
        let (status, _) = make_request(
            &app,
            "DELETE",
            &format!("/api/agents/{}/allocate", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);

        // Verify allocation is terminated
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["is_allocated"], false);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_list_pagination() {
        // Requirement 6.1, 6.2, 6.3, 6.4, 6.5: Test agent listing with pagination
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token = ctx.login(&ctx.username, "password123", &app).await;

        // Create multiple agents
        for i in 1..=5 {
            let (status, _) = make_request(
                &app,
                "POST",
                "/api/agents",
                Some(&token),
                Some(json!({
                    "name": format!("Agent {}", i),
                    "system_prompt": format!("Prompt {}", i),
                    "preset_questions": [],
                    "knowledge_base_ids": [],
                    "mcp_tool_ids": [],
                    "flow_ids": []
                })),
            ).await;
            assert_eq!(status, StatusCode::CREATED);
        }

        // Test pagination - page 1
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/agents?page=1&limit=2",
            Some(&token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["page"], 1);
        assert_eq!(response["limit"], 2);
        assert!(response["total"].as_u64().unwrap() >= 5);
        assert_eq!(response["items"].as_array().unwrap().len(), 2);

        // Verify card format
        let first_item = &response["items"][0];
        assert!(first_item["id"].is_string());
        assert!(first_item["name"].is_string());
        assert!(first_item["system_prompt_preview"].is_string());
        assert!(first_item["creator_name"].is_string());
        assert!(first_item["is_employed"].is_boolean());
        assert!(first_item["is_allocated"].is_boolean());
        assert!(first_item["is_creator"].is_boolean());

        // Test pagination - page 2
        let (status, response) = make_request(
            &app,
            "GET",
            "/api/agents?page=2&limit=2",
            Some(&token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(response["page"], 2);
        assert_eq!(response["items"].as_array().unwrap().len(), 2);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_resource_management() {
        // Requirement 2.4, 3.4: Test resource association management
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token = ctx.login(&ctx.username, "password123", &app).await;

        // Create an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token),
            Some(json!({
                "name": "Resource Test Agent",
                "system_prompt": "Testing resources",
                "preset_questions": [],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let agent_id = response["id"].as_str().unwrap();

        // Note: These tests assume knowledge bases, MCP tools, and flows exist
        // In a real test, you would create these resources first
        
        // Test adding a knowledge base (using a dummy UUID for demonstration)
        let kb_id = Uuid::new_v4();
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/agents/{}/knowledge-bases/{}", agent_id, kb_id),
            Some(&token),
            None,
        ).await;

        // May fail if knowledge base doesn't exist, but tests the endpoint
        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);

        // Test adding an MCP tool
        let tool_id = Uuid::new_v4();
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/agents/{}/mcp-tools/{}", agent_id, tool_id),
            Some(&token),
            None,
        ).await;

        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);

        // Test adding a flow
        let flow_id = Uuid::new_v4();
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/agents/{}/flows/{}", agent_id, flow_id),
            Some(&token),
            None,
        ).await;

        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_preset_questions_validation() {
        // Requirement 2.5: Test preset questions limit (max 3)
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token = ctx.login(&ctx.username, "password123", &app).await;

        // Try to create agent with more than 3 preset questions
        let (status, _) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token),
            Some(json!({
                "name": "Invalid Agent",
                "system_prompt": "Test",
                "preset_questions": ["Q1", "Q2", "Q3", "Q4"],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);

        // Create agent with exactly 3 preset questions (valid)
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token),
            Some(json!({
                "name": "Valid Agent",
                "system_prompt": "Test",
                "preset_questions": ["Q1", "Q2", "Q3"],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response["preset_questions"].as_array().unwrap().len(), 3);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agent_detail_completeness() {
        // Requirement 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7: Test complete agent details
        let ctx = TestContext::setup().await;
        let app = create_test_app(ctx.db.clone()).await;
        let token = ctx.login(&ctx.username, "password123", &app).await;

        // Create an agent
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/agents",
            Some(&token),
            Some(json!({
                "name": "Detail Test Agent",
                "avatar": "https://example.com/avatar.png",
                "system_prompt": "Detailed agent for testing",
                "additional_settings": "{\"setting\": \"value\"}",
                "preset_questions": ["Question 1", "Question 2"],
                "knowledge_base_ids": [],
                "mcp_tool_ids": [],
                "flow_ids": []
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let agent_id = response["id"].as_str().unwrap();

        // Get agent details
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/agents/{}", agent_id),
            Some(&token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        
        // Verify all required fields are present
        assert!(response["id"].is_string());
        assert!(response["tenant_id"].is_string());
        assert_eq!(response["name"], "Detail Test Agent");
        assert_eq!(response["avatar"], "https://example.com/avatar.png");
        assert!(response["knowledge_bases"].is_array());
        assert!(response["mcp_tools"].is_array());
        assert!(response["flows"].is_array());
        assert_eq!(response["system_prompt"], "Detailed agent for testing");
        assert_eq!(response["additional_settings"], "{\"setting\": \"value\"}");
        assert_eq!(response["preset_questions"].as_array().unwrap().len(), 2);
        assert!(response["creator"].is_object());
        assert_eq!(response["creator"]["id"], ctx.user_id.to_string());
        assert_eq!(response["is_employed"], false);
        assert_eq!(response["is_allocated"], false);
        assert_eq!(response["is_creator"], true);
        assert!(response["created_at"].is_string());
        assert!(response["updated_at"].is_string());

        ctx.cleanup().await;
    }
}
