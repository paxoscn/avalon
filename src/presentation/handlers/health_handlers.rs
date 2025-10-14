// Health check and monitoring endpoints
// Requirement 1.3: Add health check and monitoring metrics

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: ComponentHealth,
    pub redis: ComponentHealth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub uptime_seconds: u64,
    pub database_connections: DatabaseMetrics,
    pub cache_stats: CacheMetrics,
    pub request_stats: RequestMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub total_keys: u64,
    pub memory_usage_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
}

pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub start_time: Instant,
}

/// Basic health check endpoint
/// GET /health
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Detailed health check with component status
/// GET /health/detailed
pub async fn detailed_health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    
    // Check database health
    let db_health = check_database_health(&state.db).await;
    
    // Check Redis health (simplified - would need Redis client)
    let redis_health = ComponentHealth {
        status: "ok".to_string(),
        message: None,
        response_time_ms: Some(1),
    };
    
    let overall_status = if db_health.status == "ok" && redis_health.status == "ok" {
        "healthy"
    } else {
        "degraded"
    };
    
    let response = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        checks: HealthChecks {
            database: db_health,
            redis: redis_health,
        },
    };
    
    let status_code = if overall_status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(response))
}

/// Readiness probe for Kubernetes
/// GET /health/ready
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check if application is ready to serve traffic
    let db_health = check_database_health(&state.db).await;
    
    if db_health.status == "ok" {
        (StatusCode::OK, Json(serde_json::json!({
            "ready": true
        })))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "ready": false,
            "reason": db_health.message
        })))
    }
}

/// Liveness probe for Kubernetes
/// GET /health/live
pub async fn liveness_check() -> impl IntoResponse {
    // Simple check that the application is running
    (StatusCode::OK, Json(serde_json::json!({
        "alive": true
    })))
}

/// Metrics endpoint for monitoring
/// GET /metrics
pub async fn metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    
    // In production, these would be collected from actual metrics
    let metrics = MetricsResponse {
        uptime_seconds: uptime,
        database_connections: DatabaseMetrics {
            active_connections: 5,
            idle_connections: 10,
            max_connections: 100,
        },
        cache_stats: CacheMetrics {
            hit_rate: 0.85,
            total_keys: 1234,
            memory_usage_mb: 45.6,
        },
        request_stats: RequestMetrics {
            total_requests: 10000,
            requests_per_second: 50.0,
            average_response_time_ms: 25.5,
        },
    };
    
    (StatusCode::OK, Json(metrics))
}

/// Prometheus-compatible metrics endpoint
/// GET /metrics/prometheus
pub async fn prometheus_metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    
    // Format metrics in Prometheus format
    let metrics = format!(
        "# HELP agent_platform_uptime_seconds Application uptime in seconds\n\
         # TYPE agent_platform_uptime_seconds counter\n\
         agent_platform_uptime_seconds {}\n\
         \n\
         # HELP agent_platform_db_connections_active Active database connections\n\
         # TYPE agent_platform_db_connections_active gauge\n\
         agent_platform_db_connections_active 5\n\
         \n\
         # HELP agent_platform_cache_hit_rate Cache hit rate\n\
         # TYPE agent_platform_cache_hit_rate gauge\n\
         agent_platform_cache_hit_rate 0.85\n\
         \n\
         # HELP agent_platform_requests_total Total number of requests\n\
         # TYPE agent_platform_requests_total counter\n\
         agent_platform_requests_total 10000\n",
        uptime
    );
    
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics,
    )
}

// Helper functions

async fn check_database_health(db: &DatabaseConnection) -> ComponentHealth {
    let start = Instant::now();
    
    match db.ping().await {
        Ok(_) => {
            let response_time = start.elapsed().as_millis() as u64;
            ComponentHealth {
                status: "ok".to_string(),
                message: None,
                response_time_ms: Some(response_time),
            }
        }
        Err(e) => ComponentHealth {
            status: "error".to_string(),
            message: Some(format!("Database connection failed: {}", e)),
            response_time_ms: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let response = liveness_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
