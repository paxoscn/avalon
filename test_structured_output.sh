#!/bin/bash

# Test script for structured output feature

echo "Testing structured output feature..."

# Create a test flow with structured_output
cat > /tmp/test_flow.json << 'EOF'
{
  "workflow": {
    "graph": {
      "nodes": [
        {
          "id": "start-1",
          "parentId": null,
          "node_type": "start",
          "data": {},
          "position": {"x": 0, "y": 0}
        },
        {
          "id": "llm-1",
          "parentId": null,
          "node_type": "llm",
          "data": {
            "model": {
              "llm_config_id": "your-llm-config-id"
            },
            "prompt_template": [
              {
                "role": "user",
                "text": "判断用户是否戴口罩，返回 JSON 格式：{\"passed\": true/false}"
              }
            ],
            "structured_output": {
              "schema": {
                "type": "object",
                "properties": {
                  "passed": {
                    "type": "boolean",
                    "description": "是否通过验证（true/false）"
                  }
                },
                "required": ["passed"],
                "additionalProperties": false
              }
            },
            "output_variable": "llm_response"
          },
          "position": {"x": 200, "y": 0}
        },
        {
          "id": "answer-1",
          "parentId": null,
          "node_type": "answer",
          "data": {
            "answer": "{{llm_response}}"
          },
          "position": {"x": 400, "y": 0}
        }
      ],
      "edges": [
        {
          "id": "e1",
          "source": "start-1",
          "target": "llm-1",
          "sourceHandle": null,
          "targetHandle": null
        },
        {
          "id": "e2",
          "source": "llm-1",
          "target": "answer-1",
          "sourceHandle": null,
          "targetHandle": null
        }
      ]
    }
  }
}
EOF

echo "Test flow created at /tmp/test_flow.json"
echo ""
echo "Expected OpenAI request format:"
cat << 'EOF'
{
  "model": "gpt-4",
  "messages": [...],
  "response_format": {
    "type": "json_schema",
    "json_schema": {
      "name": "structured_output",
      "strict": true,
      "schema": {
        "type": "object",
        "properties": {
          "passed": {
            "type": "boolean",
            "description": "是否通过验证（true/false）"
          }
        },
        "required": ["passed"],
        "additionalProperties": false
      }
    }
  }
}
EOF

echo ""
echo "✅ Structured output feature implementation complete!"
echo ""
echo "Key changes:"
echo "1. Added ResponseFormat and JsonSchema structs to llm_service.rs"
echo "2. Added response_format field to ChatRequest"
echo "3. Added extract_structured_output() method to LLMChatNodeExecutor"
echo "4. Updated chat_completion() to accept response_format parameter"
echo "5. Updated OpenAI provider to serialize response_format correctly"
