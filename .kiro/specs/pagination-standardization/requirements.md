# Requirements Document

## Introduction

This feature aims to standardize all pagination list interfaces across the application services to follow a consistent pattern. Currently, different services use different pagination approaches (page starting from 0 vs 1, different parameter names, different return types), which creates confusion and inconsistency in the codebase. The goal is to align all pagination interfaces with the pattern established by `FlowApplicationService::list_flows`.

## Requirements

### Requirement 1: Standardize Pagination Parameters

**User Story:** As a developer, I want all pagination list methods to use consistent parameter names and types, so that I can easily understand and use any pagination interface in the codebase.

#### Acceptance Criteria

1. WHEN implementing a pagination list method THEN it SHALL accept `tenant_id: TenantId, page: u64, limit: u64` as parameters
2. WHEN calculating offset THEN the system SHALL use the formula `offset = page * limit`
3. WHEN the first page is requested THEN the system SHALL expect `page = 0` (zero-based indexing)
4. WHEN a pagination method is called THEN it SHALL NOT use `page_size` as a parameter name, but SHALL use `limit` instead

### Requirement 2: Standardize Return Types

**User Story:** As a developer, I want all pagination list methods to return data in a consistent format, so that I can handle pagination results uniformly across the application.

#### Acceptance Criteria

1. WHEN a pagination list method returns data THEN it SHALL return `Result<(Vec<T>, u64)>` where T is the entity type and u64 is the total count
2. WHEN returning pagination results THEN the first element of the tuple SHALL be the list of entities for the current page
3. WHEN returning pagination results THEN the second element of the tuple SHALL be the total count of all entities matching the filter criteria
4. WHEN an error occurs THEN the method SHALL return an appropriate `Result::Err` with a `PlatformError`

### Requirement 3: Update Audit Application Service

**User Story:** As a developer, I want the audit application service pagination to follow the standard pattern, so that it's consistent with other services.

#### Acceptance Criteria

1. WHEN calling `query_logs_paginated` THEN it SHALL accept `page: u64` starting from 0 instead of 1
2. WHEN calculating offset in `query_logs_paginated` THEN it SHALL use `offset = page * limit` instead of `(page - 1) * page_size`
3. WHEN calling `query_logs_paginated` THEN the parameter SHALL be named `limit` instead of `page_size`
4. WHEN `query_logs_paginated` returns data THEN it SHALL maintain the existing return type `Result<(Vec<AuditLog>, u64)>`

### Requirement 4: Update Execution History Application Service

**User Story:** As a developer, I want the execution history application service pagination to follow the standard pattern, so that it's consistent with other services.

#### Acceptance Criteria

1. WHEN calling `query_executions_paginated` THEN it SHALL accept `page: u64` starting from 0 instead of 1
2. WHEN calculating offset in `query_executions_paginated` THEN it SHALL use `offset = page * limit` instead of `(page - 1) * page_size`
3. WHEN calling `query_executions_paginated` THEN the parameter SHALL be named `limit` instead of `page_size`
4. WHEN `query_executions_paginated` returns data THEN it SHALL maintain the existing return type `Result<(Vec<FlowExecutionHistory>, u64)>`

### Requirement 5: Update Session Application Service

**User Story:** As a developer, I want the session application service to have a standardized pagination interface, so that it's consistent with other services.

#### Acceptance Criteria

1. WHEN listing user sessions THEN there SHALL be a method `list_user_sessions` that accepts `tenant_id: TenantId, user_id: &UserId, page: u64, limit: u64`
2. WHEN calling `list_user_sessions` THEN it SHALL calculate offset using `offset = page * limit`
3. WHEN `list_user_sessions` returns data THEN it SHALL return `Result<(Vec<ChatSession>, u64)>` with the list and total count
4. WHEN the existing `list_user_sessions` method is updated THEN it SHALL replace the current `offset, limit` parameters with `page, limit`

### Requirement 6: Update LLM Application Service

**User Story:** As a developer, I want the LLM application service pagination to follow the standard pattern, so that it's consistent with other services.

#### Acceptance Criteria

1. WHEN calling `list_configs_paginated` THEN it SHALL accept `page: u64` starting from 0 instead of using raw offset
2. WHEN calling `list_configs_paginated` THEN it SHALL calculate offset internally using `offset = page * limit`
3. WHEN `list_configs_paginated` returns data THEN it SHALL return `Result<(Vec<LLMConfig>, u64)>` with the list and total count
4. WHEN the method signature is updated THEN it SHALL change from `(tenant_id, offset, limit)` to `(tenant_id, page, limit)`

### Requirement 7: Update MCP Application Service

**User Story:** As a developer, I want the MCP application service pagination to follow the standard pattern, so that it's consistent with other services.

#### Acceptance Criteria

1. WHEN calling `list_tools` THEN it SHALL accept `page: u64` and `limit: u64` as required parameters (not Optional)
2. WHEN calling `list_tools` THEN page SHALL start from 0 instead of 1
3. WHEN calculating offset in `list_tools` THEN it SHALL use `offset = page * limit` instead of `(page - 1) * limit`
4. WHEN `list_tools` returns data THEN it SHALL return `Result<(Vec<MCPTool>, u64)>` instead of `MCPToolListResponse`
5. WHEN the method signature is updated THEN the DTO response type SHALL be moved to the handler layer

### Requirement 8: Add Pagination to Vector Application Service

**User Story:** As a developer, I want the vector application service to have a standardized pagination interface for listing configurations, so that it's consistent with other services.

#### Acceptance Criteria

1. WHEN listing vector configurations THEN there SHALL be a method `list_configs` that accepts `tenant_id: TenantId, page: u64, limit: u64`
2. WHEN calling `list_configs` THEN it SHALL calculate offset using `offset = page * limit`
3. WHEN `list_configs` returns data THEN it SHALL return `Result<(Vec<VectorConfigEntity>, u64)>` with the list and total count
4. WHEN the method is implemented THEN it SHALL use the repository's pagination support

### Requirement 9: Update Handler Layer

**User Story:** As a developer, I want the handler layer to properly convert between zero-based pagination (application layer) and one-based pagination (API layer), so that the API remains user-friendly while the internal implementation is consistent.

#### Acceptance Criteria

1. WHEN an API endpoint receives a page parameter THEN it SHALL accept page numbers starting from 1 (user-friendly)
2. WHEN calling an application service from a handler THEN it SHALL convert the page number from 1-based to 0-based by subtracting 1
3. WHEN returning pagination results from a handler THEN it SHALL convert the page number back to 1-based for the API response
4. WHEN a handler constructs a pagination response THEN it SHALL include `page`, `limit`, `total`, and `total_pages` fields

### Requirement 10: Maintain Backward Compatibility

**User Story:** As a developer, I want to ensure that existing functionality continues to work after the pagination standardization, so that no regressions are introduced.

#### Acceptance Criteria

1. WHEN updating pagination methods THEN all existing tests SHALL continue to pass after adjusting for the new pagination convention
2. WHEN the changes are complete THEN the API behavior SHALL remain the same from the client's perspective (1-based pagination)
3. WHEN repository methods are called THEN they SHALL continue to work with offset and limit parameters
4. WHEN filter criteria are applied THEN pagination SHALL work correctly with all existing filters
