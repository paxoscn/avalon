# Flow Examples

This directory contains example flow definitions and usage patterns.

## Available Examples

### flow_with_input_override.json

Demonstrates how to use Start node variables with `input_data` overrides.

### llm_chaining_example.json

Demonstrates how to chain multiple LLM nodes using the automatic `#node_id.text#` variable.

### complete_flow_example.json

Complete end-to-end example showing:
- Start node with variables and input_data override
- LLM node with automatic text variable storage
- End node with outputs configuration

**Key Features:**
- Define default values in the Start node
- Override values at runtime using `input_data`
- Access variables in subsequent nodes using `{{#node_id.variable_name#}}` syntax

**Quick Start:**

1. Create a flow with Start node variables:
```json
{
  "variables": [
    {"variable": "user_question", "default": "What is AI?"},
    {"variable": "language", "default": "English"}
  ]
}
```

2. Execute with default values:
```bash
POST /api/v1/flows/{flow_id}/execute
{
  "input_data": {}
}
```

3. Execute with overrides:
```bash
POST /api/v1/flows/{flow_id}/execute
{
  "input_data": {
    "user_question": "Explain quantum computing",
    "language": "Spanish"
  }
}
```

## Related Documentation

- [Flow Start Node Variables](../flow_start_node_parameters.md) - Detailed guide on Start node variables
- [Flow Execution with Variables](../flow_execution_with_variables.md) - Complete execution guide
- [LLM Node Text Variable](../llm_node_text_variable.md) - How LLM nodes store their output in `#node_id.text#`
- [End Node Outputs](../end_node_outputs.md) - How to configure End node to output specific variables
