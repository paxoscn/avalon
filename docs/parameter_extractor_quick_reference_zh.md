# 参数提取器节点 - 快速参考

## 节点类型
```
parameter_extractor
```

## 最小配置

```json
{
  "id": "extractor_1",
  "node_type": "parameter_extractor",
  "data": {
    "model": {
      "llm_config_id": "你的LLM配置UUID"
    },
    "instruction": "提取指令",
    "query": [
      ["源节点ID", "变量名"]
    ],
    "parameters": [
      {
        "name": "输出名称"
      }
    ]
  },
  "position": { "x": 0, "y": 0 }
}
```

## 字段说明

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `model.llm_config_id` | 字符串 (UUID) | 是 | 数据库中的LLM配置ID |
| `instruction` | 字符串 | 是 | 提取参数的系统提示词 |
| `query` | [节点ID, 变量名] 数组 | 是 | 输入变量路径 |
| `parameters[0].name` | 字符串 | 是 | 输出参数名称 |

## 变量访问

### 输入格式
```json
"query": [
  ["节点ID", "变量名"]
]
```

### 输出格式
```
{{#提取器节点ID.参数名#}}
```

## 常见用例

### 1. 提取产品名称
```json
{
  "instruction": "从文本中提取所有提到的产品名称",
  "query": [["start_1", "user_message"]],
  "parameters": [{"name": "products"}]
}
```

### 2. 提取行动项
```json
{
  "instruction": "从会议记录中提取所有行动项和任务",
  "query": [["start_1", "meeting_notes"]],
  "parameters": [{"name": "action_items"}]
}
```

### 3. 提取关键主题
```json
{
  "instruction": "提取对话中讨论的主要主题",
  "query": [
    ["start_1", "conversation_part1"],
    ["start_1", "conversation_part2"]
  ],
  "parameters": [{"name": "topics"}]
}
```

### 4. 提取联系信息
```json
{
  "instruction": "从文本中提取所有电子邮件地址和电话号码",
  "query": [["start_1", "document_text"]],
  "parameters": [{"name": "contacts"}]
}
```

## 输出结构

节点返回：
```json
{
  "extracted_parameters": ["项目1", "项目2", "项目3"],
  "parameter_name": "你的参数名",
  "model_used": "gpt-4"
}
```

存储在状态中：
```
#{节点ID}.{参数名}# = ["项目1", "项目2", "项目3"]
```

## 错误信息

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| "Parameter extractor node missing 'model' field" | 缺少模型配置 | 添加 `model.llm_config_id` |
| "LLM config not found" | 配置ID无效 | 检查LLM配置是否存在 |
| "Parameter extractor node missing 'query' field" | 缺少查询数组 | 添加 `query` 数组 |
| "No valid query content found" | 输入为空 | 检查源变量是否存在 |
| "LLM call failed" | LLM服务错误 | 检查LLM服务状态 |
| "Missing or invalid tenant_id" | 缺少租户上下文 | 确保状态中有tenant_id |

## 最佳实践

1. ✅ **明确具体**：编写清晰、具体的指令
2. ✅ **测试指令**：先用样本数据测试
3. ✅ **错误处理**：在流程中添加错误处理
4. ✅ **选择合适模型**：使用擅长结构化输出的模型
5. ✅ **验证输出**：使用前检查提取的参数

## 集成示例

```json
{
  "nodes": [
    {
      "id": "start_1",
      "node_type": "start",
      "data": {
        "variables": [
          {"variable": "text", "default": "示例文本"}
        ]
      }
    },
    {
      "id": "extractor_1",
      "node_type": "parameter_extractor",
      "data": {
        "model": {"llm_config_id": "uuid"},
        "instruction": "提取关键词",
        "query": [["start_1", "text"]],
        "parameters": [{"name": "keywords"}]
      }
    },
    {
      "id": "answer_1",
      "node_type": "answer",
      "data": {
        "answer": "关键词：{{#extractor_1.keywords#}}"
      }
    }
  ],
  "edges": [
    {"id": "e1", "source": "start_1", "target": "extractor_1"},
    {"id": "e2", "source": "extractor_1", "target": "answer_1"}
  ]
}
```

## 限制

- 输出始终是字符串的JSON数组
- 每个节点只支持一个输出参数
- 需要有效的LLM配置
- 依赖于LLM模型质量
- 没有内置的提取数据验证

## 相关文档

- [完整参数提取器指南](parameter_extractor_node_guide.md)
- [示例流程](examples/)
- [LLM节点文档](llm_node_guide.md)
