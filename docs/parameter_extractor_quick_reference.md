# Parameter Extractor Node - Quick Reference

## Node Type
```
parameter_extractor
```

## Minimal Configuration

```json
{
  "id": "extractor_1",
  "node_type": "parameter_extractor",
  "data": {
    "model": {
      "llm_config_id": "your-llm-config-uuid"
    },
    "instruction": "Your extraction instruction",
    "query": [
      ["source_node_id", "variable_name"]
    ],
    "parameters": [
      {
        "name": "output_name"
      }
    ]
  },
  "position": { "x": 0, "y": 0 }
}
```

## Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `model.llm_config_id` | string (UUID) | Yes | LLM configuration ID from database |
| `instruction` | string | Yes | System prompt for extraction |
| `query` | array of [node_id, var_name] | Yes | Input variable paths |
| `parameters[0].name` | string | Yes | Output parameter name |

## Variable Access

### Input Format
```json
"query": [
  ["node_id", "variable_name"]
]
```

### Output Format
```
{{#extractor_node_id.parameter_name#}}
```

## Common Use Cases

### 1. Extract Product Names
```json
{
  "instruction": "Extract all product names mentioned in the text",
  "query": [["start_1", "user_message"]],
  "parameters": [{"name": "products"}]
}
```

### 2. Extract Action Items
```json
{
  "instruction": "Extract all action items and tasks from the meeting notes",
  "query": [["start_1", "meeting_notes"]],
  "parameters": [{"name": "action_items"}]
}
```

### 3. Extract Key Topics
```json
{
  "instruction": "Extract the main topics discussed in the conversation",
  "query": [
    ["start_1", "conversation_part1"],
    ["start_1", "conversation_part2"]
  ],
  "parameters": [{"name": "topics"}]
}
```

### 4. Extract Contact Information
```json
{
  "instruction": "Extract all email addresses and phone numbers from the text",
  "query": [["start_1", "document_text"]],
  "parameters": [{"name": "contacts"}]
}
```

## Output Structure

The node returns:
```json
{
  "extracted_parameters": ["item1", "item2", "item3"],
  "parameter_name": "your_parameter_name",
  "model_used": "gpt-4"
}
```

Stored in state as:
```
#{node_id}.{parameter_name}# = ["item1", "item2", "item3"]
```

## Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Parameter extractor node missing 'model' field" | No model config | Add `model.llm_config_id` |
| "LLM config not found" | Invalid config ID | Check LLM config exists |
| "Parameter extractor node missing 'query' field" | No query array | Add `query` array |
| "No valid query content found" | Empty input | Check source variables exist |
| "LLM call failed" | LLM service error | Check LLM service status |
| "Missing or invalid tenant_id" | No tenant context | Ensure tenant_id in state |

## Best Practices

1. ✅ **Be Specific**: Write clear, specific instructions
2. ✅ **Test Instructions**: Test with sample data first
3. ✅ **Handle Errors**: Add error handling in your flow
4. ✅ **Choose Right Model**: Use models good at structured output
5. ✅ **Validate Output**: Check extracted parameters before use

## Integration Example

```json
{
  "nodes": [
    {
      "id": "start_1",
      "node_type": "start",
      "data": {
        "variables": [
          {"variable": "text", "default": "Sample text"}
        ]
      }
    },
    {
      "id": "extractor_1",
      "node_type": "parameter_extractor",
      "data": {
        "model": {"llm_config_id": "uuid"},
        "instruction": "Extract keywords",
        "query": [["start_1", "text"]],
        "parameters": [{"name": "keywords"}]
      }
    },
    {
      "id": "answer_1",
      "node_type": "answer",
      "data": {
        "answer": "Keywords: {{#extractor_1.keywords#}}"
      }
    }
  ],
  "edges": [
    {"id": "e1", "source": "start_1", "target": "extractor_1"},
    {"id": "e2", "source": "extractor_1", "target": "answer_1"}
  ]
}
```

## Limitations

- Output is always a JSON array of strings
- Only one output parameter per node
- Requires valid LLM configuration
- Depends on LLM model quality
- No built-in validation of extracted data

## See Also

- [Full Parameter Extractor Guide](parameter_extractor_node_guide.md)
- [Example Flows](examples/)
- [LLM Node Documentation](llm_node_guide.md)
