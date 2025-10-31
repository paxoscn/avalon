# 迭代节点 (Iteration Node) 使用指南

## 概述

迭代节点 (Iteration Node) 是一种特殊的流程控制节点，用于遍历数组中的每个元素，并对每个元素执行一个子流程。迭代完成后，会将所有迭代的结果聚合到一个输出数组中。

## 节点类型

```
NodeType::Iteration
```

## 数据结构

迭代节点的 `data` 字段包含以下属性：

```json
{
  "iterator_selector": ["node_id", "variable_name"],
  "output_selector": ["node_id", "variable_name"],
  "start_node_id": "sub_flow_start_node_id"
}
```

### 字段说明

#### `iterator_selector` (必需)
- **类型**: 字符串数组，包含两个元素
- **格式**: `["node_id", "variable_name"]`
- **说明**: 指定要遍历的数组变量的位置
- **示例**: `["1759993208994", "checking_items"]` 表示节点ID为 `1759993208994` 的变量 `checking_items`

#### `output_selector` (必需)
- **类型**: 字符串数组，包含两个元素
- **格式**: `["node_id", "variable_name"]`
- **说明**: 指定每次迭代结果的输出位置，结果会被聚合到一个数组中
- **示例**: `["1761578154176", "structured_output"]` 表示节点ID为 `1761578154176` 的变量 `structured_output`

#### `start_node_id` (必需)
- **类型**: 字符串
- **说明**: 每次迭代开始执行的节点ID
- **示例**: `"llm_process_node"`

## 执行逻辑

1. **获取迭代数组**: 从 `iterator_selector` 指定的位置获取要遍历的数组
2. **遍历数组**: 对数组中的每个元素执行以下操作：
   - 将当前元素存储到 `start_node_id` 节点的 `item` 变量中（格式：`#start_node_id.item#`）
   - 将当前索引存储到 `start_node_id` 节点的 `index` 变量中（格式：`#start_node_id.index#`）
   - 从 `start_node_id` 开始执行子流程
   - 子流程执行到 `output_selector` 指定的节点时，收集该节点的输出
3. **聚合结果**: 将所有迭代的结果合并到 `output_selector` 指定的变量数组中
4. **继续执行**: 迭代完成后，继续执行迭代节点之后的节点

## 使用示例

### 示例 1: 处理检查项列表

假设我们有一个检查项列表，需要对每个检查项进行LLM处理：

```json
{
  "workflow": {
    "graph": {
      "nodes": [
        {
          "id": "start_1",
          "node_type": "start",
          "data": {
            "variables": [
              {
                "variable": "user_input",
                "default": "检查以下项目：项目A, 项目B, 项目C"
              }
            ]
          }
        },
        {
          "id": "extractor_1",
          "node_type": "parameter_extractor",
          "data": {
            "query": ["start_1", "user_input"],
            "instruction": "从文本中提取所有检查项，返回JSON字符串数组",
            "parameters": [{"name": "checking_items"}],
            "model": {"llm_config_id": "config-id"}
          }
        },
        {
          "id": "iteration_1",
          "node_type": "iteration",
          "data": {
            "iterator_selector": ["extractor_1", "checking_items"],
            "output_selector": ["llm_2", "structured_output"],
            "start_node_id": "llm_1"
          }
        },
        {
          "id": "llm_1",
          "node_type": "llm",
          "data": {
            "prompt_template": [
              {
                "role": "user",
                "text": "分析检查项：{{#llm_1.item#}}"
              }
            ],
            "model": {"llm_config_id": "config-id"}
          }
        },
        {
          "id": "llm_2",
          "node_type": "llm",
          "data": {
            "prompt_template": [
              {
                "role": "user",
                "text": "将分析结果结构化：{{#llm_1.text#}}"
              }
            ],
            "model": {"llm_config_id": "config-id"}
          }
        },
        {
          "id": "answer_1",
          "node_type": "answer",
          "data": {
            "answer": "所有检查项的分析结果：{{#llm_2.structured_output#}}"
          }
        }
      ],
      "edges": [
        {"source": "start_1", "target": "extractor_1"},
        {"source": "extractor_1", "target": "iteration_1"},
        {"source": "llm_1", "target": "llm_2"},
        {"source": "iteration_1", "target": "answer_1"}
      ]
    }
  }
}
```

### 执行流程

1. **Start节点**: 接收用户输入 "检查以下项目：项目A, 项目B, 项目C"
2. **ParameterExtractor节点**: 提取检查项数组 `["项目A", "项目B", "项目C"]`
3. **Iteration节点**: 
   - 第1次迭代：
     - `#llm_1.item#` = "项目A"
     - `#llm_1.index#` = 0
     - 执行 llm_1 → llm_2
     - 收集 `#llm_2.structured_output#` 的结果
   - 第2次迭代：
     - `#llm_1.item#` = "项目B"
     - `#llm_1.index#` = 1
     - 执行 llm_1 → llm_2
     - 收集 `#llm_2.structured_output#` 的结果
   - 第3次迭代：
     - `#llm_1.item#` = "项目C"
     - `#llm_1.index#` = 2
     - 执行 llm_1 → llm_2
     - 收集 `#llm_2.structured_output#` 的结果
4. **聚合结果**: `#llm_2.structured_output#` 现在包含一个数组，包含所有三次迭代的结果
5. **Answer节点**: 使用聚合后的结果生成最终答案

## 变量访问

### 在子流程中访问迭代变量

在迭代的子流程中，可以访问以下特殊变量：

- `{{#start_node_id.item#}}`: 当前迭代的元素值
- `{{#start_node_id.index#}}`: 当前迭代的索引（从0开始）

### 访问聚合结果

迭代完成后，可以通过 `output_selector` 指定的变量访问聚合结果：

```
{{#output_node_id.output_variable_name#}}
```

这个变量将包含一个数组，数组中的每个元素对应一次迭代的输出。

## 注意事项

1. **数组类型**: `iterator_selector` 指定的变量必须是一个数组类型
2. **子流程终点**: 子流程必须执行到 `output_selector` 指定的节点才能收集输出
3. **性能考虑**: 迭代次数过多可能导致执行时间较长，建议合理控制数组大小
4. **错误处理**: 如果子流程执行失败，整个迭代节点会失败并返回错误
5. **嵌套迭代**: 理论上支持嵌套迭代，但需要注意性能和复杂度

## 错误处理

迭代节点可能返回以下错误：

- `iterator_selector must have exactly 2 elements`: `iterator_selector` 格式不正确
- `output_selector must have exactly 2 elements`: `output_selector` 格式不正确
- `Iteration node missing 'start_node_id' field`: 缺少 `start_node_id` 字段
- `Iterator variable 'xxx' not found or not an array`: 迭代变量不存在或不是数组类型
- `Sub-flow execution failed`: 子流程执行失败
- `Sub-flow exceeded maximum iterations`: 子流程超过最大迭代次数限制

## 最佳实践

1. **明确的变量命名**: 使用清晰的变量名，便于理解和维护
2. **子流程简洁**: 保持子流程简单，避免过于复杂的逻辑
3. **结果验证**: 在迭代后添加验证节点，确保结果符合预期
4. **错误处理**: 在子流程中添加适当的错误处理逻辑
5. **性能优化**: 对于大数组，考虑分批处理或使用其他优化策略

## 相关节点

- **ParameterExtractor**: 常用于生成迭代所需的数组
- **LLM**: 常用于处理迭代中的每个元素
- **Answer**: 常用于展示迭代的最终结果
- **Condition**: 可在子流程中使用，实现条件分支

## 技术实现

迭代节点的实现位于：
- 节点类型定义: `src/domain/value_objects/flow_definition.rs`
- 节点执行器: `src/domain/services/iteration_node_executor.rs`
- 执行引擎集成: `src/domain/services/execution_engine.rs`
