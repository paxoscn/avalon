# End Node Outputs Configuration

## Overview

The End node marks the completion of a flow and can be configured to output specific variables from the flow execution. This allows you to control exactly what data is returned when the flow completes.

## Configuration Format

The End node accepts an `outputs` array in its `data` field:

```json
{
  "id": "end_1",
  "node_type": "End",
  "data": {
    "outputs": [
      {
        "value_selector": ["node_id", "variable_name"],
        "value_type": "string",
        "variable": "output_variable_name"
      }
    ]
  }
}
```

### Fields

- **value_selector**: Array with two elements `[node_id, variable_name]`
  - `node_id`: The ID of the node whose variable you want to output
  - `variable_name`: The name of the variable (e.g., "text" for LLM nodes)
  
- **value_type**: The type of the value (e.g., "string", "number", "object")
  - Currently informational, not enforced

- **variable**: The name of the output variable in the final result

## How It Works

### Variable Resolution

The End node constructs variable keys using the format `#node_id.variable_name#` and looks them up in the execution state:

1. Takes `value_selector`: `["llm_1", "text"]`
2. Constructs key: `#llm_1.text#`
3. Retrieves value from execution state
4. Stores in output with the name specified in `variable` field

### Backward Compatibility

If no `outputs` configuration is provided, the End node returns all variables in the execution state (legacy behavior).

## Usage Examples

### Example 1: Output LLM Response

**Flow Structure:**
```
Start → LLM Node → End
```

**End Node Configuration:**
```json
{
  "id": "end_1",
  "node_type": "End",
  "data": {
    "outputs": [
      {
        "value_selector": ["llm_1", "text"],
        "value_type": "string",
        "variable": "text"
      }
    ]
  }
}
```

**Execution Result:**
```json
{
  "message": "Flow completed",
  "outputs": {
    "text": "This is the LLM response content..."
  }
}
```

### Example 2: Output Multiple Variables

**Flow Structure:**
```
Start → LLM Generate → LLM Summarize → End
```

**End Node Configuration:**
```json
{
  "id": "end_1",
  "node_type": "End",
  "data": {
    "outputs": [
      {
        "value_selector": ["llm_generate", "text"],
        "value_type": "string",
        "variable": "full_text"
      },
      {
        "value_selector": ["llm_summarize", "text"],
        "value_type": "string",
        "variable": "summary"
      }
    ]
  }
}
```

**Execution Result:**
```json
{
  "message": "Flow completed",
  "outputs": {
    "full_text": "Detailed explanation of quantum computing...",
    "summary": "Quantum computing uses quantum mechanics principles..."
  }
}
```

### Example 3: Output Start Node Variables and LLM Response

**Flow Structure:**
```
Start → LLM → End
```

**End Node Configuration:**
```json
{
  "id": "end_1",
  "node_type": "End",
  "data": {
    "outputs": [
      {
        "value_selector": ["start_1", "user_input"],
        "value_type": "string",
        "variable": "question"
      },
      {
        "value_selector": ["llm_1", "text"],
        "value_type": "string",
        "variable": "answer"
      }
    ]
  }
}
```

**Execution Result:**
```json
{
  "message": "Flow completed",
  "outputs": {
    "question": "What is artificial intelligence?",
    "answer": "Artificial intelligence is the simulation of human intelligence..."
  }
}
```

### Example 4: Complex Multi-Node Output

**Flow Structure:**
```
Start → LLM Analysis → Vector Search → LLM Synthesis → End
```

**End Node Configuration:**
```json
{
  "id": "end_1",
  "node_type": "End",
  "data": {
    "outputs": [
      {
        "value_selector": ["start_1", "query"],
        "value_type": "string",
        "variable": "original_query"
      },
      {
        "value_selector": ["llm_analysis", "text"],
        "value_type": "string",
        "variable": "analysis"
      },
      {
        "value_selector": ["llm_synthesis", "text"],
        "value_type": "string",
        "variable": "final_answer"
      }
    ]
  }
}
```

**Execution Result:**
```json
{
  "message": "Flow completed",
  "outputs": {
    "original_query": "How does machine learning work?",
    "analysis": "Machine learning involves training algorithms...",
    "final_answer": "Based on the analysis and relevant documents..."
  }
}
```

## Complete Flow Example

```json
{
  "workflow": {
    "nodes": [
      {
        "id": "start_1",
        "node_type": "Start",
        "data": {
          "variables": [
            {"variable": "user_question", "default": "What is AI?"}
          ]
        },
        "position": {"x": 0, "y": 0}
      },
      {
        "id": "llm_1",
        "node_type": "Llm",
        "data": {
          "model": {"llm_config_id": "config-uuid"},
          "prompt_template": [
            {
              "role": "user",
              "text": "{{#start_1.user_question#}}"
            }
          ]
        },
        "position": {"x": 200, "y": 0}
      },
      {
        "id": "end_1",
        "node_type": "End",
        "data": {
          "outputs": [
            {
              "value_selector": ["start_1", "user_question"],
              "value_type": "string",
              "variable": "question"
            },
            {
              "value_selector": ["llm_1", "text"],
              "value_type": "string",
              "variable": "answer"
            }
          ]
        },
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

## Variable Naming Conventions

### For LLM Nodes
- Use `"text"` to access the LLM response content
- Example: `["llm_1", "text"]` → `#llm_1.text#`

### For Start Nodes
- Use the variable name defined in the Start node
- Example: `["start_1", "user_input"]` → `#start_1.user_input#`

### For Custom Variables
- Use the variable name as stored in the execution state
- Example: `["var_node_1", "result"]` → `#var_node_1.result#`

## Best Practices

1. **Be Selective**: Only output the variables you need, not all variables
   - Reduces response size
   - Makes the API clearer for consumers
   - Improves security by not exposing internal variables

2. **Use Descriptive Output Names**: Choose clear names for the `variable` field
   - Good: `"answer"`, `"summary"`, `"translation"`
   - Bad: `"output1"`, `"result"`, `"data"`

3. **Document Expected Outputs**: Document what outputs your flow returns
   - Helps API consumers understand the response structure
   - Makes integration easier

4. **Consider Output Structure**: Group related outputs logically
   ```json
   {
     "outputs": [
       {"value_selector": ["llm_1", "text"], "variable": "answer"},
       {"value_selector": ["start_1", "query"], "variable": "original_question"}
     ]
   }
   ```

5. **Handle Missing Variables**: The End node gracefully handles missing variables
   - If a variable doesn't exist, it's simply not included in the output
   - No error is thrown

## Implementation Details

### Variable Lookup Process

1. **Primary Lookup**: Try `#node_id.variable_name#` format
   ```rust
   let var_key = format!("#{}.{}#", node_id, var_name);
   state.get_variable(&var_key)
   ```

2. **Fallback Lookup**: If not found, try without prefix
   ```rust
   state.get_variable(var_name)
   ```

### Output Structure

**With Outputs Configuration:**
```json
{
  "message": "Flow completed",
  "outputs": {
    "variable1": "value1",
    "variable2": "value2"
  }
}
```

**Without Outputs Configuration (Legacy):**
```json
{
  "message": "Flow completed",
  "final_variables": {
    "all": "variables",
    "in": "state"
  }
}
```

## Common Patterns

### Pattern 1: Question-Answer Flow
```json
{
  "outputs": [
    {"value_selector": ["start_1", "question"], "variable": "question"},
    {"value_selector": ["llm_1", "text"], "variable": "answer"}
  ]
}
```

### Pattern 2: Multi-Step Processing
```json
{
  "outputs": [
    {"value_selector": ["llm_extract", "text"], "variable": "extracted_info"},
    {"value_selector": ["llm_analyze", "text"], "variable": "analysis"},
    {"value_selector": ["llm_recommend", "text"], "variable": "recommendations"}
  ]
}
```

### Pattern 3: Translation Pipeline
```json
{
  "outputs": [
    {"value_selector": ["start_1", "original_text"], "variable": "original"},
    {"value_selector": ["llm_translate", "text"], "variable": "translated"},
    {"value_selector": ["start_1", "target_language"], "variable": "language"}
  ]
}
```

## Related Documentation

- [Flow Start Node Variables](flow_start_node_parameters.md)
- [LLM Node Text Variable](llm_node_text_variable.md)
- [Flow Execution with Variables](flow_execution_with_variables.md)
