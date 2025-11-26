# API Key Management System - Design Document

## Overview

The API Key Management System provides fine-grained access control for programmatic access to platform resources including Agents, Flows, MCP Tools, and Vector Stores. The system enables users to create API keys with specific resource permissions, expiration policies, and enable/disable controls. Additionally, it implements an MCP Server using the `rmcp` Rust library to expose tool listing and invocation capabilities authenticated via API keys.

### Key Design Goals

1. **Security First**: Cryptographically secure token generation with proper hashing and storage
2. **Fine-Grained Access Control**: Resource-level permissions with explicit allow-lists
3. **Audit Trail**: Comprehensive logging of all API key operations and usage
4. **MCP Integration**: Native MCP protocol support for tool access via API keys
5. **Tenant Isolation**: Strict enforcement of multi-tenant boundaries
6. **Performance**: Efficient permission checking with minimal database queries

## Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Presentation Layer                       │
│  ┌──────────────────┐  ┌─────────────────┐  ┌────────────┐ │
│  │ API Key Handlers │  │ MCP Server      │  │ Auth       │ │
│  │ (REST API)       │  │ (rmcp-based)    │  │ Middleware │ │
│  └──────────────────┘  └─────────────────┘  └────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         API Key Application Service                   │   │
│  │  - Create/List/Update/Delete API Keys                │   │
│  │  - Validate API Keys                                  │   │
│  │  - Check Resource Permissions                         │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         MCP Server Application Service                │   │
│  │  - Tool Listing (filtered by API key permissions)    │   │
│  │  - Tool Invocation (with permission checks)          │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                            │
│  ┌──────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │ API Key      │  │ Permission       │  │ API Key      │  │
│  │ Entity       │  │ Scope Value Obj  │  │ Service      │  │
│  └──────────────┘  └──────────────────┘  └──────────────┘  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         API Key Repository Interface                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │    API Key Repository Implementation (PostgreSQL)     │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │    rmcp-based MCP Server Implementation               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Component Interactions

1. **API Key Creation Flow**:
   - User → REST API Handler → Application Service → Domain Service → Repository
   - Token generated once, hashed before storage
   - Plain token returned only at creation time

2. **API Key Authentication Flow**:
   - Request → Auth Middleware → Application Service → Repository (lookup by hash)
   - Validate: exists, enabled, not expired
   - Inject auth context into request

3. **Permission Check Flow**:
   - Request → Resource Handler → Permission Check → API Key Service
   - Verify resource type and ID in permission scope
   - Enforce tenant isolation

4. **MCP Server Flow**:
   - MCP Client → rmcp Server → API Key Auth → Tool Listing/Invocation
   - Filter tools by permission scope
   - Execute tool with permission validation

## Components and Interfaces

### Domain Layer

#### 1. API Key Entity

```rust
pub struct APIKey {
    pub id: APIKeyId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: String,
    pub key_hash: String,  // SHA-256 hash of the actual key
    pub permission_scope: PermissionScope,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Design Rationale**: 
- Store only the hash of the API key for security
- Track `last_used_at` for usage analytics and security monitoring
- Separate `enabled` flag allows temporary revocation without deletion
- Optional expiration supports both time-limited and permanent keys

**Key Methods**:
- `new()`: Create new API key with validation
- `is_valid()`: Check if key is enabled and not expired
- `can_access_resource()`: Verify permission for specific resource
- `disable()`: Disable the key
- `enable()`: Re-enable the key
- `update_last_used()`: Update last usage timestamp
- `belongs_to_tenant()`: Verify tenant ownership

#### 2. Permission Scope Value Object

```rust
pub struct PermissionScope {
    pub agent_ids: Vec<AgentId>,
    pub flow_ids: Vec<FlowId>,
    pub mcp_tool_ids: Vec<MCPToolId>,
    pub vector_store_ids: Vec<VectorConfigId>,
}
```

**Design Rationale**:
- Explicit allow-list approach (deny by default)
- Empty list for a resource type means no access to that type
- Stored as JSONB in database for flexibility
- Immutable value object ensures consistency

**Key Methods**:
- `new()`: Create with validation
- `can_access_agent()`: Check agent access
- `can_access_flow()`: Check flow access
- `can_access_mcp_tool()`: Check MCP tool access
- `can_access_vector_store()`: Check vector store access
- `is_empty()`: Check if scope grants any permissions
- `merge()`: Combine multiple scopes (for future use)

#### 3. API Key Token Value Object

```rust
pub struct APIKeyToken(String);
```

**Design Rationale**:
- Encapsulates token generation and validation logic
- Format: `pk_` prefix + 32 bytes of cryptographically secure random data (base64url encoded)
- Example: `pk_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6`
- Prefix allows easy identification and future versioning

**Key Methods**:
- `generate()`: Create new cryptographically secure token
- `hash()`: Generate SHA-256 hash for storage
- `validate_format()`: Verify token format
- `as_str()`: Get string representation

#### 4. API Key Domain Service

```rust
pub trait APIKeyService {
    async fn create_api_key(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        name: String,
        permission_scope: PermissionScope,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(APIKey, APIKeyToken), PlatformError>;
    
    async fn validate_and_get_key(
        &self,
        token: &str,
    ) -> Result<APIKey, PlatformError>;
    
    async fn check_resource_permission(
        &self,
        api_key: &APIKey,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<bool, PlatformError>;
}
```

**Design Rationale**:
- Encapsulates business logic for API key operations
- Separates token generation from storage
- Provides centralized permission checking logic

#### 5. API Key Repository Interface

```rust
pub trait APIKeyRepository: Send + Sync {
    async fn save(&self, api_key: &APIKey) -> Result<(), PlatformError>;
    async fn find_by_id(&self, id: APIKeyId) -> Result<Option<APIKey>, PlatformError>;
    async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<APIKey>, PlatformError>;
    async fn find_by_tenant(&self, tenant_id: TenantId, options: QueryOptions) -> Result<APIKeyQueryResult, PlatformError>;
    async fn find_by_user(&self, user_id: UserId, options: QueryOptions) -> Result<APIKeyQueryResult, PlatformError>;
    async fn update(&self, api_key: &APIKey) -> Result<(), PlatformError>;
    async fn delete(&self, id: APIKeyId) -> Result<(), PlatformError>;
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError>;
}
```

### Application Layer

#### 1. API Key Application Service

```rust
pub struct APIKeyApplicationService {
    repository: Arc<dyn APIKeyRepository>,
    audit_service: Arc<dyn AuditApplicationService>,
}
```

**Responsibilities**:
- Orchestrate API key CRUD operations
- Coordinate with audit service for logging
- Handle DTO conversions
- Enforce business rules

**Key Methods**:
- `create_api_key()`: Create new API key with audit logging
- `list_api_keys()`: List keys with pagination and filtering
- `get_api_key()`: Get single key details (without token)
- `update_api_key()`: Update key properties (name, expiration, enabled status)
- `delete_api_key()`: Delete key with audit logging
- `validate_api_key()`: Validate token and return auth context
- `check_permission()`: Check resource access permission

#### 2. MCP Server Application Service

```rust
pub struct MCPServerApplicationService {
    api_key_service: Arc<dyn APIKeyApplicationService>,
    mcp_tool_repository: Arc<dyn MCPToolRepository>,
    mcp_proxy_service: Arc<dyn MCPProxyService>,
}
```

**Responsibilities**:
- Handle MCP protocol requests
- Filter tools based on API key permissions
- Execute tool calls with permission validation
- Integrate with existing MCP infrastructure

**Key Methods**:
- `list_tools()`: Return tools accessible by API key
- `call_tool()`: Execute tool with permission check
- `get_tool_schema()`: Return tool schema if accessible

### Presentation Layer

#### 1. API Key REST Handlers

**Endpoints**:
- `POST /api/v1/api-keys`: Create new API key
- `GET /api/v1/api-keys`: List API keys (paginated)
- `GET /api/v1/api-keys/:id`: Get API key details
- `PATCH /api/v1/api-keys/:id`: Update API key
- `DELETE /api/v1/api-keys/:id`: Delete API key

**Request/Response DTOs**:
```rust
pub struct CreateAPIKeyRequest {
    pub name: String,
    pub permission_scope: PermissionScopeDTO,
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct CreateAPIKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,  // Only returned once
    pub permission_scope: PermissionScopeDTO,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct APIKeyDTO {
    pub id: Uuid,
    pub name: String,
    pub permission_scope: PermissionScopeDTO,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UpdateAPIKeyRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

#### 2. API Key Authentication Middleware

```rust
pub async fn api_key_auth_middleware(
    State(api_key_service): State<Arc<dyn APIKeyApplicationService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, PlatformError>
```

**Design Rationale**:
- Parallel to existing JWT auth middleware
- Extracts API key from `Authorization: Bearer <api_key>` header
- Validates key and injects `APIKeyAuthContext` into request extensions
- Can be used alongside or instead of JWT auth depending on endpoint

**Auth Context**:
```rust
pub struct APIKeyAuthContext {
    pub api_key_id: APIKeyId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub permission_scope: PermissionScope,
}
```

#### 3. Resource Permission Middleware

```rust
pub fn require_resource_permission(
    resource_type: ResourceType,
) -> impl Fn(Request, Next) -> Future<Output = Result<Response, PlatformError>>
```

**Design Rationale**:
- Applied to resource-specific endpoints
- Extracts resource ID from path parameters
- Checks permission via API key auth context
- Returns 403 Forbidden if permission denied

#### 4. MCP Server Handler (rmcp-based)

```rust
pub struct RMCPServerHandler {
    mcp_service: Arc<MCPServerApplicationService>,
}
```

**Design Rationale**:
- Uses `rmcp` library for MCP protocol implementation
- Implements MCP server traits for tool listing and invocation
- Authenticates via API key in MCP connection metadata
- Filters and executes tools based on permissions

**MCP Protocol Methods**:
- `list_tools()`: Returns filtered tool list
- `call_tool()`: Executes tool with permission check
- `get_tool_schema()`: Returns tool schema if accessible

### Infrastructure Layer

#### 1. API Key Repository Implementation

**Database Schema**:
```sql
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(64) NOT NULL UNIQUE,  -- SHA-256 hash
    permission_scope JSONB NOT NULL DEFAULT '{"agent_ids":[],"flow_ids":[],"mcp_tool_ids":[],"vector_store_ids":[]}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_api_keys_tenant_id ON api_keys(tenant_id);
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_enabled ON api_keys(enabled) WHERE enabled = true;
CREATE INDEX idx_api_keys_expires_at ON api_keys(expires_at) WHERE expires_at IS NOT NULL;
```

**Design Rationale**:
- `key_hash` is unique and indexed for fast lookups during authentication
- JSONB for `permission_scope` allows flexible querying and future extensions
- Indexes on `tenant_id`, `user_id`, and `enabled` for efficient filtering
- Partial index on `expires_at` for active keys only
- Cascade delete ensures cleanup when tenant/user is deleted

#### 2. rmcp MCP Server Implementation

**Integration Points**:
- Reuse existing `MCPProxyService` for tool execution
- Reuse existing `MCPToolRepository` for tool metadata
- Add API key authentication layer
- Filter tools based on permission scope

**Configuration**:
```rust
pub struct RMCPServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
}
```

## Data Models

### API Key Lifecycle States

```
┌─────────┐
│ Created │ (enabled=true, not expired)
└────┬────┘
     │
     ├──────────────┐
     │              │
     ▼              ▼
┌─────────┐    ┌──────────┐
│ Active  │    │ Disabled │ (enabled=false)
└────┬────┘    └────┬─────┘
     │              │
     │              │ enable()
     │◄─────────────┘
     │
     │ (time passes)
     ▼
┌──────────┐
│ Expired  │ (expires_at < now)
└────┬─────┘
     │
     │ delete()
     ▼
┌─────────┐
│ Deleted │
└─────────┘
```

### Permission Scope Structure

```json
{
  "agent_ids": ["uuid1", "uuid2"],
  "flow_ids": ["uuid3", "uuid4"],
  "mcp_tool_ids": ["uuid5", "uuid6"],
  "vector_store_ids": ["uuid7", "uuid8"]
}
```

**Design Rationale**:
- Empty arrays mean no access to that resource type
- Explicit allow-list approach (secure by default)
- Easy to serialize/deserialize from JSONB
- Supports future resource types without schema changes

## Error Handling

### Error Types

```rust
pub enum APIKeyError {
    InvalidToken,
    TokenExpired,
    KeyDisabled,
    KeyNotFound,
    PermissionDenied { resource_type: String, resource_id: Uuid },
    InvalidPermissionScope,
    DuplicateKeyName,
}
```

### Error Responses

- `401 Unauthorized`: Invalid, expired, or disabled API key
- `403 Forbidden`: Valid key but insufficient permissions
- `404 Not Found`: API key not found
- `409 Conflict`: Duplicate key name for user
- `422 Unprocessable Entity`: Invalid permission scope

### Error Handling Strategy

1. **Authentication Errors**: Return generic 401 to avoid leaking information
2. **Authorization Errors**: Return 403 with resource type (but not ID)
3. **Audit All Failures**: Log all authentication and authorization failures
4. **Rate Limiting**: Apply rate limiting to prevent brute force attacks

## Testing Strategy

### Unit Tests

1. **Domain Layer**:
   - API Key entity validation
   - Permission scope logic
   - Token generation and hashing
   - State transitions (enable/disable)

2. **Application Layer**:
   - API key creation with audit logging
   - Permission checking logic
   - Token validation
   - Expiration handling

3. **Infrastructure Layer**:
   - Repository CRUD operations
   - Query filtering and pagination
   - Database constraint enforcement

### Integration Tests

1. **API Key Management**:
   - Create API key and verify token returned once
   - List API keys with pagination
   - Update API key properties
   - Delete API key and verify inaccessible

2. **Authentication Flow**:
   - Authenticate with valid API key
   - Reject invalid/expired/disabled keys
   - Verify auth context injection

3. **Permission Enforcement**:
   - Access allowed resources successfully
   - Deny access to resources not in scope
   - Enforce tenant isolation

4. **MCP Server**:
   - List tools filtered by permissions
   - Execute tool with valid permissions
   - Deny tool execution without permissions
   - Handle MCP protocol errors

### Security Tests

1. **Token Security**:
   - Verify cryptographic randomness
   - Confirm hash storage (never plain text)
   - Test token format validation

2. **Permission Bypass Attempts**:
   - Attempt cross-tenant access
   - Attempt access with disabled key
   - Attempt access with expired key
   - Attempt access to non-permitted resources

3. **Audit Logging**:
   - Verify all operations logged
   - Verify failed attempts logged
   - Verify sensitive data not logged

## Security Considerations

### Token Generation

- Use `ring::rand::SystemRandom` for cryptographically secure random generation
- Generate 32 bytes (256 bits) of entropy
- Encode as base64url for URL-safe representation
- Add `pk_` prefix for identification

### Token Storage

- Never store plain text tokens
- Use SHA-256 for hashing (fast, sufficient for this use case)
- Store only the hash in database
- Return plain token only once at creation

### Authentication

- Constant-time comparison for hash matching
- Rate limit authentication attempts
- Log all authentication failures
- Support token revocation via disable/delete

### Authorization

- Deny by default (empty scope = no access)
- Check both resource type and specific ID
- Enforce tenant isolation at every check
- Validate resource existence before permission check

### Audit Trail

- Log all API key operations (create, update, delete, enable, disable)
- Log all authentication attempts (success and failure)
- Log all authorization failures with resource type
- Include IP address and timestamp
- Never log actual tokens or hashes

### Rate Limiting

- Apply rate limiting to API key authentication endpoints
- Apply rate limiting to MCP server connections
- Use sliding window algorithm
- Different limits for different operations

## Performance Considerations

### Database Optimization

1. **Indexes**:
   - `key_hash` for fast authentication lookups
   - `tenant_id` for tenant-scoped queries
   - `user_id` for user-scoped queries
   - Partial index on `enabled` for active keys
   - Partial index on `expires_at` for non-expired keys

2. **Query Patterns**:
   - Single query for authentication (lookup by hash)
   - Efficient pagination with offset/limit
   - Filter expired keys at database level
   - Use JSONB operators for permission scope queries

### Caching Strategy

1. **API Key Cache**:
   - Cache validated API keys in Redis
   - TTL based on expiration time
   - Invalidate on update/delete/disable
   - Key: `api_key:hash:<hash>`

2. **Permission Cache**:
   - Cache permission checks for frequently accessed resources
   - Short TTL (5 minutes)
   - Invalidate on permission scope changes
   - Key: `api_key:perm:<key_id>:<resource_type>:<resource_id>`

### MCP Server Performance

1. **Connection Pooling**: Reuse database connections
2. **Tool Filtering**: Filter at database level when possible
3. **Batch Operations**: Support batch tool listing
4. **Async Processing**: Use async/await throughout

## Migration Strategy

### Phase 1: Core Infrastructure
- Database schema and migrations
- Domain entities and value objects
- Repository implementation
- Basic CRUD operations

### Phase 2: Authentication & Authorization
- API key generation and validation
- Authentication middleware
- Permission checking logic
- Audit logging integration

### Phase 3: REST API
- API key management endpoints
- Request/response DTOs
- Error handling
- API documentation

### Phase 4: MCP Server
- rmcp integration
- Tool listing with filtering
- Tool invocation with permissions
- MCP protocol error handling

### Phase 5: Testing & Documentation
- Comprehensive test suite
- Security testing
- Performance testing
- User documentation

## Future Enhancements

1. **Scoped Permissions**: Add read/write/execute granularity
2. **IP Whitelisting**: Restrict API key usage to specific IPs
3. **Usage Quotas**: Limit requests per time period
4. **Key Rotation**: Support automatic key rotation
5. **Webhook Integration**: Notify on key usage events
6. **Analytics Dashboard**: Visualize API key usage patterns
7. **Multi-Key Support**: Allow multiple active keys per user
8. **Key Templates**: Predefined permission templates for common use cases

## Dependencies

### Rust Crates

- `rmcp`: MCP server implementation
- `ring`: Cryptographic operations (random generation, hashing)
- `base64`: Token encoding
- `sha2`: SHA-256 hashing
- `serde_json`: JSON serialization for permission scope
- `chrono`: Date/time handling
- `uuid`: ID generation

### Existing Platform Components

- `AuditApplicationService`: Audit logging
- `MCPToolRepository`: Tool metadata access
- `MCPProxyService`: Tool execution
- `AuthApplicationService`: User authentication context
- Database connection pool
- Redis cache (optional)

## Open Questions

1. **Key Rotation**: Should we support automatic key rotation? If so, what's the rotation policy?
2. **Rate Limiting**: What are appropriate rate limits for API key operations?
3. **Caching**: Should we implement Redis caching for API keys, or rely on database performance?
4. **MCP Server Deployment**: Should the MCP server run as a separate process or embedded in the main application?
5. **Permission Granularity**: Do we need read/write/execute permissions, or is resource-level access sufficient for MVP?
6. **Key Expiration Notifications**: Should we notify users before keys expire?
7. **Usage Analytics**: What metrics should we track for API key usage?

