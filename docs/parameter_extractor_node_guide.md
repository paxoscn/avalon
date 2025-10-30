# Parameter Extractor Node Guide

## Overview

The Parameter Extractor node uses LLM to extract structured parameters from text input and returns them as a JSON array of strings.

## Node Type

```
parameter_extractor
```

## Node Data Structure

```json
{
  "model": {
    "llm_config_id": "uuid-of-llm-config"
  },
  "instruction": "System prompt for parameter extraction",
  "query": [
    ["node_id_1", "variable_name_1"],
    ["node_id_2", "variable_name_2"]
  ],
  "parameters": [
    {
      "name": "output_parameter_name"
    }
  ]
}
```

## Fields

### model (required)
Configuration for the LLM model to use for extraction.

- `llm_config_id`: UUID of the LLM configuration from the database

### instruction (required)
System prompt that instructs the LLM on how to extract parameters. The system will automatically append: "You must respond with a valid JSON array of strings only, no other text."

Example:
```
"Extract all product names mentioned in the following text"
```

### query (required)
Array of variable paths to use as input for parameter extraction. Each path is an array with two elements:
1. Node ID
2. Variable name

The content from these variables will be concatenated with double newlines (`\n\n`) and sent to the LLM as the user prompt.

Example:
```json
[
  ["1759993208994", "standard_text"],
  ["1759993208995", "additional_context"]
]
```

### parameters (required)
Array with a single object specifying the output parameter name. The extracted parameters will be stored in the execution state with the format: `#{node_id}.{parameter_name}#`

Example:
```json
[
  {
    "name": "extracted_items"
  }
]
```

## Execution Logic

1. **Extract Model Configuration**: Retrieves the LLM configuration from the database using `llm_config_id`

2. **Build User Prompt**: Resolves all variable paths in the `query` array and concatenates their content

3. **Call LLM**: Sends the system instruction and user prompt to the LLM with a requirement to return a JSON array of strings

4. **Parse Response**: Attempts to parse the LLM response as a JSON array. If parsing fails, it tries to extract a JSON array from the response text

5. **Store Result**: Saves the extracted parameters array in the execution state with the key `#{node_id}.{parameter_name}#`

## Output

The node stores the extracted parameters in the execution state and returns:

```json
{
  "extracted_parameters": ["param1", "param2", "param3"],
  "parameter_name": "extracted_items",
  "model_used": "gpt-4"
}
```

## Example Usage

### Flow Configuration

```json
{
  "id": "extractor_1",
  "node_type": "parameter_extractor",
  "data": {
    "model": {
      "llm_config_id": "550e8400-e29b-41d4-a716-446655440000"
    },
    "instruction": "Extract all product names from the following customer inquiry. Return only the product names as a JSON array of strings.",
    "query": [
      ["start_1", "user_message"]
    ],
    "parameters": [
      {
        "name": "product_names"
      }
    ]
  },
  "position": { "x": 200, "y": 100 }
}
```

### Accessing Results

After execution, the extracted parameters can be accessed in subsequent nodes using:

```
{{#extractor_1.product_names#}}
```

This will contain a JSON array like: `["iPhone 15", "MacBook Pro", "AirPods"]`

## Error Handling

The node will fail with an error message if:

- The `model` field is missing or `llm_config_id` is invalid
- The LLM configuration is not found in the database
- The `query` field is missing or empty
- No valid content is found from the query paths
- The LLM call fails
- The tenant_id is missing from the execution context

## Integration with Other Nodes

### Input from Start Node

```json
{
  "id": "start_1",
  "node_type": "start",
  "data": {
    "variables": [
      {
        "variable": "user_message",
        "default": "I want to buy an iPhone 15 and AirPods"
      }
    ]
  }
}
```

### Output to Answer Node

```json
{
  "id": "answer_1",
  "node_type": "answer",
  "data": {
    "answer": "You mentioned the following products: {{#extractor_1.product_names#}}"
  }
}
```

## Best Practices

1. **Clear Instructions**: Provide specific and clear instructions in the `instruction` field to get accurate extractions

2. **Appropriate Model**: Choose an LLM model that is good at structured output generation (e.g., GPT-4, Claude)

3. **Input Validation**: Ensure the input text contains the information you want to extract

4. **Error Handling**: Always have a fallback path in your flow for when extraction fails

5. **Output Format**: The node always returns an array of strings. If you need other formats, use a subsequent processing node

## Limitations

- The output is always a JSON array of strings
- Only one output parameter is supported per node
- The LLM must be configured to return valid JSON
- Extraction quality depends on the LLM model and instruction quality
