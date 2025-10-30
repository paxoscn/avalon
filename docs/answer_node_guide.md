# Answer 节点使用指南

## 概述

Answer 节点是一个用于直接回复的节点类型，它可以处理包含变量引用的字符串，并将结果输出到节点的 "answer" 变量中。

## 节点类型

```
node_type: "answer"
```

## 数据结构

Answer 节点的 `data` 字段包含以下属性：

- `answer` (string): 输出字符串，可以包含变量引用

## 变量引用格式

Answer 节点支持使用 `{{#node_id.variable_name#}}` 格式引用其他节点的变量。

### 示例

```json
{
  "id": "answer_1",
  "node_type": "answer",
  "data": {
    "answer": "你好 {{#start_1.user_name#}}，这是你的检查项目：{{#1761621778329.checking_items#}}"
  },
  "position": {
    "x": 100,
    "y": 200
  }
}
```

## 执行逻辑

1. 从节点的 `data.answer` 字段获取答案模板
2. 解析模板中的所有变量引用 `{{#node_id.variable_name#}}`
3. 从执行状态中查找对应的变量值并替换
4. 将解析后的结果存储到 `#node_id.answer#` 变量中

## 输出变量

执行完成后，Answer 节点会在执行状态中创建以下变量：

- `#<node_id>.answer#`: 解析后的完整答案字符串

## 完整示例

### Flow 定义

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
                "variable": "user_name",
                "default": "用户"
              }
            ]
          },
          "position": { "x": 0, "y": 0 }
        },
        {
          "id": "llm_1",
          "node_type": "llm",
          "data": {
            "model": {
              "llm_config_id": "config-uuid"
            },
            "prompt_template": [
              {
                "role": "user",
                "text": "请列出3个重要的检查项目"
              }
            ]
          },
          "position": { "x": 100, "y": 0 }
        },
        {
          "id": "answer_1",
          "node_type": "answer",
          "data": {
            "answer": "你好 {{#start_1.user_name#}}，以下是为你生成的检查项目：\n\n{{#llm_1.text#}}"
          },
          "position": { "x": 200, "y": 0 }
        },
        {
          "id": "end_1",
          "node_type": "end",
          "data": {
            "outputs": [
              {
                "value_selector": ["answer_1", "answer"],
                "value_type": "string",
                "variable": "final_answer"
              }
            ]
          },
          "position": { "x": 300, "y": 0 }
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
          "target": "answer_1"
        },
        {
          "id": "e3",
          "source": "answer_1",
          "target": "end_1"
        }
      ]
    }
  }
}
```

### 执行输入

```json
{
  "user_name": "Alice"
}
```

### 执行结果

Answer 节点会生成类似以下的输出：

```json
{
  "answer": "你好 Alice，以下是为你生成的检查项目：\n\n1. 检查系统配置\n2. 验证数据完整性\n3. 测试备份恢复"
}
```

最终在 End 节点的输出中，可以通过 `final_answer` 变量获取这个结果。

## 变量类型支持

Answer 节点支持解析以下类型的变量值：

- **String**: 直接插入字符串值
- **Number**: 转换为字符串后插入
- **Boolean**: 转换为 "true" 或 "false"
- **Array**: 转换为 JSON 字符串
- **Object**: 转换为 JSON 字符串
- **Null**: 插入 "null"

## 注意事项

1. 如果引用的变量不存在，占位符将保持不变（不会被替换）
2. 变量引用必须使用完整的格式：`{{#node_id.variable_name#}}`
3. Answer 节点不会修改原始的执行状态，只会添加新的输出变量
4. 建议在 End 节点中使用 Answer 节点的输出作为最终结果

## 测试

项目中包含了完整的单元测试，位于 `src/domain/services/node_executors.rs` 文件中：

- `test_answer_node_executor`: 测试基本的变量引用功能
- `test_answer_node_with_no_variables`: 测试纯文本输出
- `test_answer_node_with_missing_variable`: 测试缺失变量的处理
