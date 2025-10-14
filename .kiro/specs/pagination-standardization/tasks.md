# Implementation Plan

- [x] 1. Update AuditApplicationService pagination
  - Update `query_logs_paginated` method signature to use `page` and `limit` parameters
  - Change offset calculation from `(page - 1) * page_size` to `page * limit`
  - Ensure return type is `Result<(Vec<AuditLog>, u64)>`
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4_

- [x] 2. Update ExecutionHistoryApplicationService pagination
  - Update `query_executions_paginated` method signature to use `page` and `limit` parameters
  - Change offset calculation from `(page - 1) * page_size` to `page * limit`
  - Ensure return type is `Result<(Vec<FlowExecutionHistory>, u64)>`
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 4.1, 4.2, 4.3, 4.4_

- [x] 3. Update SessionApplicationService pagination
  - Update `list_user_sessions` method signature to accept `tenant_id`, `user_id`, `page`, and `limit`
  - Implement offset calculation using `offset = page * limit`
  - Update return type to `Result<(Vec<ChatSession>, u64)>` with total count
  - Add call to `count_by_user` repository method to get total count
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 5.1, 5.2, 5.3, 5.4_

- [x] 4. Update LLMApplicationService pagination
  - Update `list_configs_paginated` method signature to use `page` instead of `offset`
  - Implement offset calculation using `offset = page * limit`
  - Update return type to `Result<(Vec<LLMConfig>, u64)>` with total count
  - Add call to `count_by_tenant` repository method to get total count
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 6.1, 6.2, 6.3, 6.4_

- [x] 5. Update MCPApplicationService pagination
  - Update `list_tools` method signature to use required `page: u64` and `limit: u64` parameters
  - Change page calculation from `(page - 1) * limit` to `page * limit`
  - Update return type from `Result<MCPToolListResponse>` to `Result<(Vec<MCPTool>, u64)>`
  - Remove DTO construction logic from service (move to handler)
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 6. Add pagination to VectorApplicationService
  - Create new `list_configs` method with signature `(tenant_id: TenantId, page: u64, limit: u64)`
  - Implement offset calculation using `offset = page * limit`
  - Implement pagination logic using skip/take or repository pagination
  - Return `Result<(Vec<VectorConfigEntity>, u64)>` with list and total count
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 8.1, 8.2, 8.3, 8.4_

- [x] 7. Update audit handlers for pagination conversion
  - Update `query_audit_logs` handler to convert page from 1-based to 0-based
  - Implement conversion: `let page = request.page.unwrap_or(1).saturating_sub(1)`
  - Update response to convert page back to 1-based: `page: page + 1`
  - Calculate and include `total_pages` in response
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 8. Update execution history handlers for pagination conversion
  - Update execution history list handler to convert page from 1-based to 0-based
  - Implement conversion: `let page = request.page.unwrap_or(1).saturating_sub(1)`
  - Update response to convert page back to 1-based
  - Calculate and include `total_pages` in response
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 9. Update session handlers for pagination conversion
  - Create or update session list handler to convert page from 1-based to 0-based
  - Implement conversion: `let page = request.page.unwrap_or(1).saturating_sub(1)`
  - Update response to convert page back to 1-based
  - Calculate and include `total_pages` in response
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 10. Update config handlers for pagination conversion
  - Update `list_llm_configs` handler to convert page from 1-based to 0-based
  - Update `list_vector_configs` handler to convert page from 1-based to 0-based
  - Implement conversion: `let page = query.page.saturating_sub(1)` (with default of 1)
  - Update responses to convert page back to 1-based
  - Calculate and include `total_pages` in responses
  - Handle provider filtering with pagination
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 11. Update MCP handlers for pagination conversion
  - Update `list_mcp_tools` handler to convert page from 1-based to 0-based
  - Implement conversion: `let page = query.page.unwrap_or(1).saturating_sub(1)`
  - Move DTO construction logic from service to handler
  - Update response to convert page back to 1-based
  - Calculate and include `total_pages` in response
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 12. Update flow handlers for consistency
  - Review `list_flows` handler to ensure it follows the pagination conversion pattern
  - Verify page conversion from 1-based to 0-based
  - Verify response includes all required pagination fields
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 13. Update application service tests
  - Update AuditApplicationService tests for 0-based pagination
  - Update ExecutionHistoryApplicationService tests for 0-based pagination
  - Update SessionApplicationService tests for 0-based pagination
  - Update LLMApplicationService tests for 0-based pagination
  - Update MCPApplicationService tests for 0-based pagination
  - Add VectorApplicationService pagination tests
  - Verify offset calculation in all tests
  - Verify total count accuracy in all tests
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [x] 14. Update handler integration tests
  - Update audit handler tests to verify 1-based API pagination
  - Update execution history handler tests to verify 1-based API pagination
  - Update session handler tests to verify 1-based API pagination
  - Update config handler tests to verify 1-based API pagination
  - Update MCP handler tests to verify 1-based API pagination
  - Verify page conversion correctness in all handlers
  - Verify total_pages calculation in all handlers
  - Test edge cases (page=0, page=1, empty results)
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [x] 15. Update API documentation
  - Document pagination parameters (page starts from 1 in API)
  - Document pagination response format
  - Add examples for pagination requests
  - Document limit constraints (min=1, max=100, default=20)
  - Update OpenAPI/Swagger specs if applicable
  - _Requirements: 9.1, 9.2, 9.3, 9.4_
