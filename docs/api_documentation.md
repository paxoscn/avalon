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
- `page`: number (default: 0)
- `limit`: number (default: 20)

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
- `flow_id`: uuid (optional)
- `page`: number (default: 0)
- `limit`: number (default: 20)

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
- `page`: number (default: 0)
- `limit`: number (default: 20)
- `provider`: string (optional)

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
- `user_id`: uuid (optional)
- `action`: string (optional)
- `resource_type`: string (optional)
- `start_date`: ISO 8601 timestamp (optional)
- `end_date`: ISO 8601 timestamp (optional)
- `page`: number (default: 0)
- `page_size`: number (default: 50)

#### GET /audit/statistics
Get audit statistics.

**Query Parameters:**
- `start_date`: ISO 8601 timestamp (optional)
- `end_date`: ISO 8601 timestamp (optional)

### Execution History

#### GET /execution-history
Query execution history.

**Query Parameters:**
- `flow_id`: uuid (optional)
- `user_id`: uuid (optional)
- `status`: string (optional)
- `start_date`: ISO 8601 timestamp (optional)
- `end_date`: ISO 8601 timestamp (optional)
- `page`: number (default: 0)
- `page_size`: number (default: 50)

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

List endpoints support pagination with the following query parameters:
- `page`: Page number (0-indexed)
- `limit` or `page_size`: Number of items per page

Responses include:
- `total`: Total number of items
- `page`: Current page number
- `limit` or `page_size`: Items per page

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
