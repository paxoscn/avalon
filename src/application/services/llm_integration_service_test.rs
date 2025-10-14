use super::llm_integration_service::*;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_default() {
        let config = LLMIntegrationConfig::default();
        
        assert!(config.enable_auto_failover);
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert!(config.enable_metrics);
        assert!(!config.enable_caching);
        assert_eq!(config.cache_ttl, Duration::from_secs(300));
    }

    #[test]
    fn test_service_metrics_default() {
        let metrics = LLMServiceMetrics::default();
        
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 0);
        assert_eq!(metrics.average_response_time_ms, 0.0);
        assert!(metrics.provider_usage.is_empty());
        assert!(metrics.error_counts.is_empty());
    }

    #[test]
    fn test_integration_config_custom() {
        let config = LLMIntegrationConfig {
            enable_auto_failover: false,
            health_check_interval: Duration::from_secs(120),
            max_concurrent_requests: 50,
            request_timeout: Duration::from_secs(60),
            enable_metrics: false,
            enable_caching: true,
            cache_ttl: Duration::from_secs(600),
        };

        assert!(!config.enable_auto_failover);
        assert_eq!(config.health_check_interval, Duration::from_secs(120));
        assert_eq!(config.max_concurrent_requests, 50);
        assert_eq!(config.request_timeout, Duration::from_secs(60));
        assert!(!config.enable_metrics);
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl, Duration::from_secs(600));
    }

    #[test]
    fn test_builder_default() {
        let _builder = LLMIntegrationServiceBuilder::new();
        
        // Test that builder can be created with default values
        // We can't access private fields, but creation should succeed
    }

    #[test]
    fn test_builder_with_service_config() {
        let service_config = LLMIntegrationConfig {
            enable_auto_failover: false,
            health_check_interval: Duration::from_secs(30),
            max_concurrent_requests: 200,
            request_timeout: Duration::from_secs(15),
            enable_metrics: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(300),
        };

        let _builder = LLMIntegrationServiceBuilder::new()
            .with_service_config(service_config.clone());

        // Builder should be created successfully with custom config
    }

    #[test]
    fn test_builder_with_health_monitoring() {
        let _builder = LLMIntegrationServiceBuilder::new()
            .with_health_monitoring(true, Duration::from_secs(30));

        // Builder should be created successfully with health monitoring config
    }

    #[test]
    fn test_builder_with_auto_failover() {
        let _builder = LLMIntegrationServiceBuilder::new()
            .with_auto_failover(false);

        // Builder should be created successfully with auto failover config
    }

    #[test]
    fn test_builder_with_request_timeout() {
        let _builder = LLMIntegrationServiceBuilder::new()
            .with_request_timeout(Duration::from_secs(45));

        // Builder should be created successfully with request timeout config
    }

    #[test]
    fn test_builder_with_caching() {
        let _builder = LLMIntegrationServiceBuilder::new()
            .with_caching(true, Duration::from_secs(600));

        // Builder should be created successfully with caching config
    }

    #[test]
    fn test_service_metrics_operations() {
        let mut metrics = LLMServiceMetrics::default();
        
        // Test that we can modify metrics
        metrics.total_requests = 100;
        metrics.successful_requests = 95;
        metrics.failed_requests = 5;
        metrics.average_response_time_ms = 250.5;
        
        assert_eq!(metrics.total_requests, 100);
        assert_eq!(metrics.successful_requests, 95);
        assert_eq!(metrics.failed_requests, 5);
        assert_eq!(metrics.average_response_time_ms, 250.5);
    }

    #[test]
    fn test_service_metrics_provider_usage() {
        let mut metrics = LLMServiceMetrics::default();
        
        metrics.provider_usage.insert("openai".to_string(), 50);
        metrics.provider_usage.insert("claude".to_string(), 30);
        metrics.error_counts.insert("rate_limit".to_string(), 2);
        
        assert_eq!(metrics.provider_usage.get("openai"), Some(&50));
        assert_eq!(metrics.provider_usage.get("claude"), Some(&30));
        assert_eq!(metrics.error_counts.get("rate_limit"), Some(&2));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::application::services::llm_service_factory::LLMServiceFactoryConfig;

    #[test]
    fn test_factory_config_validation() {
        // Test that we can create a factory config for testing
        let config = LLMServiceFactoryConfig {
            openai_api_key: Some("sk-test123".to_string()),
            claude_api_key: Some("sk-ant-test123".to_string()),
            local_llm_url: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };

        assert!(config.openai_api_key.is_some());
        assert!(config.claude_api_key.is_some());
        assert!(config.local_llm_url.is_some());
    }

    #[test]
    fn test_integration_config_combinations() {
        // Test high availability configuration
        let ha_config = LLMIntegrationConfig {
            enable_auto_failover: true,
            health_check_interval: Duration::from_secs(30),
            max_concurrent_requests: 200,
            request_timeout: Duration::from_secs(15),
            enable_metrics: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(600),
        };

        assert!(ha_config.enable_auto_failover);
        assert_eq!(ha_config.health_check_interval, Duration::from_secs(30));
        assert_eq!(ha_config.max_concurrent_requests, 200);

        // Test performance optimized configuration
        let perf_config = LLMIntegrationConfig {
            enable_auto_failover: true,
            health_check_interval: Duration::from_secs(45),
            max_concurrent_requests: 150,
            request_timeout: Duration::from_secs(10),
            enable_metrics: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(180),
        };

        assert!(perf_config.enable_auto_failover);
        assert_eq!(perf_config.health_check_interval, Duration::from_secs(45));
        assert_eq!(perf_config.max_concurrent_requests, 150);
        assert_eq!(perf_config.request_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_builder_chain() {
        use crate::application::services::integrated_llm_service::LoadBalancingStrategy;
        
        let builder = LLMIntegrationServiceBuilder::new()
            .with_load_balancing_strategy(LoadBalancingStrategy::RoundRobin)
            .with_health_monitoring(true, Duration::from_secs(30))
            .with_auto_failover(true)
            .with_request_timeout(Duration::from_secs(15))
            .with_caching(true, Duration::from_secs(300));

        // Test that builder can be created with chained configuration
        // We can't access private fields, but creation should succeed
    }
}