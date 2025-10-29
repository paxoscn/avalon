# MCP Tools Quick Reference

## Parameter Positions

| Position | Usage | Example |
|----------|-------|---------|
| `body` | Request body (default) | Search queries, form data |
| `header` | HTTP headers | Authorization tokens, custom headers |
| `path` | URL path | User IDs, resource identifiers |

## Path Parameter Syntax

```
Endpoint: https://api.example.com/users/{userId}/orders/{orderId}

Parameters:
- userId (position: path)
- orderId (position: path)

Result: https://api.example.com/users/user123/orders/order456
```

## Response Template Syntax

### Variables
```handlebars
{{ .field_name }}
{{ .nested.field.name }}
```

### Loops
```handlebars
{{- range $index, $item := .items }}
- {{ .name }}: {{ .value }}
{{- end }}
```

### Conditionals
```handlebars
{{- if .condition }}
  True branch
{{- else }}
  False branch
{{- end }}
```

### Whitespace Control
```handlebars
{{- removes whitespace before
-}} removes whitespace after
```

## Complete Configuration Example

```json
{
  "name": "get_user_profile",
  "description": "Retrieve user profile with orders",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/users/{userId}",
      "method": "GET",
      "headers": {
        "Content-Type": "application/json"
      },
      "parameters": [
        {
          "name": "userId",
          "parameter_type": "String",
          "description": "User identifier",
          "required": true,
          "position": "path"
        },
        {
          "name": "Authorization",
          "parameter_type": "String",
          "description": "Bearer token",
          "required": true,
          "position": "header"
        },
        {
          "name": "include_orders",
          "parameter_type": "Boolean",
          "description": "Include order history",
          "required": false,
          "position": "body",
          "default_value": false
        }
      ],
      "timeout_seconds": 30,
      "retry_count": 3,
      "response_template": "User Profile:\nName: {{ .name }}\nEmail: {{ .email }}\n\n{{- if .orders }}\nRecent Orders:\n{{- range $index, $order := .orders }}\n- Order #{{ .id }}: {{ .status }} (${{ .total }})\n{{- end }}\n{{- end }}"
    }
  }
}
```

## MCP Server Endpoints

### List Tools
```bash
GET /api/v1/mcp/tools?page=1&limit=20
Authorization: Bearer <token>
X-Tenant-ID: <tenant_id>
```

### Call Tool
```bash
POST /api/v1/mcp/tools/call
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "tool_name",
  "arguments": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

## Type Mappings

| Internal Type | JSON Schema Type |
|---------------|------------------|
| String | string |
| Integer | integer |
| Number | number |
| Boolean | boolean |
| Array | array |
| Object | object |

## Common Patterns

### RESTful CRUD Operations

**GET Resource:**
```json
{
  "endpoint": "/api/resources/{id}",
  "method": "GET",
  "parameters": [
    {"name": "id", "position": "path", "required": true},
    {"name": "Authorization", "position": "header", "required": true}
  ]
}
```

**POST Resource:**
```json
{
  "endpoint": "/api/resources",
  "method": "POST",
  "parameters": [
    {"name": "Authorization", "position": "header", "required": true},
    {"name": "name", "position": "body", "required": true},
    {"name": "data", "position": "body", "required": true}
  ]
}
```

**PUT Resource:**
```json
{
  "endpoint": "/api/resources/{id}",
  "method": "PUT",
  "parameters": [
    {"name": "id", "position": "path", "required": true},
    {"name": "Authorization", "position": "header", "required": true},
    {"name": "name", "position": "body", "required": false},
    {"name": "data", "position": "body", "required": false}
  ]
}
```

**DELETE Resource:**
```json
{
  "endpoint": "/api/resources/{id}",
  "method": "DELETE",
  "parameters": [
    {"name": "id", "position": "path", "required": true},
    {"name": "Authorization", "position": "header", "required": true}
  ]
}
```

### Authentication Patterns

**Bearer Token:**
```json
{
  "name": "Authorization",
  "parameter_type": "String",
  "position": "header",
  "required": true
}
```

**API Key Header:**
```json
{
  "name": "X-API-Key",
  "parameter_type": "String",
  "position": "header",
  "required": true
}
```

**API Key in Path:**
```json
{
  "endpoint": "/api/{apiKey}/resources",
  "parameters": [
    {"name": "apiKey", "position": "path", "required": true}
  ]
}
```

### Response Template Patterns

**Simple Object:**
```handlebars
Name: {{ .name }}
Status: {{ .status }}
Created: {{ .created_at }}
```

**List with Numbering:**
```handlebars
Results ({{ len .items }} total):
{{- range $index, $item := .items }}
{{ add $index 1 }}. {{ .name }}
{{- end }}
```

**Conditional Display:**
```handlebars
{{- if .error }}
❌ Error: {{ .error }}
{{- else }}
✅ Success: {{ .message }}
{{- end }}
```

**Nested Data:**
```handlebars
User: {{ .user.name }}
Company: {{ .user.company.name }}
Location: {{ .user.company.address.city }}
```

**Formatted Lists:**
```handlebars
Order Summary:
- Order ID: {{ .id }}
- Customer: {{ .customer.name }}
- Total: ${{ .total }}

Items:
{{- range .items }}
  • {{ .quantity }}x {{ .product }} @ ${{ .price }}
{{- end }}

Shipping to:
{{ .shipping.address }}
{{ .shipping.city }}, {{ .shipping.state }} {{ .shipping.zip }}
```

## Validation Rules

### Path Parameters
- ✅ Must use `{parameterName}` syntax in endpoint
- ✅ All placeholders must have corresponding parameters
- ✅ All path parameters must appear in endpoint
- ✅ Values are automatically URL-encoded

### Header Parameters
- ✅ Must follow HTTP header naming conventions
- ✅ Allowed: letters, numbers, hyphens
- ❌ Not allowed: spaces, special characters (except hyphen)

### Response Templates
- ✅ Must be valid Handlebars syntax
- ✅ Compiled and cached for performance
- ✅ Automatically falls back to JSON on error
- ✅ Renders in <1ms after first compilation

## Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| Path parameter 'X' not found in endpoint | Parameter defined but not in URL | Add `{X}` to endpoint |
| Endpoint placeholder '{X}' has no parameter | Placeholder in URL but no parameter | Add parameter with position: path |
| Invalid header name | Header name has invalid characters | Use letters, numbers, hyphens only |
| Template syntax error | Invalid Handlebars syntax | Check matching braces and syntax |
| Template render error | Variable not found in JSON | Add conditional or fix variable name |

## Performance Tips

1. **Template Caching**: Templates are compiled once and cached
2. **First Call**: ~2-5ms (compilation + rendering)
3. **Subsequent Calls**: <1ms (cached)
4. **Cache Invalidation**: Automatic on tool update
5. **Timeout Settings**: 10-30 seconds recommended
6. **Retry Counts**: 2-3 retries for transient failures

## Testing Checklist

- [ ] Test with all required parameters
- [ ] Test with optional parameters omitted
- [ ] Test with invalid parameter values
- [ ] Test path parameter URL encoding (special characters)
- [ ] Test header parameter formatting
- [ ] Test response template with various API responses
- [ ] Test error scenarios (timeout, 404, 500)
- [ ] Verify template falls back to JSON on error
- [ ] Check execution time is acceptable
- [ ] Verify audit logs capture tool calls

## Resources

- [MCP Tools Guide](mcp_tools_guide.md) - Comprehensive documentation
- [API Documentation](api_documentation.md) - API reference
- [User Guide](user_guide.md) - General platform usage
- [Handlebars Syntax](https://handlebarsjs.com/) - Template reference

---

**Version**: 2.0.0  
**Last Updated**: 2024-10-29
