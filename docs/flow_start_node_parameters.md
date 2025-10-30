# Flow Start Node Variables

## Overview

The Start node in a flow can define variables with default values that are accessible throughout the flow execution. These variables are stored with a special naming convention and can be referenced in subsequent nodes.

## How It Works

### 1. Defining Variables in Start Node

When executing a flow, you can define variables in the Start node's `data.variables` field:

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
  },
  "position": { "x": 0, "y": 0 }
}
```

### 2. Variable Storage

The `StartNodeExecutor` automatically stores these variables in the execution state with a special prefix format:

```
#<node_id>.<variable_name>#
```

For example, with a start node ID of `start_1`:
- `user_input` → stored as `#start_1.user_input#`
- `max_tokens` → stored as `#start_1.max_tokens#`
- `temperature` → stored as `#start_1.temperature#`

### 3. Accessing Variables in Other Nodes

You can reference these variables in subsequent nodes using the `{{...}}` template syntax:

#### Example: LLM Chat Node

```json
{
  "id": "llm_1",
  "node_type": "Llm",
  "data": {
    "model": {
      "llm_config_id": "config-uuid"
    },
    "prompt_template": [
      {
        "role": "user",
        "text": "{{#start_1.user_input#}}"
      }
    ],
    "output_variable": "llm_response"
  }
}
```

#### Example: Variable Node

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

#### Example: MCP Tool Node

```json
{
  "id": "tool_1",
  "node_type": "McpTool",
  "data": {
    "tool_id": "tool-uuid",
    "parameters": {
      "query": "{{#start_1.user_input#}}",
      "max_results": "{{#start_1.max_tokens#}}"
    },
    "output_variable": "tool_result"
  }
}
```

## Use Cases

### 1. User Input Processing

Pass user input through the start node and use it in LLM prompts:

```json
{
  "variables": [
    {"variable": "user_question", "default": "What is the weather today?"},
    {"variable": "context", "default": "User is in New York"}
  ]
}
```

### 2. Configuration Variables

Pass configuration values that control flow behavior:

```json
{
  "variables": [
    {"variable": "max_iterations", "default": 5},
    {"variable": "temperature", "default": 0.7},
    {"variable": "model_name", "default": "gpt-4"}
  ]
}
```

### 3. Dynamic Prompts

Build dynamic prompts using start node variables:

```json
{
  "prompt_template": [
    {
      "role": "system",
      "text": "You are a helpful assistant."
    },
    {
      "role": "user",
      "text": "Answer this question: {{#start_1.user_question#}}. Context: {{#start_1.context#}}"
    }
  ]
}
```

## Best Practices

1. **Use Descriptive Variable Names**: Choose clear, descriptive names for your variables
   - Good: `user_input`, `max_tokens`, `search_query`
   - Bad: `x`, `val`, `data`

2. **Document Expected Variables**: Document what variables your flow expects in the flow metadata

3. **Provide Default Values**: Always provide default values in the `default` field for each variable

4. **Type Consistency**: Be aware that all variables are stored as JSON values and may need type conversion

## Implementation Details

The variable storage and resolution is handled by:

- **StartNodeExecutor**: Extracts variables from `data.variables` array and stores them with the `#node_id.variable_name#` prefix
- **Template Resolution**: Various executors (LLM, MCP Tool, Variable) resolve `{{...}}` references
- **ExecutionState**: Maintains the variable storage throughout flow execution

## Example Flow

Here's a complete example of a flow using start node variables:

```json
{
  "workflow": {
    "nodes": [
      {
        "id": "start_1",
        "node_type": "Start",
        "data": {
          "variables": [
            {"variable": "user_query", "default": "Explain quantum computing"},
            {"variable": "max_length", "default": 500}
          ]
        }
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
              "role": "user",
              "text": "{{#start_1.user_query#}}. Keep response under {{#start_1.max_length#}} words."
            }
          ],
          "output_variable": "response"
        }
      },
      {
        "id": "end_1",
        "node_type": "End",
        "data": {}
      }
    ],
    "edges": [
      {
        "id": "e1",
        "source": "start_1",
        "target": "llm_1"
      },
      {
        "id": "e2",
        "source": "llm_1",
        "target": "end_1"
      }
    ]
  }
}
```
