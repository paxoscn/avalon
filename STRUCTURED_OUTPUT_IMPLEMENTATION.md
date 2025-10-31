# Structured Output Implementation

## 概述

实现了 LLMChatNodeExecutor 对结构化输出（structured output）的支持，允许节点配置中的 `structured_output` 字段被转换为大模型请求的 `response_format` 参数。

## 实现细节

### 1. 数据结构定义 (src/domain/services/llm_service.rs)

添加了两个新的数据结构来支持结构化输出：

```rust
/// Response format for structured outputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<JsonSchema>,
}

/// JSON schema for structured outputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonSchema {
    pub name: String,
    pub strict: bool,
    pub schema: serde_json::Value,
}
```

### 2. ChatRequest 扩展

在 `ChatRequest` 中添加了 `response_format` 字段：

```rust
pub struct ChatRequest {
    // ... 其他字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}
```

### 3. LLMChatNodeExecutor 增强

添加了 `extract_structured_output()` 方法来从节点数据中提取结构化输出配置：

```rust
fn extract_structured_output(
    &self,
    node: &FlowNode,
) -> Option<ResponseFormat> {
    let structured_output = node.data.get("structured_output")?;
    let schema = structured_output.get("schema")?;

    Some(ResponseFormat {
        format_type: "json_schema".to_string(),
        json_schema: Some(JsonSchema {
            name: "structured_output".to_string(),
            strict: true,
            schema: schema.clone(),
        }),
    })
}
```

### 4. LLM Service 接口更新

更新了 `chat_completion` 方法签名以接受 `response_format` 参数：

```rust
async fn chat_completion(
    &self,
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    tenant_id: Uuid,
    response_format: Option<ResponseFormat>,
) -> Result<ChatResponse, LLMError>;
```

### 5. OpenAI Provider 支持

在 `OpenAIChatRequest` 中添加了 `response_format` 字段，并在 `convert_request()` 方法中正确序列化：

```rust
#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    // ... 其他字段
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
}
```

## 使用示例

### 输入：节点配置

```yaml
structured_output:
  schema:
    type: object
    properties:
      passed:
        type: boolean
        description: "是否通过验证（true/false）"
    required:
      - passed
    additionalProperties: false
```

### 输出：OpenAI API 请求

```json
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
```

## 完整的流程节点示例

```json
{
  "id": "llm-1",
  "node_type": "llm",
  "data": {
    "model": {
      "llm_config_id": "your-llm-config-id"
    },
    "prompt_template": [
      {
        "role": "user",
        "text": "判断用户是否戴口罩，返回 JSON 格式"
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
  }
}
```

## 修改的文件

1. **src/domain/services/llm_service.rs**
   - 添加 `ResponseFormat` 和 `JsonSchema` 结构体
   - 在 `ChatRequest` 中添加 `response_format` 字段
   - 更新 `chat_completion` 方法签名

2. **src/domain/services/node_executors.rs**
   - 在 `LLMChatNodeExecutor` 中添加 `extract_structured_output()` 方法
   - 更新 `execute()` 方法以提取和传递 `response_format`
   - 更新 `ParameterExtractorNodeExecutor` 的 `chat_completion` 调用

3. **src/infrastructure/llm/providers/openai.rs**
   - 在 `OpenAIChatRequest` 中添加 `response_format` 字段
   - 更新 `convert_request()` 方法以序列化 `response_format`
   - 更新测试代码

## 兼容性

- 如果节点配置中没有 `structured_output` 字段，`response_format` 将为 `None`，不会影响现有功能
- 所有现有的 LLM 节点将继续正常工作
- 只有配置了 `structured_output` 的节点才会使用结构化输出功能

## 测试

运行以下命令进行编译检查：

```bash
cargo check
```

所有编译警告都是关于未使用的导入和变量，不影响功能。

## 注意事项

1. 结构化输出功能依赖于 OpenAI 的 `json_schema` 响应格式
2. `strict: true` 确保模型严格遵循提供的 schema
3. schema 必须是有效的 JSON Schema 格式
4. 其他 LLM provider（如果有）需要单独实现对 `response_format` 的支持
