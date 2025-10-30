# LLM Node Text Variable

## Overview

When an LLM (Large Language Model) node executes, it automatically stores the response content in a special `text` variable that can be easily accessed by subsequent nodes.

## Automatic Variable Storage

After an LLM node completes execution, the response content is stored in **two** locations:

1. **Custom Output Variable**: The variable name specified in `output_variable` (default: `llm_response`)
2. **Node Text Variable**: Automatically stored as `#<node_id>.text#`

## Variable Format

```
#<llm_node_id>.text#
```

For example, if your LLM node has ID `llm_1`, the response content will be stored as:
```
#llm_1.text#
```

## Benefits

### 1. Consistent Access Pattern

All LLM nodes follow the same pattern, making it easy to reference their output:
- `#llm_1.text#` - First LLM node's response
- `#llm_2.text#` - Second LLM node's response
- `#llm_3.text#` - Third LLM node's response

### 2. No Need to Remember Output Variable Names

You don't need to track custom `output_variable` names. Just use the node ID:

```json
{
  "id": "llm_1",
  "node_type": "Llm",
  "data": {
    "output_variable": "some_custom_name"
  }
}
```

Access via: `{{#llm_1.text#}}` (regardless of the custom name)

### 3. Easy Chaining

Chain multiple LLM calls easily:

```json
{
  "id": "llm_2",
  "node_type": "Llm",
  "data": {
    "prompt_template": [
      {
        "role": "user",
        "text": "Summarize this: {{#llm_1.text#}}"
      }
    ]
  }
}
```

## Usage Examples

### Example 1: Basic LLM Node

**Node Definition:**
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
        "text": "What is artificial intelligence?"
      }
    ],
    "output_variable": "ai_explanation"
  }
}
```

**Variables Stored:**
- `ai_explanation` = "Artificial intelligence is..."
- `#llm_1.text#` = "Artificial intelligence is..."

### Example 2: Chaining LLM Nodes

**Flow:**
```
Start → LLM 1 (Generate) → LLM 2 (Summarize) → LLM 3 (Translate) → End
```

**LLM 1 - Generate Content:**
```json
{
  "id": "llm_generate",
  "node_type": "Llm",
  "data": {
    "prompt_template": [
      {
        "role": "user",
        "text": "Write a detailed explanation of {{#start_1.topic#}}"
      }
    ]
  }
}
```

**LLM 2 - Summarize:**
```json
{
  "id": "llm_summarize",
  "node_type": "Llm",
  "data": {
    "prompt_template": [
      {
        "role": "user",
        "text": "Summarize this in 3 sentences: {{#llm_generate.text#}}"
      }
    ]
  }
}
```

**LLM 3 - Translate:**
```json
{
  "id": "llm_translate",
  "node_type": "Llm",
  "data": {
    "prompt_template": [
      {
        "role": "user",
        "text": "Translate to Spanish: {{#llm_summarize.text#}}"
      }
    ]
  }
}
```

### Example 3: Using in Variable Nodes

```json
{
  "id": "var_1",
  "node_type": "Variable",
  "data": {
    "assignments": [
      {
        "name": "final_result",
        "value": "{{#llm_1.text#}}"
      }
    ]
  }
}
```

### Example 4: Using in MCP Tool Nodes

```json
{
  "id": "tool_1",
  "node_type": "McpTool",
  "data": {
    "tool_id": "tool-uuid",
    "parameters": {
      "content": "{{#llm_1.text#}}",
      "action": "process"
    }
  }
}
```

### Example 5: Combining Multiple LLM Outputs

```json
{
  "id": "llm_final",
  "node_type": "Llm",
  "data": {
    "prompt_template": [
      {
        "role": "system",
        "text": "You are a helpful assistant that combines information."
      },
      {
        "role": "user",
        "text": "Combine these two perspectives:\n\nPerspective 1: {{#llm_1.text#}}\n\nPerspective 2: {{#llm_2.text#}}"
      }
    ]
  }
}
```

## Implementation Details

When an LLM node executes successfully:

1. The LLM service returns a response with `content` field
2. The executor stores the content in the custom `output_variable` (if specified)
3. The executor also stores the content in `#<node_id>.text#`
4. Both variables contain the same content
5. Subsequent nodes can reference either variable

**Code:**
```rust
// Store response in custom output variable
state.set_variable(output_var.to_string(), serde_json::json!(response.content));

// Also store in #node_id.text# for easy access
let text_var = format!("#{}.text#", node.id);
state.set_variable(text_var, serde_json::json!(response.content));
```

## Best Practices

1. **Use `#node_id.text#` for Chaining**: When referencing LLM output in subsequent nodes, prefer using `#node_id.text#` for consistency

2. **Use Custom Variables for Complex Logic**: If you need to store and manipulate the output separately, use the custom `output_variable`

3. **Descriptive Node IDs**: Use descriptive IDs for LLM nodes to make references clear:
   - Good: `llm_summarize`, `llm_translate`, `llm_analyze`
   - Bad: `llm_1`, `llm_2`, `llm_3`

4. **Document Dependencies**: When building complex flows, document which nodes depend on which LLM outputs

## Complete Example Flow

```json
{
  "workflow": {
    "nodes": [
      {
        "id": "start_1",
        "node_type": "Start",
        "data": {
          "variables": [
            {"variable": "user_query", "default": "Explain quantum computing"}
          ]
        }
      },
      {
        "id": "llm_explain",
        "node_type": "Llm",
        "data": {
          "model": {"llm_config_id": "config-uuid"},
          "prompt_template": [
            {
              "role": "user",
              "text": "{{#start_1.user_query#}}"
            }
          ]
        }
      },
      {
        "id": "llm_simplify",
        "node_type": "Llm",
        "data": {
          "model": {"llm_config_id": "config-uuid"},
          "prompt_template": [
            {
              "role": "user",
              "text": "Explain this in simpler terms for a 10-year-old: {{#llm_explain.text#}}"
            }
          ]
        }
      },
      {
        "id": "end_1",
        "node_type": "End",
        "data": {}
      }
    ],
    "edges": [
      {"id": "e1", "source": "start_1", "target": "llm_explain"},
      {"id": "e2", "source": "llm_explain", "target": "llm_simplify"},
      {"id": "e3", "source": "llm_simplify", "target": "end_1"}
    ]
  }
}
```

**Execution Result:**
- `#start_1.user_query#` = "Explain quantum computing"
- `#llm_explain.text#` = "Quantum computing is a type of computing that..."
- `#llm_simplify.text#` = "Imagine you have a magic box that can try..."

## Related Documentation

- [Flow Start Node Variables](flow_start_node_parameters.md)
- [Flow Execution with Variables](flow_execution_with_variables.md)
- [Variable Node Documentation](variable_node.md)
