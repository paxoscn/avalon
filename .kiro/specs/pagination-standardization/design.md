# Design Document

## Overview

This design document outlines the approach to standardize pagination across all application services in the platform. The standardization will ensure consistency in how pagination is handled throughout the codebase, making it easier to maintain and understand. The design follows the pattern established by `FlowApplicationService::list_flows` as the reference implementation.

## Architecture

### Layered Approach

The pagination standardization will be implemented across three layers:

1. **Application Service Layer**: Uses zero-based pagination (`page` starts from 0)
   - Consistent method signatures: `(tenant_id: TenantId, page: u64, limit: u64)`
   - Consistent return type: `Result<(Vec<T>, u64)>`
   - Internal offset calculation: `offset = page * limit`

2. **Handler Layer**: Converts between API and application layer conventions
   - Receives 1-based page numbers from API clients (user-friendly)
   - Converts to 0-based before calling application services
   - Converts back to 1-based in responses

3. **Repository Layer**: Remains unchanged
   - Continues to use `offset` and `limit` parameters
   - No changes required to existing repository interfaces

### Data Flow

```
API Request (page=1) 
  → Handler (converts to page=0) 
    → Application Service (calculates offset=0) 
      → Repository (uses offset=0, limit=20)
        → Database Query
      ← Repository (returns Vec<T> + total)
    ← Application Service (returns (Vec<T>, total))
  ← Handler (converts page back to 1, constructs response)
← API Response (page=1, total_pages=calculated)
```

## Components and Interfaces

### 1. Application Service Interface Pattern

All pagination methods will follow this pattern:

```rust
async fn list_<entities>(
    &self,
    tenant_id: TenantId,
    page: u64,
    limit: u64,
) -> Result<(Vec<Entity>, u64)> {
    let offset = page * limit;
    let entities = self.repository.find_by_tenant_paginated(&tenant_id, offset, limit).await?;
    let total = self.repository.count_by_tenant(&tenant_id).await?;
    Ok((entities, total))
}
```

### 2. Handler Layer Pattern

Handlers will convert pagination parameters:

```rust
pub async fn list_<entities>(
    State(service): State<Arc<Service>>,
    user: AuthenticatedUser,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse> {
    // Convert from 1-based (API) to 0-based (internal)
    let page = query.page.unwrap_or(1).saturating_sub(1);
    let limit = query.limit.unwrap_or(20);
    
    let (entities, total) = service.list_entities(user.tenant_id, page, limit).await?;
    
    // Convert back to 1-based for response
    let response = ListResponse {
        data: entities.into_iter().map(to_dto).collect(),
        page: page + 1,
        limit,
        total,
        total_pages: (total + limit - 1) / limit,
    };
    
    Ok(Json(response))
}
```

### 3. Query DTO Pattern

```rust
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: u64,  // API uses 1-based
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_page() -> u64 { 1 }
fn default_limit() -> u64 { 20 }
```

### 4. Response DTO Pattern

```rust
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub page: u64,      // 1-based for API
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
}
```

## Data Models

### Service Method Signatures

#### Before Standardization

```rust
// Audit Service - uses (page-1) calculation
async fn query_logs_paginated(
    &self,
    tenant_id: Uuid,
    page: u64,              // 1-based
    page_size: u64,         // inconsistent naming
    ...
) -> Result<(Vec<AuditLog>, u64)> {
    let offset = (page - 1) * page_size;  // 1-based calculation
    ...
}

// LLM Service - uses raw offset
async fn list_configs_paginated(
    &self,
    tenant_id: TenantId,
    offset: u64,            // raw offset, not page
    limit: u64,
) -> Result<Vec<LLMConfig>>  // missing total count

// MCP Service - uses custom response type
async fn list_tools(
    &self,
    tenant_id: TenantId,
    page: Option<u32>,      // optional, 1-based
    limit: Option<u32>,
) -> Result<MCPToolListResponse>  // custom response type
```

#### After Standardization

```rust
// All services follow this pattern
async fn list_<entities>(
    &self,
    tenant_id: TenantId,
    page: u64,              // 0-based, consistent
    limit: u64,             // consistent naming
) -> Result<(Vec<Entity>, u64)> {  // consistent return type
    let offset = page * limit;  // 0-based calculation
    ...
}
```

## Error Handling

### Validation Rules

1. **Page Validation**: No explicit validation needed at application layer (u64 naturally >= 0)
2. **Limit Validation**: 
   - Minimum: 1
   - Maximum: 100 (configurable)
   - Default: 20

3. **Handler Layer Validation**:
```rust
fn validate_pagination(page: u64, limit: u64) -> Result<()> {
    if limit == 0 {
        return Err(PlatformError::ValidationError(
            "Limit must be greater than 0".to_string()
        ));
    }
    if limit > 100 {
        return Err(PlatformError::ValidationError(
            "Limit cannot exceed 100".to_string()
        ));
    }
    Ok(())
}
```

### Error Scenarios

1. **Invalid Limit**: Return `ValidationError` with clear message
2. **Repository Errors**: Propagate as-is
3. **Empty Results**: Return empty vector with total=0 (not an error)

## Testing Strategy

### Unit Tests

Each service will have tests covering:

1. **Zero-based pagination**:
```rust
#[tokio::test]
async fn test_list_first_page() {
    let (entities, total) = service.list_entities(tenant_id, 0, 20).await?;
    assert_eq!(entities.len(), min(20, total));
}
```

2. **Offset calculation**:
```rust
#[tokio::test]
async fn test_list_second_page() {
    let (entities, _) = service.list_entities(tenant_id, 1, 20).await?;
    // Verify offset=20 was used
}
```

3. **Total count accuracy**:
```rust
#[tokio::test]
async fn test_total_count() {
    let (_, total) = service.list_entities(tenant_id, 0, 20).await?;
    let actual_count = repository.count_by_tenant(tenant_id).await?;
    assert_eq!(total, actual_count);
}
```

### Integration Tests

Handler tests will verify:

1. **API uses 1-based pagination**:
```rust
#[tokio::test]
async fn test_api_pagination_starts_at_one() {
    let response = get("/api/entities?page=1&limit=20").await?;
    assert_eq!(response.page, 1);
}
```

2. **Conversion correctness**:
```rust
#[tokio::test]
async fn test_page_conversion() {
    // API page=1 should fetch first 20 items
    let response1 = get("/api/entities?page=1&limit=20").await?;
    // API page=2 should fetch next 20 items
    let response2 = get("/api/entities?page=2&limit=20").await?;
    // Verify no overlap
}
```

3. **Total pages calculation**:
```rust
#[tokio::test]
async fn test_total_pages_calculation() {
    // With 45 total items and limit=20
    let response = get("/api/entities?page=1&limit=20").await?;
    assert_eq!(response.total, 45);
    assert_eq!(response.total_pages, 3);  // ceil(45/20) = 3
}
```

## Implementation Details

### Service-Specific Changes

#### 1. AuditApplicationService

**Current**:
```rust
pub async fn query_logs_paginated(
    &self,
    tenant_id: Uuid,
    page: u64,
    page_size: u64,
    ...
) -> Result<(Vec<AuditLog>, u64)> {
    let offset = (page - 1) * page_size;
    ...
}
```

**Updated**:
```rust
pub async fn query_logs_paginated(
    &self,
    tenant_id: Uuid,
    page: u64,
    limit: u64,
    ...
) -> Result<(Vec<AuditLog>, u64)> {
    let offset = page * limit;
    ...
}
```

#### 2. ExecutionHistoryApplicationService

**Current**:
```rust
pub async fn query_executions_paginated(
    &self,
    tenant_id: Uuid,
    page: u64,
    page_size: u64,
    ...
) -> Result<(Vec<FlowExecutionHistory>, u64)> {
    let offset = (page - 1) * page_size;
    ...
}
```

**Updated**:
```rust
pub async fn query_executions_paginated(
    &self,
    tenant_id: Uuid,
    page: u64,
    limit: u64,
    ...
) -> Result<(Vec<FlowExecutionHistory>, u64)> {
    let offset = page * limit;
    ...
}
```

#### 3. SessionApplicationService

**Current**:
```rust
pub async fn list_user_sessions(
    &self,
    user_id: &UserId,
    offset: u64,
    limit: u64,
) -> Result<Vec<ChatSession>>
```

**Updated**:
```rust
pub async fn list_user_sessions(
    &self,
    tenant_id: TenantId,
    user_id: &UserId,
    page: u64,
    limit: u64,
) -> Result<(Vec<ChatSession>, u64)> {
    let offset = page * limit;
    let sessions = self.session_repo
        .find_by_user_paginated(user_id, offset, limit)
        .await?;
    let total = self.session_repo.count_by_user(user_id).await?;
    Ok((sessions, total))
}
```

#### 4. LLMApplicationService

**Current**:
```rust
async fn list_configs_paginated(
    &self,
    tenant_id: TenantId,
    offset: u64,
    limit: u64,
) -> Result<Vec<LLMConfig>>
```

**Updated**:
```rust
async fn list_configs_paginated(
    &self,
    tenant_id: TenantId,
    page: u64,
    limit: u64,
) -> Result<(Vec<LLMConfig>, u64)> {
    let offset = page * limit;
    let configs = self.config_repository
        .find_by_tenant_paginated(tenant_id, offset, limit)
        .await?;
    let total = self.config_repository.count_by_tenant(tenant_id).await?;
    Ok((configs, total))
}
```

#### 5. MCPApplicationService

**Current**:
```rust
async fn list_tools(
    &self,
    tenant_id: TenantId,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<MCPToolListResponse>
```

**Updated**:
```rust
async fn list_tools(
    &self,
    tenant_id: TenantId,
    page: u64,
    limit: u64,
) -> Result<(Vec<MCPTool>, u64)> {
    let offset = page * limit;
    let query_options = MCPToolQueryOptions::new()
        .with_tenant_id(tenant_id)
        .with_pagination(limit, offset);
    let query_result = self.tool_repository
        .find_by_options(query_options)
        .await?;
    Ok((query_result.tools, query_result.total_count))
}
```

#### 6. VectorApplicationService

**New Method**:
```rust
pub async fn list_configs(
    &self,
    tenant_id: TenantId,
    page: u64,
    limit: u64,
) -> Result<(Vec<VectorConfigEntity>, u64)> {
    let offset = page * limit;
    let configs = self.vector_config_repository
        .find_by_tenant(tenant_id)
        .await?
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    let total = self.vector_config_repository
        .count_by_tenant(tenant_id)
        .await?;
    Ok((configs, total))
}
```

### Handler Layer Updates

#### Audit Handlers

```rust
pub async fn query_audit_logs(
    State(service): State<Arc<AuditApplicationService>>,
    user: AuthenticatedUser,
    Query(request): Query<QueryAuditLogsRequest>,
) -> Result<impl IntoResponse> {
    // Convert from 1-based to 0-based
    let page = request.page.unwrap_or(1).saturating_sub(1);
    let limit = request.page_size.unwrap_or(20);

    let (logs, total) = service
        .query_logs_paginated(
            user.tenant_id.0,
            page,  // Now 0-based
            limit,
            request.user_id,
            action,
            resource_type,
            request.start_date,
            request.end_date,
        )
        .await?;

    let response = QueryAuditLogsResponse {
        logs: log_dtos,
        total,
        page: page + 1,  // Convert back to 1-based
        page_size: limit,
    };

    Ok((StatusCode::OK, Json(response)))
}
```

#### Config Handlers

```rust
pub async fn list_llm_configs(
    State(service): State<Arc<dyn LLMApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<ListConfigsQuery>,
) -> Result<impl IntoResponse> {
    let page = query.page.saturating_sub(1);  // Convert to 0-based
    let limit = query.limit;

    let (configs, total) = if let Some(provider) = query.provider {
        // For filtered queries, still need pagination
        let all_configs = service.get_configs_by_provider(user.tenant_id, &provider).await?;
        let total = all_configs.len() as u64;
        let offset = (page * limit) as usize;
        let paginated = all_configs.into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();
        (paginated, total)
    } else {
        service.list_configs_paginated(user.tenant_id, page, limit).await?
    };

    let response = PaginatedResponse {
        data: configs.iter().map(llm_config_to_response).collect(),
        page: page + 1,  // Convert back to 1-based
        limit,
        total,
        total_pages: (total + limit - 1) / limit,
    };

    Ok(Json(response))
}
```

#### MCP Handlers

```rust
pub async fn list_mcp_tools(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<MCPToolListQuery>,
) -> Result<Json<MCPToolListResponse>, PlatformError> {
    // Convert from 1-based to 0-based
    let page = query.page.unwrap_or(1).saturating_sub(1);
    let limit = query.limit.unwrap_or(20);

    let (tools, total) = service
        .list_tools(user.tenant_id, page, limit)
        .await?;

    let tool_responses: Vec<MCPToolResponse> = tools
        .iter()
        .map(|tool| tool_to_response(tool))
        .collect();

    let response = MCPToolListResponse {
        tools: tool_responses,
        total: total as u32,
        page: (page + 1) as u32,  // Convert back to 1-based
        limit: limit as u32,
        total_pages: ((total + limit - 1) / limit) as u32,
    };

    Ok(Json(response))
}
```

## Migration Strategy

### Phase 1: Update Application Services
1. Update method signatures
2. Change offset calculation
3. Add total count to return types where missing
4. Update internal documentation

### Phase 2: Update Handlers
1. Add page conversion logic
2. Update response construction
3. Maintain API compatibility (1-based)

### Phase 3: Update Tests
1. Update unit tests for 0-based pagination
2. Update integration tests
3. Add new test cases for edge cases

### Phase 4: Documentation
1. Update API documentation
2. Update internal developer guides
3. Add migration notes

## Backward Compatibility

### API Layer
- **No breaking changes**: API continues to use 1-based pagination
- Clients don't need to change their requests
- Response format remains the same

### Internal Layer
- Application services change from 1-based to 0-based
- Handlers handle the conversion
- Repository layer unchanged

## Performance Considerations

1. **No Performance Impact**: The change from `(page - 1) * limit` to `page * limit` with adjusted page values has zero performance impact

2. **Consistency Benefits**: Uniform pagination logic reduces cognitive load and potential bugs

3. **Repository Optimization**: Repository layer remains unchanged, so existing optimizations (indexes, query plans) continue to work

## Security Considerations

1. **Input Validation**: Handlers validate page and limit parameters before passing to services
2. **Tenant Isolation**: All pagination methods require tenant_id, maintaining multi-tenancy security
3. **No SQL Injection Risk**: Using parameterized queries with offset/limit (no changes to this aspect)

## Monitoring and Observability

1. **Logging**: Log pagination parameters at handler level for debugging
2. **Metrics**: Track pagination usage patterns (common page sizes, deep pagination)
3. **Alerts**: Alert on unusual pagination patterns (very large limits, very deep pages)
