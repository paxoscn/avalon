#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::{IntoResponse, Response},
    };
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        application::{
            services::MockAuthApplicationService,
            dto::{
                LoginRequest, LoginResponse, UserInfo, RefreshTokenRequest, 
                RefreshTokenResponse, LogoutRequest, LogoutResponse,
                ChangePasswordRequest, ChangePasswordResponse, AuthContext
            },
        },
        domain::events::{UserAuthenticatedEvent, TokenRefreshedEvent, UserLoggedOutEvent, PasswordChangedEvent},
        presentation::{
            handlers::{health_handler, me_handler},
            routes::create_auth_routes,
        },
    };
    use mockall::predicate::*;

    fn create_test_auth_context() -> AuthContext {
        AuthContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "testuser".to_string(),
            Some("Test User".to_string()),
            Uuid::new_v4(),
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
        )
    }

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_me_handler_success() {
        let auth_context = create_test_auth_context();

        let result = me_handler(axum::Extension(auth_context.clone())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_handler_success() {
        let mut auth_service = MockAuthApplicationService::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let login_response = LoginResponse {
            token: "test_token".to_string(),
            user: UserInfo {
                id: user_id,
                tenant_id,
                username: "testuser".to_string(),
                nickname: Some("Test User".to_string()),
                created_at: chrono::Utc::now(),
            },
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        };

        let auth_event = UserAuthenticatedEvent::new(
            user_id,
            tenant_id,
            "testuser".to_string(),
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
            1,
        );

        auth_service
            .expect_login()
            .with(always(), always(), always())
            .times(1)
            .returning(move |_, _, _| Ok((login_response.clone(), auth_event.clone())));

        let app = create_auth_routes(Arc::new(auth_service));

        let request_body = json!({
            "tenant_id": tenant_id,
            "username": "testuser",
            "password": "password123"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_handler_invalid_credentials() {
        let mut auth_service = MockAuthApplicationService::new();

        auth_service
            .expect_login()
            .with(always(), always(), always())
            .times(1)
            .returning(|_, _, _| Err(crate::error::PlatformError::AuthenticationFailed("Invalid credentials".to_string())));

        let app = create_auth_routes(Arc::new(auth_service));

        let request_body = json!({
            "tenant_id": Uuid::new_v4(),
            "username": "testuser",
            "password": "wrongpassword"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_refresh_token_handler_success() {
        let mut auth_service = MockAuthApplicationService::new();

        let refresh_response = RefreshTokenResponse {
            token: "new_test_token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        };

        let refresh_event = TokenRefreshedEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some("127.0.0.1".to_string()),
            1,
        );

        auth_service
            .expect_refresh_token()
            .with(always(), always())
            .times(1)
            .returning(move |_, _| Ok((refresh_response.clone(), refresh_event.clone())));

        let app = create_auth_routes(Arc::new(auth_service));

        let request_body = json!({
            "token": "old_test_token"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/auth/refresh")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logout_handler_success() {
        let mut auth_service = MockAuthApplicationService::new();

        let logout_response = LogoutResponse {
            success: true,
            message: "Successfully logged out".to_string(),
        };

        let logout_event = UserLoggedOutEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "testuser".to_string(),
            Uuid::new_v4(),
            Some("127.0.0.1".to_string()),
            1,
        );

        auth_service
            .expect_logout()
            .with(always(), always())
            .times(1)
            .returning(move |_, _| Ok((logout_response.clone(), logout_event.clone())));

        let app = create_auth_routes(Arc::new(auth_service));

        let request_body = json!({
            "token": "test_token"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_me_handler_with_auth_middleware() {
        let mut auth_service = MockAuthApplicationService::new();
        let auth_context = create_test_auth_context();

        auth_service
            .expect_validate_token()
            .with(eq("test_token"))
            .times(1)
            .returning(move |_| Ok(auth_context.clone()));

        let app = create_auth_routes(Arc::new(auth_service));

        let request = Request::builder()
            .method("GET")
            .uri("/auth/me")
            .header("authorization", "Bearer test_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_me_handler_unauthorized() {
        let auth_service = MockAuthApplicationService::new();
        let app = create_auth_routes(Arc::new(auth_service));

        let request = Request::builder()
            .method("GET")
            .uri("/auth/me")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_change_password_handler_success() {
        let mut auth_service = MockAuthApplicationService::new();
        let auth_context = create_test_auth_context();

        let change_password_response = ChangePasswordResponse {
            success: true,
            message: "Password changed successfully".to_string(),
        };

        let password_change_event = PasswordChangedEvent::new(
            auth_context.user_id,
            auth_context.tenant_id,
            auth_context.username.clone(),
            auth_context.user_id,
            1,
        );

        auth_service
            .expect_validate_token()
            .with(eq("test_token"))
            .times(1)
            .returning(move |_| Ok(auth_context.clone()));

        auth_service
            .expect_change_password()
            .with(always(), always())
            .times(1)
            .returning(move |_, _| Ok((change_password_response.clone(), password_change_event.clone())));

        let app = create_auth_routes(Arc::new(auth_service));

        let request_body = json!({
            "current_password": "oldpassword",
            "new_password": "newpassword123"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/auth/change-password")
            .header("authorization", "Bearer test_token")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}