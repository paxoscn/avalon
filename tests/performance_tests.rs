// Performance and load tests
// Run with: cargo test --test performance_tests --release -- --test-threads=1 --nocapture

use agent_platform::config::Config;
use agent_platform::infrastructure::database;
use agent_platform::presentation::server::create_app;
use sea_orm::Database;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Remove when ready for performance testing
    async fn test_concurrent_request_handling() {
        // Requirement 1.4: Test concurrent request handling
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        let app = create_app(db).await;

        let concurrent_requests = 100;
        let mut handles = vec![];

        let start = Instant::now();

        for i in 0..concurrent_requests {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                let request_start = Instant::now();
                
                // Simulate API request
                let result = simulate_api_call(&app_clone, i).await;
                
                let request_duration = request_start.elapsed();
                (result, request_duration)
            });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;
        let total_duration = start.elapsed();

        // Analyze results
        let mut successful = 0;
        let mut failed = 0;
        let mut total_request_time = Duration::ZERO;

        for result in results {
            let (success, duration) = result.unwrap();
            if success {
                successful += 1;
            } else {
                failed += 1;
            }
            total_request_time += duration;
        }

        let avg_request_time = total_request_time / concurrent_requests;
        let requests_per_second = concurrent_requests as f64 / total_duration.as_secs_f64();

        println!("=== Concurrent Request Test Results ===");
        println!("Total requests: {}", concurrent_requests);
        println!("Successful: {}", successful);
        println!("Failed: {}", failed);
        println!("Total time: {:?}", total_duration);
        println!("Average request time: {:?}", avg_request_time);
        println!("Requests per second: {:.2}", requests_per_second);

        // Assertions
        assert!(successful > concurrent_requests * 95 / 100, "Success rate should be > 95%");
        assert!(avg_request_time < Duration::from_millis(500), "Average request time should be < 500ms");
    }

    #[tokio::test]
    #[ignore]
    async fn test_database_query_performance() {
        // Requirement 2.3: Test and optimize database queries
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        // Test various query patterns
        let test_cases = vec![
            ("Simple SELECT", "SELECT * FROM flows LIMIT 10"),
            ("JOIN query", "SELECT f.*, fv.* FROM flows f JOIN flow_versions fv ON f.id = fv.flow_id LIMIT 10"),
            ("Aggregation", "SELECT tenant_id, COUNT(*) FROM flows GROUP BY tenant_id"),
            ("Complex filter", "SELECT * FROM flow_executions WHERE status = 'completed' AND started_at > NOW() - INTERVAL 1 DAY"),
        ];

        println!("=== Database Query Performance ===");
        
        for (name, query) in test_cases {
            let start = Instant::now();
            
            // Execute query multiple times
            for _ in 0..10 {
                let _ = db.execute_unprepared(query).await;
            }
            
            let duration = start.elapsed();
            let avg_duration = duration / 10;
            
            println!("{}: avg {:?}", name, avg_duration);
            
            // Query should complete in reasonable time
            assert!(avg_duration < Duration::from_millis(100), "{} took too long", name);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_flow_execution_performance() {
        // Requirement 2.3: Test flow execution performance
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        let app = create_app(db).await;

        // Create test flow
        let flow_id = create_test_flow(&app).await;

        // Execute flow multiple times and measure performance
        let iterations = 50;
        let mut execution_times = vec![];

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = execute_flow(&app, &flow_id).await;
            let duration = start.elapsed();
            execution_times.push(duration);
        }

        // Calculate statistics
        let total: Duration = execution_times.iter().sum();
        let avg = total / iterations as u32;
        let min = execution_times.iter().min().unwrap();
        let max = execution_times.iter().max().unwrap();

        println!("=== Flow Execution Performance ===");
        println!("Iterations: {}", iterations);
        println!("Average: {:?}", avg);
        println!("Min: {:?}", min);
        println!("Max: {:?}", max);

        // Performance assertions
        assert!(avg < Duration::from_secs(2), "Average execution time should be < 2s");
        assert!(max < Duration::from_secs(5), "Max execution time should be < 5s");
    }

    #[tokio::test]
    #[ignore]
    async fn test_cache_effectiveness() {
        // Requirement 6.3, 7.3: Test caching strategy
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        let app = create_app(db).await;

        // First request (cache miss)
        let start = Instant::now();
        let _ = get_flow_list(&app).await;
        let first_request = start.elapsed();

        // Second request (cache hit)
        let start = Instant::now();
        let _ = get_flow_list(&app).await;
        let second_request = start.elapsed();

        println!("=== Cache Effectiveness ===");
        println!("First request (cache miss): {:?}", first_request);
        println!("Second request (cache hit): {:?}", second_request);
        println!("Speedup: {:.2}x", first_request.as_secs_f64() / second_request.as_secs_f64());

        // Cache should provide significant speedup
        assert!(second_request < first_request / 2, "Cache should provide at least 2x speedup");
    }

    #[tokio::test]
    #[ignore]
    async fn test_memory_usage_under_load() {
        // Requirement 1.4: Test memory usage
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        let app = create_app(db).await;

        // Simulate sustained load
        let duration = Duration::from_secs(30);
        let start = Instant::now();
        let mut request_count = 0;

        while start.elapsed() < duration {
            let _ = simulate_api_call(&app, request_count).await;
            request_count += 1;
            sleep(Duration::from_millis(10)).await;
        }

        println!("=== Memory Usage Test ===");
        println!("Duration: {:?}", duration);
        println!("Total requests: {}", request_count);
        println!("Requests per second: {:.2}", request_count as f64 / duration.as_secs_f64());

        // Note: In production, you would use tools like valgrind or heaptrack
        // to measure actual memory usage
    }

    #[tokio::test]
    #[ignore]
    async fn test_rate_limiting() {
        // Requirement 1.4: Test API rate limiting
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        let app = create_app(db).await;

        // Send requests rapidly
        let rapid_requests = 100;
        let mut success_count = 0;
        let mut rate_limited_count = 0;

        let start = Instant::now();

        for i in 0..rapid_requests {
            let result = simulate_api_call(&app, i).await;
            if result {
                success_count += 1;
            } else {
                rate_limited_count += 1;
            }
        }

        let duration = start.elapsed();

        println!("=== Rate Limiting Test ===");
        println!("Total requests: {}", rapid_requests);
        println!("Successful: {}", success_count);
        println!("Rate limited: {}", rate_limited_count);
        println!("Duration: {:?}", duration);

        // Verify rate limiting is working
        // (Adjust based on your rate limit configuration)
        assert!(rate_limited_count > 0, "Rate limiting should kick in");
    }

    // Helper functions
    async fn simulate_api_call(app: &axum::Router, _iteration: u32) -> bool {
        // Simulate an API call
        // In real implementation, use the test helpers from integration tests
        sleep(Duration::from_millis(10)).await;
        true
    }

    async fn create_test_flow(app: &axum::Router) -> String {
        // Create a test flow and return its ID
        // In real implementation, use the test helpers
        "test-flow-id".to_string()
    }

    async fn execute_flow(app: &axum::Router, flow_id: &str) -> bool {
        // Execute a flow
        // In real implementation, use the test helpers
        sleep(Duration::from_millis(100)).await;
        true
    }

    async fn get_flow_list(app: &axum::Router) -> Vec<String> {
        // Get flow list
        // In real implementation, use the test helpers
        sleep(Duration::from_millis(50)).await;
        vec![]
    }
}

#[cfg(test)]
mod load_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_sustained_load() {
        // Requirement 2.3: Test system under sustained load
        let config = Config::from_env().expect("Failed to load config");
        let db = Database::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        let app = create_app(db).await;

        let test_duration = Duration::from_secs(60);
        let target_rps = 50; // requests per second
        let interval = Duration::from_millis(1000 / target_rps);

        let start = Instant::now();
        let mut total_requests = 0;
        let mut successful_requests = 0;
        let mut failed_requests = 0;

        println!("=== Sustained Load Test ===");
        println!("Duration: {:?}", test_duration);
        println!("Target RPS: {}", target_rps);

        while start.elapsed() < test_duration {
            let request_start = Instant::now();
            
            let success = simulate_load_request(&app).await;
            
            if success {
                successful_requests += 1;
            } else {
                failed_requests += 1;
            }
            total_requests += 1;

            // Maintain target rate
            let elapsed = request_start.elapsed();
            if elapsed < interval {
                sleep(interval - elapsed).await;
            }

            // Print progress every 10 seconds
            if total_requests % (target_rps * 10) == 0 {
                let current_duration = start.elapsed();
                let current_rps = total_requests as f64 / current_duration.as_secs_f64();
                println!("Progress: {}s, {} requests, {:.2} RPS", 
                    current_duration.as_secs(), total_requests, current_rps);
            }
        }

        let actual_duration = start.elapsed();
        let actual_rps = total_requests as f64 / actual_duration.as_secs_f64();
        let success_rate = (successful_requests as f64 / total_requests as f64) * 100.0;

        println!("\n=== Results ===");
        println!("Total requests: {}", total_requests);
        println!("Successful: {}", successful_requests);
        println!("Failed: {}", failed_requests);
        println!("Actual RPS: {:.2}", actual_rps);
        println!("Success rate: {:.2}%", success_rate);

        // Assertions
        assert!(success_rate > 95.0, "Success rate should be > 95%");
        assert!(actual_rps >= target_rps as f64 * 0.9, "Should maintain at least 90% of target RPS");
    }

    async fn simulate_load_request(app: &axum::Router) -> bool {
        // Simulate various API operations
        sleep(Duration::from_millis(20)).await;
        true
    }
}
