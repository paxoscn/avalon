# Answer 节点快速参考

## 基本信息

- **节点类型**: `answer`
- **用途**: 处理包含变量引用的字符串并直接输出回复
- **执行器**: `AnswerNodeExecutor`

## 数据格式

```json
{
  "id": "answer_1",
  "node_type": "answer",
  "data": {
    "answer": "模板字符串，可包含 {{#node_id.variable_name#}} 格式的变量引用"
  },
  "position": { "x": 0, "y": 0 }
}
```

## 变量引用语法

```
{{#node_id.variable_name#}}
```

### 示例

```
"你好 {{#start_1.user_name#}}，这是你的结果：{{#llm_1.text#}}"
```

## 输出

执行后创建变量：`#<node_id>.answer#`

## 支持的数据类型

| 类型 | 转换方式 |
|------|---------|
| String | 直接使用 |
| Number | 转为字符串 |
| Boolean | "true" 或 "false" |
| Array | JSON 字符串 |
| Object | JSON 字符串 |
| Null | "null" |

## 常见用例

### 1. 组合多个节点的输出

```json
{
  "answer": "问题：{{#start_1.question#}}\n\n回答：{{#llm_1.text#}}"
}
```

### 2. 格式化 LLM 响应

```json
{
  "answer": "亲爱的 {{#start_1.user_name#}}，\n\n{{#llm_1.text#}}\n\n祝好！"
}
```

### 3. 创建结构化输出

```json
{
  "answer": "检查项目：\n{{#llm_1.text#}}\n\n状态：{{#check_1.status#}}\n完成时间：{{#check_1.timestamp#}}"
}
```

## 在 End 节点中使用

```json
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
  }
}
```

## 注意事项

1. ✅ 变量不存在时，占位符保持不变
2. ✅ 支持多个变量引用
3. ✅ 自动类型转换
4. ⚠️ 必须使用完整格式：`{{#node_id.variable_name#}}`
5. ⚠️ 节点 ID 和变量名区分大小写

## 完整示例

参见：`docs/examples/answer_node_example.json`
