# Implementation Plan

- [x] 1. Create domain layer foundations for API key management
  - Create value objects for APIKeyId, APIKeyToken, and PermissionScope with validation logic
  - Implement APIKey entity with state management methods (new, is_valid, can_access_resource, enable, disable, update_last_used)
  - Add ResourceType enum to support agent, flow, mcp_tool, and vector_store resource types
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4_

- [x] 2. Implement API key repository interface and database layer
  - [x] 2.1 Define APIKeyRepository trait with CRUD and query methods
    - Define methods: save, find_by_id, find_by_key_hash, find_by_tenant, find_by_user, update, delete, count_by_tenant
    - Add QueryOptions struct for pagination and filtering
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 2.2 Create database migration for api_keys table
    - Create migration file with table schema including all columns (id, tenant_id, user_id, name, key_hash, permission_scope, enabled, expires_at, last_used_at, created_at, updated_at)
    - Add indexes for key_hash, tenant_id, user_id, enabled, and expires_at
    - Add foreign key constraints to tenants and users tables with CASCADE delete
    - _Requirements: 1.4, 4.4, 6.4, 7.1_
  
  - [x] 2.3 Implement APIKeyRepositoryImpl with PostgreSQL/SeaORM
    - Create database entity mapping for api_keys table
    - Implement all repository trait methods with proper error handling
    - Add JSONB serialization/deserialization for permission_scope field
    - Implement pagination and filtering logic in query methods
    - _Requirements: 5.2, 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 3. Build API key domain service with token generation and validation
  - Implement APIKeyToken value object with cryptographically secure token generation using ring::rand::SystemRandom
  - Add token hashing method using SHA-256 (sha2 crate)
  - Create APIKeyService trait with methods: create_api_key, validate_and_get_key, check_resource_permission
  - Implement APIKeyDomainService with business logic for token generation, validation, and permission checking
  - _Requirements: 1.1, 1.5, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3_

- [x] 4. Create application layer services for API key operations
  - [x] 4.1 Implement APIKeyApplicationService for CRUD operations
    - Create service struct with dependencies on APIKeyRepository and AuditApplicationService
    - Implement create_api_key method with token generation and audit logging
    - Implement list_api_keys method with pagination support
    - Implement get_api_key method (without returning token value)
    - Implement update_api_key method for name, enabled status, and expiration
    - Implement delete_api_key method with audit logging
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2, 7.1, 7.2, 7.3, 7.4, 8.1, 8.2, 8.3, 8.4_
  
  - [x] 4.2 Add authentication and permission validation methods
    - Implement validate_api_key method that checks token hash, enabled status, and expiration
    - Implement check_permission method that verifies resource access based on permission scope
    - Add update_last_used method to track API key usage
    - Integrate with audit service to log authentication attempts and authorization failures
    - _Requirements: 4.1, 4.2, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 11.2, 11.4_

- [ ] 5. Implement DTOs for API key management
  - Create CreateAPIKeyRequest, CreateAPIKeyResponse, APIKeyDTO, UpdateAPIKeyRequest DTOs
  - Create PermissionScopeDTO with agent_ids, flow_ids, mcp_tool_ids, vector_store_ids fields
  - Add APIKeyAuthContext struct for request context injection
  - Implement conversion methods between domain entities and DTOs
  - _Requirements: 1.2, 1.3, 7.2, 7.3_

- [x] 6. Build REST API handlers for API key management
  - [x] 6.1 Create API key CRUD handlers
    - Implement POST /api/v1/api-keys handler for creating API keys (returns token once)
    - Implement GET /api/v1/api-keys handler for listing API keys with pagination
    - Implement GET /api/v1/api-keys/:id handler for getting single API key details
    - Implement PATCH /api/v1/api-keys/:id handler for updating API key properties
    - Implement DELETE /api/v1/api-keys/:id handler for deleting API keys
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2, 7.1, 7.2, 7.3, 8.1, 8.2, 8.3, 8.4_
  
  - [x] 6.2 Add request validation and error handling
    - Validate request payloads (name length, permission scope format, expiration date)
    - Handle duplicate key name errors
    - Return appropriate HTTP status codes (401, 403, 404, 409, 422)
    - Format error responses consistently
    - _Requirements: 5.5, 7.2_

- [x] 7. Implement authentication middleware for API keys
  - Create api_key_auth_middleware that extracts API key from Authorization header
  - Validate API key format (pk_ prefix)
  - Call APIKeyApplicationService to validate token and get auth context
  - Inject APIKeyAuthContext into request extensions
  - Handle authentication errors with appropriate status codes
  - Log authentication attempts (success and failure)
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 11.2_

- [x] 8. Create resource permission middleware
  - Implement require_resource_permission middleware factory function
  - Extract resource type and ID from request path parameters
  - Retrieve APIKeyAuthContext from request extensions
  - Call permission checking logic to verify access
  - Return 403 Forbidden if permission denied
  - Log authorization failures with resource type
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 11.4_

- [x] 9. Integrate audit logging for API key operations
  - Log API key creation events with user and tenant information
  - Log API key state changes (enable, disable, delete)
  - Log authentication attempts with success/failure status
  - Log authorization failures with resource type and API key identifier
  - Include timestamps and IP addresses in all audit logs
  - Ensure tokens and hashes are never logged
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [x] 10. Build MCP server application service with API key authentication
  - [x] 10.1 Create MCPServerApplicationService
    - Create service struct with dependencies on APIKeyApplicationService, MCPToolRepository, and MCPProxyService
    - Implement list_tools method that filters tools based on API key permission scope
    - Implement call_tool method that validates permissions before execution
    - Implement get_tool_schema method with permission checking
    - _Requirements: 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4_
  
  - [x] 10.2 Integrate with existing MCP infrastructure
    - Reuse MCPProxyService for tool execution
    - Reuse MCPToolRepository for tool metadata retrieval
    - Add permission filtering logic to tool queries
    - Handle MCP protocol errors appropriately
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 11. Implement rmcp-based MCP server handler
  - Add rmcp crate dependency to Cargo.toml
  - Create RMCPServerHandler struct implementing rmcp server traits
  - Implement MCP protocol methods: list_tools, call_tool
  - Add API key authentication to MCP connection handling
  - Configure server with host, port, max_connections, and timeout settings
  - Wire up RMCPServerHandler with MCPServerApplicationService
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4_

- [x] 12. Add API routes and wire up all components
  - Register API key routes in router with appropriate middleware
  - Apply JWT authentication middleware to API key management endpoints
  - Apply API key authentication middleware to resource endpoints (optional based on use case)
  - Configure MCP server startup in main application
  - Update dependency injection container with new services
  - _Requirements: 1.1, 5.1, 6.1, 9.1_

- [x] 13. Create integration tests for API key management
  - [x] 13.1 Test API key CRUD operations
    - Test creating API key and verify token returned once
    - Test listing API keys with pagination and filtering
    - Test getting single API key details (verify token not returned)
    - Test updating API key properties (name, enabled, expiration)
    - Test deleting API key and verify subsequent access denied
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2, 7.1, 7.2, 7.3, 8.1, 8.2, 8.3, 8.4_
  
  - [x] 13.2 Test authentication and authorization flows
    - Test successful authentication with valid API key
    - Test authentication failure with invalid token
    - Test authentication failure with expired key
    - Test authentication failure with disabled key
    - Test authorization success for permitted resources
    - Test authorization failure for non-permitted resources
    - Test tenant isolation enforcement
    - _Requirements: 4.1, 4.2, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4_
  
  - [x] 13.3 Test MCP server functionality
    - Test listing tools filtered by API key permissions
    - Test tool invocation with valid permissions
    - Test tool invocation denial without permissions
    - Test MCP protocol error handling
    - Test API key authentication in MCP connections
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4_

- [ ]* 14. Add unit tests for core components
  - [ ]* 14.1 Test domain entities and value objects
    - Test APIKeyToken generation and hashing
    - Test PermissionScope validation and access checking
    - Test APIKey entity state transitions (enable/disable)
    - Test APIKey validation logic (expiration, enabled status)
    - _Requirements: 1.1, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2_
  
  - [ ]* 14.2 Test application service logic
    - Test API key creation with audit logging
    - Test token validation logic
    - Test permission checking for different resource types
    - Test expiration handling
    - Test error handling for various failure scenarios
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 11.1, 11.2, 11.3, 11.4_
  
  - [ ]* 14.3 Test repository operations
    - Test CRUD operations with mock database
    - Test query filtering and pagination
    - Test unique constraint enforcement on key_hash
    - Test cascade delete behavior
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2_

- [ ]* 15. Implement security hardening
  - Add rate limiting to API key authentication endpoints
  - Implement constant-time comparison for hash matching
  - Add request validation to prevent injection attacks
  - Implement IP address logging for security monitoring
  - Add security headers to API responses
  - _Requirements: 5.5, 11.2, 11.4, 11.5_

- [ ]* 16. Add API documentation
  - Document all API key management endpoints with request/response examples
  - Document authentication header format for API keys
  - Document permission scope structure and resource types
  - Create usage examples for common scenarios
  - Document MCP server connection and authentication
  - _Requirements: 1.1, 1.2, 1.3, 5.1, 9.1, 9.2_

