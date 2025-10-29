# MCP Tools Migration Guide

## Overview

This guide helps you migrate existing MCP tools to take advantage of the new features introduced in version 2.0:

- Parameter positioning (body/header/path)
- Path parameters with URL placeholders
- Response templates for formatted output
- MCP Server protocol interface

## Backward Compatibility

**Good News**: All existing MCP tools will continue to work without any changes!

- Parameters without a `position` field default to `body`
- Tools without `response_template` return raw JSON as before
- Existing API endpoints remain unchanged
- No database migration required

## Migration Strategies

### Strategy 1: No Changes Required

If your existing tools work well, you don't need to change anything. The new features are opt-in.

### Strategy 2: Gradual Migration

Migrate tools one at a time as you need the new features:

1. Start with tools that would benefit most from path parameters
2. Add response templates to improve readability
3. Move authentication to header parameters for better security

### Strategy 3: Full Migration

Update all tools to use the new features for consistency and improved functionality.

## Migration Steps

### Step 1: Add Position Fields to Parameters

**Before (v1.0):**
```json
{
  "parameters": [
    {
      "name": "city",
      "parameter_type": "String",
      "required": true
    }
  ]
}
```

**After (v2.0):**
```json
{
  "parameters": [
    {
      "name": "city",
      "parameter_type": "String",
      "required": true,
      "position": "body"
    }
  ]
}
```

**Note**: This step is optional. Parameters without `position` default to `body`.

### Step 2: Convert to Path Parameters

If your tool uses resource identifiers, convert them to path parameters.

**Before:**
```json
{
  "endpoint": "https://api.example.com/users",
  "method": "GET",
  "parameters": [
    {
      "name": "user_id",
      "parameter_type": "String",
      "required": true,
      "position": "body"
    }
  ]
}
```

**After:**
```json
{
  "endpoint": "https://api.example.com/users/{userId}",
  "method": "GET",
  "parameters": [
    {
      "name": "userId",
      "parameter_type": "String",
      "required": true,
      "position": "path"
    }
  ]
}
```

**Benefits**:
- More RESTful API design
- Cleaner URLs
- Better semantic meaning
- Improved caching

### Step 3: Move Authentication to Headers

**Before:**
```json
{
  "headers": {
    "Authorization": "Bearer hardcoded-token"
  },
  "parameters": []
}
```

**After:**
```json
{
  "headers": {
    "Content-Type": "application/json"
  },
  "parameters": [
    {
      "name": "Authorization",
      "parameter_type": "String",
      "description": "Bearer token for authentication",
      "required": true,
      "position": "header"
    }
  ]
}
```

**Benefits**:
- Dynamic authentication tokens
- No hardcoded credentials
- Better security
- Token can be passed at call time

### Step 4: Add Response Templates

**Before:**
Returns raw JSON:
```json
{
  "user": {
    "id": "123",
    "name": "John Doe",
    "email": "john@example.com",
    "orders": [
      {"id": "ord1", "total": 50.00, "status": "shipped"},
      {"id": "ord2", "total": 75.50, "status": "pending"}
    ]
  }
}
```

**After:**
Add a template for formatted output:
```json
{
  "response_template": "User Profile:\nName: {{ .user.name }}\nEmail: {{ .user.email }}\n\nOrders:\n{{- range .user.orders }}\n- Order {{ .id }}: {{ .status }} (${{ .total }})\n{{- end }}"
}
```

Returns formatted text:
```
User Profile:
Name: John Doe
Email: john@example.com

Orders:
- Order ord1: shipped ($50.00)
- Order ord2: pending ($75.50)
```

**Benefits**:
- Improved readability
- Better user experience
- Consistent formatting
- Easier to parse for humans

## Common Migration Patterns

### Pattern 1: Simple GET with ID

**Before:**
```json
{
  "name": "get_order",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/orders",
      "method": "GET",
      "parameters": [
        {
          "name": "order_id",
          "parameter_type": "String",
          "required": true
        }
      ]
    }
  }
}
```

**After:**
```json
{
  "name": "get_order",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/orders/{orderId}",
      "method": "GET",
      "parameters": [
        {
          "name": "orderId",
          "parameter_type": "String",
          "required": true,
          "position": "path"
        },
        {
          "name": "Authorization",
          "parameter_type": "String",
          "required": true,
          "position": "header"
        }
      ],
      "response_template": "Order #{{ .id }}\nStatus: {{ .status }}\nTotal: ${{ .total }}\nCustomer: {{ .customer.name }}"
    }
  }
}
```

### Pattern 2: POST with Authentication

**Before:**
```json
{
  "name": "create_user",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/users",
      "method": "POST",
      "headers": {
        "Authorization": "Bearer static-token"
      },
      "parameters": [
        {
          "name": "name",
          "parameter_type": "String",
          "required": true
        },
        {
          "name": "email",
          "parameter_type": "String",
          "required": true
        }
      ]
    }
  }
}
```

**After:**
```json
{
  "name": "create_user",
  "config": {
    "HTTP": {
      "endpoint": "https://api.example.com/users",
      "method": "POST",
      "parameters": [
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
      ],
      "response_template": "✅ User created successfully!\nID: {{ .id }}\nName: {{ .name }}\nEmail: {{ .email }}"
    }
  }
}
```

### Pattern 3: Nested Resource Access

**Before:**
```json
{
  "endpoint": "https://api.example.com/resources",
  "parameters": [
    {
      "name": "user_id",
      "parameter_type": "String",
      "required": true
    },
    {
      "name": "resource_id",
      "parameter_type": "String",
      "required": true
    }
  ]
}
```

**After:**
```json
{
  "endpoint": "https://api.example.com/users/{userId}/resources/{resourceId}",
  "parameters": [
    {
      "name": "userId",
      "parameter_type": "String",
      "required": true,
      "position": "path"
    },
    {
      "name": "resourceId",
      "parameter_type": "String",
      "required": true,
      "position": "path"
    },
    {
      "name": "Authorization",
      "parameter_type": "String",
      "required": true,
      "position": "header"
    }
  ]
}
```

## Testing Your Migration

### 1. Test in Development First

- Create a copy of your tool with a different name
- Apply the migration changes
- Test thoroughly before updating production tools

### 2. Use the Test Interface

1. Open the migrated tool configuration
2. Go to the **Test** tab
3. Provide test parameters
4. Verify the output matches expectations
5. Check that path parameters are correctly substituted
6. Verify headers are sent correctly
7. Confirm response template renders properly

### 3. Validate Configuration

The system automatically validates:
- Path parameter consistency
- Header naming conventions
- Template syntax

Fix any validation errors before saving.

### 4. Monitor Execution

After migration:
1. Check audit logs for tool calls
2. Monitor execution times
3. Review error rates
4. Verify response formats

## Rollback Plan

If you encounter issues after migration:

### Option 1: Revert to Previous Configuration

1. Go to tool versions
2. Select the pre-migration version
3. Click **Rollback**

### Option 2: Quick Fix

Remove the new features temporarily:
- Remove `position` fields (defaults to `body`)
- Remove `response_template` (returns raw JSON)
- Convert path parameters back to body parameters

## Common Issues and Solutions

### Issue 1: Path Parameter Not Substituted

**Symptom**: URL still contains `{parameterName}`

**Cause**: Parameter name doesn't match placeholder

**Solution**: Ensure exact match (case-sensitive):
```json
{
  "endpoint": "/users/{userId}",
  "parameters": [
    {"name": "userId", "position": "path"}  // Must match exactly
  ]
}
```

### Issue 2: Template Renders Empty

**Symptom**: Response is blank or shows template syntax

**Cause**: Variable names don't match JSON structure

**Solution**: Check actual JSON response structure:
```handlebars
{{/* Debug: output raw JSON */}}
{{ . }}
```

### Issue 3: Header Not Sent

**Symptom**: API returns authentication error

**Cause**: Header parameter not provided or wrong position

**Solution**: Verify parameter configuration:
```json
{
  "name": "Authorization",
  "position": "header",  // Must be "header"
  "required": true
}
```

### Issue 4: Validation Error on Save

**Symptom**: "Path parameter 'X' not found in endpoint"

**Cause**: Mismatch between endpoint placeholders and parameters

**Solution**: Ensure consistency:
- Every `{placeholder}` needs a path parameter
- Every path parameter needs a `{placeholder}`

## Migration Checklist

- [ ] Review all existing MCP tools
- [ ] Identify tools that would benefit from path parameters
- [ ] Identify tools that need response templates
- [ ] Create test copies of tools to migrate
- [ ] Update parameter positions
- [ ] Convert to path parameters where appropriate
- [ ] Move authentication to header parameters
- [ ] Add response templates
- [ ] Test each migrated tool thoroughly
- [ ] Validate configurations
- [ ] Update documentation for tool users
- [ ] Monitor execution after migration
- [ ] Update any flows that call the tools
- [ ] Train team on new features

## Benefits Summary

After migration, you'll gain:

✅ **Better API Design**: RESTful URLs with path parameters  
✅ **Improved Security**: Dynamic authentication tokens in headers  
✅ **Enhanced Readability**: Formatted text responses via templates  
✅ **Standard Protocol**: MCP Server interface for external integrations  
✅ **Better Performance**: Template caching for fast rendering  
✅ **Easier Debugging**: Clear parameter positioning and validation  

## Getting Help

If you need assistance with migration:

1. **Documentation**: Review the [MCP Tools Guide](mcp_tools_guide.md)
2. **Quick Reference**: Check [MCP Tools Quick Reference](mcp_tools_quick_reference.md)
3. **Test Interface**: Use the built-in testing tools
4. **Audit Logs**: Review execution logs for errors
5. **Support**: Contact support team for complex migrations

## Next Steps

After completing migration:

1. **Explore MCP Server**: Enable external integrations via the MCP protocol
2. **Optimize Templates**: Refine response templates for better UX
3. **Share Patterns**: Document successful patterns for your team
4. **Monitor Performance**: Track execution times and success rates
5. **Iterate**: Continuously improve tool configurations

---

**Version**: 2.0.0  
**Last Updated**: 2024-10-29
