# Agent Platform API Documentation

## Overview

This document describes the REST API endpoints for the Agent Platform. All endpoints require authentication via JWT token unless otherwise specified.

## Authentication

All authenticated endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

## Base URL

```
http://localhost:8080/api
```

## API Endpoints

### Authentication

#### POST /auth/login
Login to the platform and receive a JWT token.

**Request Body:**
```json
{
  "tenant_id": "uuid",
  "username": "string",
  "password": "string"
}
```

**Response:**
```json
{
  "token": "string",
  "user": {
    "id": "uuid",
    "username": "string",
    "nickname": "string"
  },
  "expires_at": "timestamp"
}
```

### Flow Management

#### POST /flows
Create a new flow.

**Request Body:**
```json
{
  "name": "string",
  "description": "string (optional)"
}
```

#### GET /flows
List all flows for the authenticated tenant.

**Query Parameters:**
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `limit`: number (default: 20, min: 1, max: 100) - Items per page

**Response:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "string",
      "description": "string",
      "status": "string",
      "created_at": "timestamp",
      "updated_at": "timestamp"
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 100,
  "total_pages": 5
}
```

#### GET /flows/:flow_id
Get a specific flow by ID.

#### PUT /flows/:flow_id
Update a flow.

#### DELETE /flows/:flow_id
Delete a flow.

#### POST /flows/:flow_id/activate
Activate a flow.

#### POST /flows/:flow_id/archive
Archive a flow.

#### POST /flows/import-dsl
Import a flow from Dify DSL.

**Request Body:**
```json
{
  "name": "string",
  "dsl": "string"
}
```

#### POST /flows/:flow_id/execute
Execute a flow.

**Request Body:**
```json
{
  "session_id": "uuid (optional)",
  "input_data": "object (optional)"
}
```

#### GET /executions/:execution_id
Get execution status.

#### GET /executions
List executions with optional filtering.

**Query Parameters:**
- `flow_id`: uuid (optional) - Filter by specific flow
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `limit`: number (default: 20, min: 1, max: 100) - Items per page

**Response:**
```json
{
  "data": [
    {
      "id": "uuid",
      "flow_id": "uuid",
      "status": "string",
      "started_at": "timestamp",
      "completed_at": "timestamp"
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 50,
  "total_pages": 3
}
```

### Flow Versions

#### POST /flows/:flow_id/versions
Create a new version of a flow.

#### GET /flows/:flow_id/versions
Get all versions of a flow.

#### POST /flows/:flow_id/rollback
Rollback to a specific version.

**Request Body:**
```json
{
  "target_version": "number"
}
```

### LLM Configuration

#### POST /llm-configs
Create a new LLM configuration.

**Request Body:**
```json
{
  "name": "string",
  "provider": "string",
  "model_name": "string",
  "parameters": "object (optional)",
  "credentials": "object (optional)",
  "description": "string (optional)"
}
```

#### GET /llm-configs
List all LLM configurations.

**Query Parameters:**
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `limit`: number (default: 20, min: 1, max: 100) - Items per page
- `provider`: string (optional) - Filter by provider (e.g., "openai", "anthropic")

**Response:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "string",
      "provider": "string",
      "model_name": "string",
      "is_default": false
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 15,
  "total_pages": 1
}
```

#### GET /llm-configs/:config_id
Get a specific LLM configuration.

#### PUT /llm-configs/:config_id
Update an LLM configuration.

#### DELETE /llm-configs/:config_id
Delete an LLM configuration.

#### POST /llm-configs/:config_id/set-default
Set an LLM configuration as default.

#### POST /llm-configs/:config_id/test
Test connection to an LLM provider.

#### GET /llm-providers/:provider/models
Get available models for a provider.

### Vector Configuration

#### POST /vector-configs
Create a new vector database configuration.

**Request Body:**
```json
{
  "name": "string",
  "provider": "string",
  "connection_params": "object"
}
```

#### GET /vector-configs
List all vector configurations.

**Query Parameters:**
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `limit`: number (default: 20, min: 1, max: 100) - Items per page

**Response:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "string",
      "provider": "string",
      "is_default": false
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 8,
  "total_pages": 1
}
```

#### GET /vector-configs/:config_id
Get a specific vector configuration.

#### PUT /vector-configs/:config_id
Update a vector configuration.

#### DELETE /vector-configs/:config_id
Delete a vector configuration.

#### POST /vector-configs/:config_id/set-default
Set a vector configuration as default.

#### POST /vector-configs/:config_id/test
Test connection to a vector database.

#### GET /vector-providers/:provider/params
Get required and optional parameters for a vector provider.

#### GET /vector-configs/health
Get health status of all vector configurations.

### Session Management

#### POST /sessions
Create a new chat session.

**Request Body:**
```json
{
  "title": "string (optional)"
}
```

#### GET /sessions
List all sessions for the authenticated user.

**Query Parameters:**
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `limit`: number (default: 20, min: 1, max: 100) - Items per page

**Response:**
```json
{
  "data": [
    {
      "id": "uuid",
      "title": "string",
      "created_at": "timestamp",
      "updated_at": "timestamp"
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 45,
  "total_pages": 3
}
```

#### GET /sessions/:session_id
Get a specific session.

#### PUT /sessions/:session_id
Update a session.

#### DELETE /sessions/:session_id
Delete a session.

#### POST /sessions/:session_id/messages
Add a message to a session.

**Request Body:**
```json
{
  "role": "string",
  "content": "string",
  "metadata": "object (optional)"
}
```

#### POST /sessions/:session_id/context
Set a context variable in a session.

**Request Body:**
```json
{
  "key": "string",
  "value": "any"
}
```

#### GET /sessions/:session_id/context/:key
Get a context variable from a session.

### Audit Logs

#### GET /audit/logs
Query audit logs.

**Query Parameters:**
- `user_id`: uuid (optional) - Filter by user
- `action`: string (optional) - Filter by action type
- `resource_type`: string (optional) - Filter by resource type
- `start_date`: ISO 8601 timestamp (optional) - Filter from date
- `end_date`: ISO 8601 timestamp (optional) - Filter to date
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `page_size`: number (default: 50, min: 1, max: 100) - Items per page

**Response:**
```json
{
  "logs": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "action": "string",
      "resource_type": "string",
      "resource_id": "uuid",
      "timestamp": "ISO 8601 timestamp"
    }
  ],
  "page": 1,
  "page_size": 50,
  "total": 500,
  "total_pages": 10
}
```

**Note:** This endpoint uses `page_size` instead of `limit` for historical reasons, but follows the same pagination pattern.

#### GET /audit/statistics
Get audit statistics.

**Query Parameters:**
- `start_date`: ISO 8601 timestamp (optional)
- `end_date`: ISO 8601 timestamp (optional)

### Execution History

#### GET /execution-history
Query execution history.

**Query Parameters:**
- `flow_id`: uuid (optional) - Filter by flow
- `user_id`: uuid (optional) - Filter by user
- `status`: string (optional) - Filter by status (e.g., "completed", "failed", "running")
- `start_date`: ISO 8601 timestamp (optional) - Filter from date
- `end_date`: ISO 8601 timestamp (optional) - Filter to date
- `page`: number (default: 1, min: 1) - Page number (1-based)
- `page_size`: number (default: 50, min: 1, max: 100) - Items per page

**Response:**
```json
{
  "executions": [
    {
      "id": "uuid",
      "flow_id": "uuid",
      "user_id": "uuid",
      "status": "string",
      "started_at": "timestamp",
      "completed_at": "timestamp",
      "duration_ms": 1234
    }
  ],
  "page": 1,
  "page_size": 50,
  "total": 200,
  "total_pages": 4
}
```

**Note:** This endpoint uses `page_size` instead of `limit` for historical reasons, but follows the same pagination pattern.

#### GET /execution-history/:execution_id
Get detailed execution history including steps and metrics.

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
```json
{
  "error": "Validation error message",
  "timestamp": "ISO 8601 timestamp"
}
```

### 401 Unauthorized
```json
{
  "error": "Authentication required",
  "timestamp": "ISO 8601 timestamp"
}
```

### 403 Forbidden
```json
{
  "error": "Access denied",
  "timestamp": "ISO 8601 timestamp"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found",
  "timestamp": "ISO 8601 timestamp"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error",
  "timestamp": "ISO 8601 timestamp"
}
```

## Rate Limiting

API rate limiting is not currently implemented but should be added in production environments.

## Pagination

All list endpoints in the API support standardized pagination to efficiently retrieve large datasets.

### Pagination Parameters

All paginated endpoints accept the following query parameters:

- **`page`** (number, optional): Page number starting from **1** (one-based indexing for user-friendly API)
  - Default: `1`
  - Minimum: `1`
  - Example: `?page=1` retrieves the first page

- **`limit`** (number, optional): Number of items to return per page
  - Default: `20`
  - Minimum: `1`
  - Maximum: `100`
  - Example: `?limit=50` retrieves 50 items per page

**Note:** Some legacy endpoints may use `page_size` instead of `limit`. Both parameters serve the same purpose.

### Pagination Response Format

All paginated responses follow this standardized format:

```json
{
  "data": [...],           // Array of items for the current page
  "page": 1,               // Current page number (1-based)
  "limit": 20,             // Number of items per page
  "total": 150,            // Total number of items across all pages
  "total_pages": 8         // Total number of pages (calculated as ceil(total / limit))
}
```

### Pagination Examples

#### Example 1: Retrieve First Page with Default Settings

Request:
```bash
curl -X GET "http://localhost:8080/api/flows" \
  -H "Authorization: Bearer <token>"
```

Response:
```json
{
  "data": [
    {"id": "uuid-1", "name": "Flow 1", ...},
    {"id": "uuid-2", "name": "Flow 2", ...},
    ...
  ],
  "page": 1,
  "limit": 20,
  "total": 150,
  "total_pages": 8
}
```

#### Example 2: Retrieve Second Page with Custom Limit

Request:
```bash
curl -X GET "http://localhost:8080/api/flows?page=2&limit=50" \
  -H "Authorization: Bearer <token>"
```

Response:
```json
{
  "data": [
    {"id": "uuid-51", "name": "Flow 51", ...},
    {"id": "uuid-52", "name": "Flow 52", ...},
    ...
  ],
  "page": 2,
  "limit": 50,
  "total": 150,
  "total_pages": 3
}
```

#### Example 3: Retrieve Last Page

Request:
```bash
curl -X GET "http://localhost:8080/api/flows?page=8&limit=20" \
  -H "Authorization: Bearer <token>"
```

Response:
```json
{
  "data": [
    {"id": "uuid-141", "name": "Flow 141", ...},
    {"id": "uuid-142", "name": "Flow 142", ...},
    ...
  ],
  "page": 8,
  "limit": 20,
  "total": 150,
  "total_pages": 8
}
```

#### Example 4: Empty Results

When no items match the query or the page is beyond available data:

Request:
```bash
curl -X GET "http://localhost:8080/api/flows?page=100" \
  -H "Authorization: Bearer <token>"
```

Response:
```json
{
  "data": [],
  "page": 100,
  "limit": 20,
  "total": 150,
  "total_pages": 8
}
```

### Pagination Constraints and Validation

The API enforces the following constraints on pagination parameters:

1. **Page Number**:
   - Must be a positive integer (≥ 1)
   - Pages beyond the available data return empty results (not an error)

2. **Limit**:
   - Must be between 1 and 100
   - Values outside this range will return a `400 Bad Request` error
   - Default value of 20 is used if not specified

3. **Validation Errors**:

```json
{
  "error": "Limit must be greater than 0",
  "timestamp": "2025-10-15T10:30:00Z"
}
```

```json
{
  "error": "Limit cannot exceed 100",
  "timestamp": "2025-10-15T10:30:00Z"
}
```

### Calculating Total Pages

The `total_pages` field is calculated using the formula:

```
total_pages = ceil(total / limit)
```

This allows clients to easily determine if there are more pages available:
- If `page < total_pages`, more pages are available
- If `page >= total_pages`, you're on or past the last page

### Best Practices for Pagination

1. **Start with page 1**: Always begin pagination with `page=1`
2. **Use reasonable limits**: Default of 20 is suitable for most use cases
3. **Check total_pages**: Use this to determine when to stop paginating
4. **Handle empty results**: Pages beyond available data return empty arrays, not errors
5. **Combine with filters**: Pagination works seamlessly with filtering parameters

### Paginated Endpoints

The following endpoints support pagination:

- `GET /flows` - List flows
- `GET /flows/:flow_id/versions` - List flow versions
- `GET /executions` - List flow executions
- `GET /llm-configs` - List LLM configurations
- `GET /vector-configs` - List vector configurations
- `GET /sessions` - List chat sessions
- `GET /audit/logs` - Query audit logs
- `GET /execution-history` - Query execution history
- `GET /mcp-tools` - List MCP tools (if applicable)

## Filtering

Many list endpoints support filtering via query parameters. Refer to individual endpoint documentation for available filters.

## Date Formats

All dates should be provided in ISO 8601 format:
```
2025-10-14T12:00:00Z
```

## Testing

For testing the API, you can use tools like:
- curl
- Postman
- HTTPie
- Automated test suites (see tests directory)

## Examples

### Create and Execute a Flow

1. Login:
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"tenant_id":"...","username":"user","password":"pass"}'
```

2. Create a flow:
```bash
curl -X POST http://localhost:8080/api/flows \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"My Flow","description":"Test flow"}'
```

3. Execute the flow:
```bash
curl -X POST http://localhost:8080/api/flows/<flow_id>/execute \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"input_data":{"key":"value"}}'
```

4. Check execution status:
```bash
curl -X GET http://localhost:8080/api/executions/<execution_id> \
  -H "Authorization: Bearer <token>"
```

### Paginating Through Results

1. Get the first page of flows (default 20 items):
```bash
curl -X GET "http://localhost:8080/api/flows?page=1&limit=20" \
  -H "Authorization: Bearer <token>"
```

Response:
```json
{
  "data": [...],
  "page": 1,
  "limit": 20,
  "total": 150,
  "total_pages": 8
}
```

2. Get the next page:
```bash
curl -X GET "http://localhost:8080/api/flows?page=2&limit=20" \
  -H "Authorization: Bearer <token>"
```

3. Get a larger page size (up to 100):
```bash
curl -X GET "http://localhost:8080/api/flows?page=1&limit=100" \
  -H "Authorization: Bearer <token>"
```

4. Iterate through all pages (pseudocode):
```javascript
let page = 1;
let hasMorePages = true;

while (hasMorePages) {
  const response = await fetch(`/api/flows?page=${page}&limit=50`);
  const data = await response.json();
  
  // Process data.data array
  processFlows(data.data);
  
  // Check if there are more pages
  hasMorePages = page < data.total_pages;
  page++;
}
```

### Filtering with Pagination

Combine filters with pagination for targeted queries:

1. Get audit logs for a specific user with pagination:
```bash
curl -X GET "http://localhost:8080/api/audit/logs?user_id=<uuid>&page=1&page_size=50" \
  -H "Authorization: Bearer <token>"
```

2. Get LLM configs filtered by provider:
```bash
curl -X GET "http://localhost:8080/api/llm-configs?provider=openai&page=1&limit=20" \
  -H "Authorization: Bearer <token>"
```

3. Get execution history within a date range:
```bash
curl -X GET "http://localhost:8080/api/execution-history?start_date=2025-10-01T00:00:00Z&end_date=2025-10-15T23:59:59Z&page=1&page_size=50" \
  -H "Authorization: Bearer <token>"
```
