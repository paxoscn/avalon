# MCP Tools Guide

## Overview

MCP (Model Context Protocol) Tools enable you to integrate external HTTP APIs as callable tools within the Agent Platform. This guide covers the enhanced features including parameter positioning, path parameters, and response templates.

## Table of Contents

1. [Basic Concepts](#basic-concepts)
2. [Parameter Positions](#parameter-positions)
3. [Path Parameters](#path-parameters)
4. [Response Templates](#response-templates)
5. [MCP Server Interface](#mcp-server-interface)
6. [Configuration Examples](#configuration-examples)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## Basic Concepts

### What are MCP Tools?

MCP Tools are wrappers around HTTP APIs that allow you to:
- Call external REST APIs from your flows
- Transform API responses into readable formats
- Integrate third-party services seamlessly
- Provide standardized tool interfaces via MCP protocol

### Tool Configuration Structure

```json
{
  "name": "tool-name",
  "description": "What the tool does",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/resource",
      "method": "GET|POST|PUT|DELETE|PATCH",
      "headers": {},
      "parameters": [],
      "timeout_seconds": 30,
      "retry_count": 3,
      "response_template": "optional template"
    }
  }
}
```

## Parameter Positions

### Overview

Parameters can be placed in three different locations in the HTTP request:

1. **Body** (default): Sent in the request body as JSON
2. **Header**: Sent as HTTP headers
3. **Path**: Embedded in the URL path

### Position Field

Each parameter must specify its position:

```json
{
  "name": "parameter_name",
  "parameter_type": "String",
  "description": "Parameter description",
  "required": true,
  "position": "body|header|path"
}
```

### Body Parameters

Body parameters are sent in the request body as JSON. This is the default position.

**Example:**
```json
{
  "parameters": [
    {
      "name": "query",
      "parameter_type": "String",
      "description": "Search query",
      "required": true,
      "position": "body"
    },
    {
      "name": "limit",
      "parameter_type": "Integer",
      "description": "Maximum results",
      "required": false,
      "position": "body",
      "default_value": 10
    }
  ]
}
```

**Resulting Request:**
```http
POST /api/search HTTP/1.1
Content-Type: application/json

{
  "query": "example",
  "limit": 10
}
```

### Header Parameters

Header parameters are sent as HTTP headers. Useful for authentication tokens, content types, and custom headers.

**Example:**
```json
{
  "parameters": [
    {
      "name": "Authorization",
      "parameter_type": "String",
      "description": "API authentication token",
      "required": true,
      "position": "header"
    },
    {
      "name": "X-Request-ID",
      "parameter_type": "String",
      "description": "Request tracking ID",
      "required": false,
      "position": "header"
    }
  ]
}
```

**Resulting Request:**
```http
GET /api/resource HTTP/1.1
Authorization: Bearer token123
X-Request-ID: req-456
```

**Header Naming Rules:**
- Must follow HTTP header naming conventions
- Allowed characters: letters, numbers, hyphens
- Common headers: `Authorization`, `Content-Type`, `Accept`, `X-Custom-Header`

### Path Parameters

Path parameters are embedded directly in the URL path. Essential for RESTful APIs.

**Example:**
```json
{
  "endpoint": "https://api.example.com/users/{userId}/orders/{orderId}",
  "parameters": [
    {
      "name": "userId",
      "parameter_type": "String",
      "description": "User identifier",
      "required": true,
      "position": "path"
    },
    {
      "name": "orderId",
      "parameter_type": "String",
      "description": "Order identifier",
      "required": true,
      "position": "path"
    }
  ]
}
```

**Resulting Request:**
```http
GET /users/user123/orders/order456 HTTP/1.1
```

**Path Parameter Rules:**
- Use `{parameterName}` syntax in the endpoint URL
- All path parameters must be defined in the parameters array
- All `{placeholders}` in the endpoint must have corresponding path parameters
- Values are automatically URL-encoded for safety

### Combining Parameter Positions

You can mix different parameter positions in a single tool:

```json
{
  "endpoint": "https://api.example.com/users/{userId}/profile",
  "method": "PUT",
  "parameters": [
    {
      "name": "userId",
      "parameter_type": "String",
      "required": true,
      "position": "path"
    },
    {
      "name": "Authorization",
      "parameter_type": "String",
      "required": true,
      "position": "header"
    },
    {
      "name": "name",
      "parameter_type": "String",
      "required": true,
      "position": "body"
    },
    {
      "name": "email",
      "parameter_type": "String",
      "required": true,
      "position": "body"
    }
  ]
}
```

**Resulting Request:**
```http
PUT /users/user123/profile HTTP/1.1
Authorization: Bearer token123
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

## Path Parameters

### Syntax

Path parameters use curly brace syntax: `{parameterName}`

**Valid Examples:**
- `/users/{id}`
- `/api/v1/orders/{orderId}/items/{itemId}`
- `/resources/{resourceType}/{resourceId}`

**Invalid Examples:**
- `/users/:id` (wrong syntax)
- `/users/$id` (wrong syntax)
- `/users/{id` (missing closing brace)

### URL Encoding

Path parameter values are automatically URL-encoded to handle special characters safely.

**Example:**
```json
{
  "userId": "user@example.com"
}
```

**Becomes:**
```
/users/user%40example.com
```

### Validation

The system validates path parameters at configuration time:

1. **Consistency Check**: Every `{placeholder}` in the endpoint must have a corresponding parameter with `position: "path"`
2. **Completeness Check**: Every parameter with `position: "path"` must appear in the endpoint
3. **Runtime Check**: All path parameters must be provided when calling the tool

**Validation Errors:**
```json
{
  "error": "Path parameter 'userId' is defined but not found in endpoint URL"
}
```

```json
{
  "error": "Endpoint contains placeholder '{orderId}' but no corresponding path parameter is defined"
}
```

## Response Templates

### Overview

Response templates transform JSON API responses into human-readable text using Handlebars syntax.

### Why Use Templates?

- **Readability**: Convert complex JSON into clean, formatted text
- **Customization**: Extract only the data you need
- **Consistency**: Standardize output format across different APIs
- **User Experience**: Present data in a natural, conversational way

### Template Syntax

Response templates use Handlebars syntax with Go-style template features.

#### Variable Access

Access JSON fields using dot notation:

**JSON Response:**
```json
{
  "name": "John Doe",
  "age": 30,
  "email": "john@example.com"
}
```

**Template:**
```handlebars
Name: {{ .name }}
Age: {{ .age }}
Email: {{ .email }}
```

**Output:**
```
Name: John Doe
Age: 30
Email: john@example.com
```

#### Nested Fields

Access nested objects:

**JSON Response:**
```json
{
  "user": {
    "profile": {
      "name": "John Doe",
      "location": "New York"
    }
  }
}
```

**Template:**
```handlebars
{{ .user.profile.name }} is located in {{ .user.profile.location }}
```

**Output:**
```
John Doe is located in New York
```

#### Loops

Iterate over arrays:

**JSON Response:**
```json
{
  "items": [
    {"name": "Apple", "price": 1.50},
    {"name": "Banana", "price": 0.75},
    {"name": "Orange", "price": 1.25}
  ]
}
```

**Template:**
```handlebars
Shopping List:
{{- range $index, $item := .items }}
- {{ .name }}: ${{ .price }}
{{- end }}
```

**Output:**
```
Shopping List:
- Apple: $1.50
- Banana: $0.75
- Orange: $1.25
```

#### Conditionals

Use if/else statements:

**JSON Response:**
```json
{
  "status": "success",
  "message": "Operation completed",
  "error": null
}
```

**Template:**
```handlebars
{{- if .error }}
Error: {{ .error }}
{{- else }}
Status: {{ .status }}
Message: {{ .message }}
{{- end }}
```

**Output:**
```
Status: success
Message: Operation completed
```

#### Index Access

Access array elements by index:

**Template:**
```handlebars
First item: {{ index .items 0 }}
Second item: {{ index .items 1 }}
```

#### Whitespace Control

Use `-` to trim whitespace:

```handlebars
{{- if .condition -}}
  Content
{{- end -}}
```

### Template Examples

#### Example 1: Weather API

**JSON Response:**
```json
{
  "location": "New York",
  "temperature": 72,
  "conditions": "Partly Cloudy",
  "humidity": 65,
  "wind_speed": 10
}
```

**Template:**
```handlebars
Weather for {{ .location }}:
Temperature: {{ .temperature }}°F
Conditions: {{ .conditions }}
Humidity: {{ .humidity }}%
Wind Speed: {{ .wind_speed }} mph
```

#### Example 2: User Orders

**JSON Response:**
```json
{
  "order_id": "ORD-12345",
  "customer": "John Doe",
  "total": 125.50,
  "items": [
    {"product": "Widget A", "quantity": 2, "price": 25.00},
    {"product": "Widget B", "quantity": 3, "price": 25.50}
  ],
  "status": "shipped"
}
```

**Template:**
```handlebars
Order #{{ .order_id }} for {{ .customer }}

Items:
{{- range $index, $item := .items }}
- {{ .product }} (x{{ .quantity }}): ${{ .price }}
{{- end }}

Total: ${{ .total }}
Status: {{ .status }}
```

**Output:**
```
Order #ORD-12345 for John Doe

Items:
- Widget A (x2): $25.00
- Widget B (x3): $25.50

Total: $125.50
Status: shipped
```

#### Example 3: Search Results

**JSON Response:**
```json
{
  "query": "rust programming",
  "total_results": 1250,
  "results": [
    {
      "title": "The Rust Programming Language",
      "url": "https://doc.rust-lang.org/book/",
      "snippet": "The official Rust book"
    },
    {
      "title": "Rust by Example",
      "url": "https://doc.rust-lang.org/rust-by-example/",
      "snippet": "Learn Rust with examples"
    }
  ]
}
```

**Template:**
```handlebars
Search results for "{{ .query }}" ({{ .total_results }} total):

{{- range $index, $result := .results }}
{{ add $index 1 }}. {{ .title }}
   {{ .url }}
   {{ .snippet }}
{{- end }}
```

### Template Performance

- Templates are compiled and cached for performance
- First use: ~2-5ms (compilation + rendering)
- Subsequent uses: <1ms (cached rendering)
- Cache is automatically cleared when tool configuration is updated

### Template Error Handling

If template rendering fails:
1. The original JSON response is returned
2. An error message is included
3. The tool call is still considered successful

**Error Response:**
```json
{
  "result": { "original": "json", "data": "here" },
  "template_error": "Template syntax error at line 5"
}
```

## MCP Server Interface

### Overview

The MCP Server provides a standardized interface following the Model Context Protocol specification. External systems can discover and call tools without knowing the internal implementation details.

### Authentication

All MCP Server endpoints require:
- **Authorization header**: `Bearer <jwt_token>`
- **Tenant identification**: Via token claims or `X-Tenant-ID` header

### Listing Tools (tools/list)

**Endpoint:** `GET /api/v1/mcp/tools`

**Purpose:** Discover available tools and their schemas

**Request:**
```bash
curl -X GET "https://api.example.com/api/v1/mcp/tools" \
  -H "Authorization: Bearer <token>" \
  -H "X-Tenant-ID: <tenant_id>"
```

**Response:**
```json
{
  "tools": [
    {
      "name": "get_weather",
      "description": "Get current weather for a location",
      "inputSchema": {
        "type": "object",
        "properties": {
          "city": {
            "type": "string",
            "description": "City name"
          },
          "units": {
            "type": "string",
            "description": "Temperature units",
            "enum": ["celsius", "fahrenheit"]
          }
        },
        "required": ["city"]
      }
    }
  ]
}
```

**Pagination:**
```bash
curl -X GET "https://api.example.com/api/v1/mcp/tools?page=1&limit=50" \
  -H "Authorization: Bearer <token>"
```

### Calling Tools (tools/call)

**Endpoint:** `POST /api/v1/mcp/tools/call`

**Purpose:** Execute a tool with provided arguments

**Request:**
```bash
curl -X POST "https://api.example.com/api/v1/mcp/tools/call" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "get_weather",
    "arguments": {
      "city": "New York",
      "units": "fahrenheit"
    }
  }'
```

**Success Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Weather for New York:\nTemperature: 72°F\nConditions: Partly Cloudy"
    }
  ],
  "isError": false
}
```

**Error Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Error: Required parameter 'city' is missing"
    }
  ],
  "isError": true
}
```

### Schema Conversion

The MCP Server automatically converts internal parameter schemas to JSON Schema format:

**Internal Parameter:**
```json
{
  "name": "city",
  "parameter_type": "String",
  "description": "City name",
  "required": true,
  "position": "body"
}
```

**JSON Schema Output:**
```json
{
  "city": {
    "type": "string",
    "description": "City name"
  }
}
```

**Type Mappings:**
- `String` → `string`
- `Integer` → `integer`
- `Number` → `number`
- `Boolean` → `boolean`
- `Array` → `array`
- `Object` → `object`

## Configuration Examples

### Example 1: Simple GET Request

```json
{
  "name": "get_user",
  "description": "Retrieve user information by ID",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/users/{userId}",
      "method": "GET",
      "parameters": [
        {
          "name": "userId",
          "parameter_type": "String",
          "description": "User ID",
          "required": true,
          "position": "path"
        },
        {
          "name": "Authorization",
          "parameter_type": "String",
          "description": "API key",
          "required": true,
          "position": "header"
        }
      ],
      "timeout_seconds": 10,
      "response_template": "User: {{ .name }}\nEmail: {{ .email }}\nStatus: {{ .status }}"
    }
  }
}
```

### Example 2: POST with Body Parameters

```json
{
  "name": "create_order",
  "description": "Create a new order",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/orders",
      "method": "POST",
      "headers": {
        "Content-Type": "application/json"
      },
      "parameters": [
        {
          "name": "Authorization",
          "parameter_type": "String",
          "required": true,
          "position": "header"
        },
        {
          "name": "customer_id",
          "parameter_type": "String",
          "required": true,
          "position": "body"
        },
        {
          "name": "items",
          "parameter_type": "Array",
          "required": true,
          "position": "body"
        },
        {
          "name": "shipping_address",
          "parameter_type": "Object",
          "required": true,
          "position": "body"
        }
      ],
      "response_template": "Order created successfully!\nOrder ID: {{ .order_id }}\nTotal: ${{ .total }}\nEstimated delivery: {{ .estimated_delivery }}"
    }
  }
}
```

### Example 3: Complex RESTful API

```json
{
  "name": "update_product",
  "description": "Update product information",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/stores/{storeId}/products/{productId}",
      "method": "PUT",
      "parameters": [
        {
          "name": "storeId",
          "parameter_type": "String",
          "required": true,
          "position": "path"
        },
        {
          "name": "productId",
          "parameter_type": "String",
          "required": true,
          "position": "path"
        },
        {
          "name": "Authorization",
          "parameter_type": "String",
          "required": true,
          "position": "header"
        },
        {
          "name": "X-Request-ID",
          "parameter_type": "String",
          "required": false,
          "position": "header"
        },
        {
          "name": "name",
          "parameter_type": "String",
          "required": false,
          "position": "body"
        },
        {
          "name": "price",
          "parameter_type": "Number",
          "required": false,
          "position": "body"
        },
        {
          "name": "stock",
          "parameter_type": "Integer",
          "required": false,
          "position": "body"
        }
      ],
      "timeout_seconds": 15,
      "retry_count": 2,
      "response_template": "Product updated: {{ .name }}\nPrice: ${{ .price }}\nStock: {{ .stock }} units"
    }
  }
}
```

### Example 4: Search API with Template

```json
{
  "name": "search_products",
  "description": "Search for products",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/search",
      "method": "POST",
      "parameters": [
        {
          "name": "query",
          "parameter_type": "String",
          "required": true,
          "position": "body"
        },
        {
          "name": "category",
          "parameter_type": "String",
          "required": false,
          "position": "body"
        },
        {
          "name": "max_price",
          "parameter_type": "Number",
          "required": false,
          "position": "body"
        },
        {
          "name": "limit",
          "parameter_type": "Integer",
          "required": false,
          "position": "body",
          "default_value": 10
        }
      ],
      "response_template": "Found {{ .total_results }} products for \"{{ .query }}\":\n\n{{- range $index, $product := .results }}\n{{ add $index 1 }}. {{ .name }} - ${{ .price }}\n   {{ .description }}\n{{- end }}\n\n{{- if gt .total_results .limit }}\nShowing {{ .limit }} of {{ .total_results }} results\n{{- end }}"
    }
  }
}
```

## Best Practices

### Parameter Design

1. **Use appropriate positions**:
   - Authentication → `header`
   - Resource identifiers → `path`
   - Complex data → `body`

2. **Provide clear descriptions**: Help users understand what each parameter does

3. **Set sensible defaults**: Use `default_value` for optional parameters

4. **Validate at configuration time**: The system validates path parameter consistency

### Template Design

1. **Keep templates simple**: Complex logic should be in the API, not the template

2. **Handle missing data**: Use conditionals to check for null/undefined values

3. **Format numbers**: Use appropriate formatting for currency, percentages, etc.

4. **Test templates**: Use the test interface to verify template output

5. **Consider readability**: Add whitespace and formatting for human readers

### Security

1. **Never expose secrets**: Don't include API keys in tool descriptions or templates

2. **Use header parameters for auth**: Keep authentication tokens in headers

3. **Validate input**: The system validates parameter types and requirements

4. **URL encode path parameters**: Automatic encoding prevents injection attacks

5. **Limit retry counts**: Avoid overwhelming external APIs

### Performance

1. **Set appropriate timeouts**: Balance responsiveness with API requirements

2. **Use caching when possible**: Consider caching at the API level

3. **Optimize templates**: Simple templates render faster

4. **Monitor execution times**: Use audit logs to identify slow tools

### Error Handling

1. **Provide fallbacks**: Templates automatically fall back to JSON on error

2. **Test error scenarios**: Verify behavior when APIs return errors

3. **Set retry counts**: Use retries for transient failures

4. **Log failures**: All tool calls are logged for debugging

## Troubleshooting

### Path Parameter Issues

**Problem**: "Path parameter 'userId' not found in endpoint"

**Solution**: Ensure the endpoint contains `{userId}` placeholder:
```json
{
  "endpoint": "https://api.example.com/users/{userId}"
}
```

**Problem**: "Endpoint placeholder '{orderId}' has no corresponding parameter"

**Solution**: Add the path parameter to the parameters array:
```json
{
  "parameters": [
    {
      "name": "orderId",
      "position": "path",
      "required": true
    }
  ]
}
```

### Template Errors

**Problem**: "Template syntax error"

**Solution**: Check Handlebars syntax:
- Ensure all `{{` have matching `}}`
- Use `{{- ` for whitespace control
- Check variable names match JSON fields

**Problem**: "Template renders empty output"

**Solution**: Verify JSON structure matches template:
```handlebars
{{/* Debug: output raw JSON */}}
{{ . }}
```

### Header Issues

**Problem**: "Invalid header name"

**Solution**: Use valid HTTP header naming:
- Allowed: `Authorization`, `X-Custom-Header`, `Content-Type`
- Not allowed: `invalid header`, `header@name`

### MCP Server Issues

**Problem**: "Tool not found in MCP list"

**Solution**: 
- Verify tool is created for the correct tenant
- Check authentication token is valid
- Ensure tool is not archived/deleted

**Problem**: "MCP call fails with validation error"

**Solution**:
- Check argument names match parameter names
- Verify required parameters are provided
- Ensure argument types match parameter types

### Performance Issues

**Problem**: "Tool execution is slow"

**Solution**:
- Check external API response time
- Reduce timeout if API is consistently slow
- Consider caching results
- Simplify response template

**Problem**: "Template rendering is slow"

**Solution**:
- Simplify template logic
- Reduce loop iterations
- Check if template cache is working
- Verify template is not being recompiled each time

## Additional Resources

- [MCP Tools Quick Reference](mcp_tools_quick_reference.md) - Quick syntax and examples
- [API Documentation](api_documentation.md) - Complete API reference
- [User Guide](user_guide.md) - General platform usage
- [Handlebars Documentation](https://handlebarsjs.com/) - Template syntax reference
- [JSON Schema](https://json-schema.org/) - Schema specification
- [Model Context Protocol](https://modelcontextprotocol.io/) - MCP specification

---

**Last Updated**: 2024-10-29  
**Version**: 2.0.0
