# Flow Execution with Start Node Variables

## Overview

This guide shows how to execute a flow with variables defined in the Start node and how to override those default values at runtime.

## Start Node Variable Structure

The Start node accepts variables in the following format:

```json
{
  "id": "start_1",
  "node_type": "Start",
  "data": {
    "variables": [
      {"variable": "user_input", "default": "Hello World"},
      {"variable": "max_tokens", "default": 100},
      {"variable": "temperature", "default": 0.7}
    ]
  }
}
```

## Executing a Flow

### 1. Using Default Values

When you execute a flow, the Start node will use the default values specified in the `variables` array:

```bash
POST /api/v1/flows/{flow_id}/execute
Content-Type: application/json

{
  "input_data": {}
}
```

In this case:
- `#start_1.user_input#` = "Hello World"
- `#start_1.max_tokens#` = 100
- `#start_1.temperature#` = 0.7

### 2. Overriding Default Values

You can override the default values by passing them in the `input_data`:

```bash
POST /api/v1/flows/{flow_id}/execute
Content-Type: application/json

{
  "input_data": {
    "user_input": "Custom user question",
    "max_tokens": 200
  }
}
```

In this case:
- `#start_1.user_input#` = "Custom user question" (overridden)
- `#start_1.max_tokens#` = 200 (overridden)
- `#start_1.temperature#` = 0.7 (default)

## How Variables Are Stored

When the Start node executes:

1. It reads the `variables` array from `node.data.variables`
2. For each variable, it checks if there's an override in the execution context
3. It stores the value (either default or overridden) with the format: `#<node_id>.<variable_name>#`

## Accessing Variables in Subsequent Nodes

### In LLM Chat Nodes

```json
{
  "id": "llm_1",
  "node_type": "Llm",
  "data": {
    "prompt_template": [
      {
        "role": "user",
        "text": "{{#start_1.user_input#}}"
      }
    ]
  }
}
```

### In Variable Nodes

```json
{
  "id": "var_1",
  "node_type": "Variable",
  "data": {
    "assignments": [
      {
        "name": "processed_input",
        "value": "{{#start_1.user_input#}}"
      }
    ]
  }
}
```

### In MCP Tool Nodes

```json
{
  "id": "tool_1",
  "node_type": "McpTool",
  "data": {
    "tool_id": "tool-uuid",
    "parameters": {
      "query": "{{#start_1.user_input#}}",
      "limit": "{{#start_1.max_tokens#}}"
    }
  }
}
```

## Complete Example

### Flow Definition

```json
{
  "workflow": {
    "nodes": [
      {
        "id": "start_1",
        "node_type": "Start",
        "data": {
          "variables": [
            {"variable": "question", "default": "What is AI?"},
            {"variable": "language", "default": "English"}
          ]
        },
        "position": {"x": 0, "y": 0}
      },
      {
        "id": "llm_1",
        "node_type": "Llm",
        "data": {
          "model": {
            "llm_config_id": "config-uuid"
          },
          "prompt_template": [
            {
              "role": "system",
              "text": "You are a helpful assistant. Answer in {{#start_1.language#}}."
            },
            {
              "role": "user",
              "text": "{{#start_1.question#}}"
            }
          ],
          "output_variable": "answer"
        },
        "position": {"x": 200, "y": 0}
      },
      {
        "id": "end_1",
        "node_type": "End",
        "data": {},
        "position": {"x": 400, "y": 0}
      }
    ],
    "edges": [
      {"id": "e1", "source": "start_1", "target": "llm_1"},
      {"id": "e2", "source": "llm_1", "target": "end_1"}
    ]
  }
}
```

### Execution Request

```bash
POST /api/v1/flows/{flow_id}/execute
Content-Type: application/json

{
  "input_data": {
    "question": "Explain quantum computing in simple terms",
    "language": "Spanish"
  }
}
```

### What Happens

1. **Start Node Execution**:
   - Reads variables from `data.variables`
   - Checks `input_data` for overrides
   - Stores: `#start_1.question#` = "Explain quantum computing in simple terms"
   - Stores: `#start_1.language#` = "Spanish"

2. **LLM Node Execution**:
   - Resolves template: "You are a helpful assistant. Answer in Spanish."
   - Resolves template: "Explain quantum computing in simple terms"
   - Calls LLM with resolved prompts
   - Stores response in `answer` variable

3. **End Node Execution**:
   - Returns final state with all variables

## Implementation Notes

### Current Implementation

The current implementation stores the **default values** from the Start node's `variables` array. To support runtime overrides, you would need to:

1. Pass `input_data` through the execution context
2. Modify `StartNodeExecutor` to check for overrides in the execution context
3. Use the override value if present, otherwise use the default

### Example Enhancement

```rust
// In StartNodeExecutor::execute
if let Some(variables) = node.data.get("variables") {
    if let Some(vars_array) = variables.as_array() {
        for var_item in vars_array {
            if let Some(var_obj) = var_item.as_object() {
                if let (Some(var_name), Some(default_value)) = (
                    var_obj.get("variable").and_then(|v| v.as_str()),
                    var_obj.get("default"),
                ) {
                    // Check for override in execution context
                    let value = state
                        .variables
                        .get(var_name)
                        .cloned()
                        .unwrap_or_else(|| default_value.clone());
                    
                    let prefixed_key = format!("#{}.{}#", node.id, var_name);
                    state.set_variable(prefixed_key, value);
                }
            }
        }
    }
}
```

## Best Practices

1. **Always Provide Defaults**: Every variable should have a sensible default value
2. **Document Variables**: Document what variables your flow expects and their purpose
3. **Validate Input**: Consider validating input_data before execution
4. **Use Descriptive Names**: Variable names should be clear and descriptive
5. **Type Safety**: Be aware that all values are JSON - handle type conversions appropriately
