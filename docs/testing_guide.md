# Testing Guide

## Overview

This document describes the testing strategy and how to run tests for the Agent Platform.

## Test Types

### 1. Unit Tests

Unit tests are located alongside the source code in each module. They test individual functions and methods in isolation.

**Running unit tests:**
```bash
cargo test --lib
```

**Running tests for a specific module:**
```bash
cargo test --lib domain::services::flow_service
```

### 2. Integration Tests

Integration tests are located in the `tests/` directory. They test the interaction between multiple components.

**Running integration tests:**
```bash
cargo test --test api_integration_tests
```

**Note:** Integration tests are currently marked with `#[ignore]` and need to be implemented. Remove the ignore attribute when implementing.

### 3. API Tests

API tests verify the REST endpoints work correctly end-to-end.

**Prerequisites:**
- Running database (MySQL)
- Running Redis instance
- Test environment configuration

**Running API tests:**
```bash
# Set up test database
export DATABASE_URL="mysql://user:pass@localhost/agent_platform_test"
export REDIS_URL="redis://localhost:6379"

# Run tests
cargo test --test api_integration_tests -- --test-threads=1
```

## Test Database Setup

For integration and API tests, you need a test database:

```bash
# Create test database
mysql -u root -p -e "CREATE DATABASE agent_platform_test;"

# Run migrations
DATABASE_URL="mysql://user:pass@localhost/agent_platform_test" cargo run --bin migrate
```

## Writing Tests

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_validation() {
        let flow = Flow::new(
            TenantId::new(),
            FlowName::new("Test Flow").unwrap(),
            None,
            UserId::new(),
        );
        
        assert!(flow.validate().is_ok());
    }
}
```

### Integration Test Example

```rust
#[tokio::test]
async fn test_create_and_get_flow() {
    // Setup
    let service = setup_flow_service().await;
    let tenant_id = TenantId::new();
    let user_id = UserId::new();
    
    // Create flow
    let flow = service.create_flow(
        tenant_id,
        "Test Flow".to_string(),
        None,
        user_id,
    ).await.unwrap();
    
    // Get flow
    let retrieved = service.get_flow(flow.id, tenant_id).await.unwrap();
    
    // Assert
    assert_eq!(flow.id, retrieved.id);
    assert_eq!(flow.name, retrieved.name);
    
    // Cleanup
    cleanup_test_data().await;
}
```

## Test Coverage

To generate test coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## Mocking

For unit tests that depend on external services, use mocking:

```rust
use mockall::mock;

mock! {
    pub FlowRepository {}
    
    #[async_trait]
    impl FlowRepository for FlowRepository {
        async fn find_by_id(&self, id: &FlowId) -> Result<Option<Flow>>;
        async fn save(&self, flow: &Flow) -> Result<()>;
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock_repo = MockFlowRepository::new();
    mock_repo
        .expect_find_by_id()
        .returning(|_| Ok(Some(create_test_flow())));
    
    // Use mock in test
}
```

## Performance Testing

For performance testing, use criterion:

```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks
cargo criterion
```

## Load Testing

For load testing the API, use tools like:

- **Apache Bench (ab)**
  ```bash
  ab -n 1000 -c 10 http://localhost:8080/api/flows
  ```

- **wrk**
  ```bash
  wrk -t12 -c400 -d30s http://localhost:8080/api/flows
  ```

- **k6**
  ```javascript
  import http from 'k6/http';
  import { check } from 'k6';

  export default function () {
    const res = http.get('http://localhost:8080/api/flows');
    check(res, { 'status is 200': (r) => r.status === 200 });
  }
  ```

## Continuous Integration

Tests should be run in CI/CD pipeline:

```yaml
# Example GitHub Actions workflow
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      mysql:
        image: mysql:8.0
        env:
          MYSQL_ROOT_PASSWORD: password
          MYSQL_DATABASE: agent_platform_test
        ports:
          - 3306:3306
      
      redis:
        image: redis:7
        ports:
          - 6379:6379
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all
        env:
          DATABASE_URL: mysql://root:password@localhost/agent_platform_test
          REDIS_URL: redis://localhost:6379
```

## Test Data Management

### Fixtures

Create test fixtures for common test data:

```rust
pub fn create_test_tenant() -> Tenant {
    Tenant {
        id: TenantId::new(),
        name: "Test Tenant".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn create_test_user(tenant_id: TenantId) -> User {
    User {
        id: UserId::new(),
        tenant_id,
        username: Username::new("testuser").unwrap(),
        nickname: Some("Test User".to_string()),
        password_hash: "hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
```

### Cleanup

Always clean up test data after tests:

```rust
#[tokio::test]
async fn test_with_cleanup() {
    // Setup
    let data = setup_test_data().await;
    
    // Test
    let result = perform_test(&data).await;
    
    // Cleanup
    cleanup_test_data(&data).await;
    
    // Assert
    assert!(result.is_ok());
}
```

## Best Practices

1. **Isolation**: Each test should be independent and not rely on other tests
2. **Cleanup**: Always clean up test data to avoid side effects
3. **Naming**: Use descriptive test names that explain what is being tested
4. **Arrange-Act-Assert**: Structure tests with clear setup, execution, and verification
5. **Fast Tests**: Keep unit tests fast; use mocks for external dependencies
6. **Realistic Data**: Use realistic test data that represents actual use cases
7. **Error Cases**: Test both success and failure scenarios
8. **Edge Cases**: Test boundary conditions and edge cases
9. **Documentation**: Document complex test scenarios
10. **Maintenance**: Keep tests up to date with code changes

## Troubleshooting

### Tests Failing Locally

1. Check database connection
2. Verify Redis is running
3. Ensure migrations are up to date
4. Check for port conflicts
5. Review test logs for specific errors

### Flaky Tests

1. Check for race conditions
2. Verify test isolation
3. Look for timing dependencies
4. Check for shared state
5. Use proper synchronization

### Slow Tests

1. Profile test execution
2. Use mocks for external services
3. Optimize database queries
4. Parallelize independent tests
5. Use test fixtures efficiently

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Mockall Documentation](https://docs.rs/mockall/)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
