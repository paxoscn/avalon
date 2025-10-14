// Integration tests for REST API endpoints
// These tests require a running database and should be run with:
// cargo test --test api_integration_tests -- --test-threads=1

use agent_platform::config::Config;
use agent_platform::infrastructure::database;
use agent_platform::presentation::server::create_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sea_orm::{Database, DatabaseConnection};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

#[cfg(test)]
mod test_helpers {
    use super::*;
    use bcrypt::{hash, DEFAULT_COST};
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    use agent_platform::infrastructure::database::entities::{tenant, user};

    pub struct TestContext {
        pub db: DatabaseConnection,
        pub tenant_id: Uuid,
        pub user_id: Uuid,
        pub auth_token: String,
        pub tenant2_id: Uuid,
        pub user2_id: Uuid,
        pub auth_token2: String,
    }

    impl TestContext {
        pub async fn setup() -> Self {
            // Load test configuration
            let config = Config::from_env().expect("Failed to load config");
            
            // Connect to test database
            let db = Database::connect(&config.database_url)
                .await
                .expect("Failed to connect to test database");

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

            // Generate auth tokens (simplified - in real implementation, call login endpoint)
            let auth_token = format!("test_token_{}", user_id);
            let auth_token2 = format!("test_token_{}", user2_id);

            Self {
                db,
                tenant_id,
                user_id,
                auth_token,
                tenant2_id,
                user2_id,
                auth_token2,
            }
        }

        pub async fn cleanup(&self) {
            // Clean up test data
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
mod tests {
    use super::*;
    use test_helpers::*;

    #[tokio::test]
    #[ignore] // Remove when test database is configured
    async fn test_complete_user_flow_login_to_execution() {
        // Requirement 10.1, 10.4, 2.1, 2.2: Complete user flow from login to flow execution
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // Step 1: Login with valid credentials
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/auth/login",
            None,
            Some(json!({
                "tenant_id": ctx.tenant_id.to_string(),
                "username": "testuser1",
                "password": "password123"
            })),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["token"].is_string());
        let token = response["token"].as_str().unwrap();

        // Step 2: Access protected endpoint - list flows
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/flows?tenant_id={}", ctx.tenant_id),
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["flows"].is_array());

        // Step 3: Create a new flow
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/flows",
            Some(token),
            Some(json!({
                "name": "Test Flow",
                "description": "Integration test flow",
                "definition": {
                    "nodes": [],
                    "edges": []
                }
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let flow_id = response["id"].as_str().unwrap();

        // Step 4: Execute the flow
        let (status, response) = make_request(
            &app,
            "POST",
            &format!("/api/flows/{}/execute", flow_id),
            Some(token),
            Some(json!({
                "variables": {}
            })),
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["execution_id"].is_string());

        // Step 5: Check execution status
        let execution_id = response["execution_id"].as_str().unwrap();
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/executions/{}", execution_id),
            Some(token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(["pending", "running", "completed", "failed"].contains(&response["status"].as_str().unwrap()));

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_multi_tenant_isolation() {
        // Requirement 10.4: Test multi-tenant isolation
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // User 1 creates a flow
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/flows",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Tenant 1 Flow",
                "description": "Private flow",
                "definition": {"nodes": [], "edges": []}
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let flow_id = response["id"].as_str().unwrap();

        // User 2 (different tenant) tries to access User 1's flow
        let (status, _) = make_request(
            &app,
            "GET",
            &format!("/api/flows/{}", flow_id),
            Some(&ctx.auth_token2),
            None,
        ).await;

        // Should be forbidden or not found
        assert!(status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND);

        // User 2 lists flows - should not see User 1's flow
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/flows?tenant_id={}", ctx.tenant2_id),
            Some(&ctx.auth_token2),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let flows = response["flows"].as_array().unwrap();
        assert!(!flows.iter().any(|f| f["id"].as_str().unwrap() == flow_id));

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_permission_control() {
        // Requirement 10.4: Test permission control
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // Attempt to access protected endpoint without token
        let (status, _) = make_request(
            &app,
            "GET",
            "/api/flows",
            None,
            None,
        ).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);

        // Attempt with invalid token
        let (status, _) = make_request(
            &app,
            "GET",
            "/api/flows",
            Some("invalid_token"),
            None,
        ).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_data_consistency_and_transactions() {
        // Requirement 2.1, 2.2: Verify data consistency and transaction integrity
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // Create a flow with version
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/flows",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Versioned Flow",
                "description": "Test versioning",
                "definition": {"nodes": [], "edges": []}
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let flow_id = response["id"].as_str().unwrap();
        let initial_version = response["version"].as_i64().unwrap();

        // Update flow - should create new version
        let (status, response) = make_request(
            &app,
            "PUT",
            &format!("/api/flows/{}", flow_id),
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Updated Flow",
                "description": "Updated description",
                "definition": {"nodes": [{"id": "1"}], "edges": []}
            })),
        ).await;

        assert_eq!(status, StatusCode::OK);
        let new_version = response["version"].as_i64().unwrap();
        assert!(new_version > initial_version);

        // Verify version history
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/flows/{}/versions", flow_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        let versions = response["versions"].as_array().unwrap();
        assert!(versions.len() >= 2);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_concurrent_flow_executions() {
        // Requirement 2.1, 2.2: Test concurrent executions
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // Create a flow
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/flows",
            Some(&ctx.auth_token),
            Some(json!({
                "name": "Concurrent Test Flow",
                "definition": {"nodes": [], "edges": []}
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let flow_id = response["id"].as_str().unwrap().to_string();

        // Execute flow multiple times concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let app_clone = app.clone();
            let token = ctx.auth_token.clone();
            let flow_id_clone = flow_id.clone();
            
            let handle = tokio::spawn(async move {
                make_request(
                    &app_clone,
                    "POST",
                    &format!("/api/flows/{}/execute", flow_id_clone),
                    Some(&token),
                    Some(json!({"variables": {"iteration": i}})),
                ).await
            });
            
            handles.push(handle);
        }

        // Wait for all executions
        let results = futures::future::join_all(handles).await;

        // Verify all executions succeeded
        for result in results {
            let (status, response) = result.unwrap();
            assert_eq!(status, StatusCode::OK);
            assert!(response["execution_id"].is_string());
        }

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_session_context_persistence() {
        // Requirement 2.1: Test session context storage and retrieval
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // Create a session
        let (status, response) = make_request(
            &app,
            "POST",
            "/api/sessions",
            Some(&ctx.auth_token),
            Some(json!({
                "title": "Test Session"
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);
        let session_id = response["id"].as_str().unwrap();

        // Add messages to session
        let (status, _) = make_request(
            &app,
            "POST",
            &format!("/api/sessions/{}/messages", session_id),
            Some(&ctx.auth_token),
            Some(json!({
                "role": "user",
                "content": "Hello, agent!"
            })),
        ).await;

        assert_eq!(status, StatusCode::CREATED);

        // Retrieve session with messages
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/sessions/{}", session_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["messages"].is_array());
        assert!(response["messages"].as_array().unwrap().len() > 0);

        ctx.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_audit_trail_completeness() {
        // Requirement 10.1: Verify audit logging
        let ctx = TestContext::setup().await;
        let app = create_app(ctx.db.clone()).await;

        // Perform various operations
        let operations = vec![
            ("POST", "/api/flows", Some(json!({"name": "Audit Test", "definition": {}}))),
            ("GET", "/api/flows", None),
        ];

        for (method, path, body) in operations {
            let _ = make_request(&app, method, path, Some(&ctx.auth_token), body).await;
        }

        // Query audit logs
        let (status, response) = make_request(
            &app,
            "GET",
            &format!("/api/audit?tenant_id={}", ctx.tenant_id),
            Some(&ctx.auth_token),
            None,
        ).await;

        assert_eq!(status, StatusCode::OK);
        assert!(response["logs"].is_array());
        let logs = response["logs"].as_array().unwrap();
        assert!(logs.len() >= 2);

        ctx.cleanup().await;
    }
}
