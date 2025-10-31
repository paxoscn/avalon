# 迭代节点快速参考

## 节点类型
```
iteration
```

## 数据结构
```json
{
  "iterator_selector": ["<node_id>", "<variable_name>"],
  "output_selector": ["<node_id>", "<variable_name>"],
  "start_node_id": "<sub_flow_start_node_id>"
}
```

## 参数说明

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `iterator_selector` | Array[String, String] | ✓ | 要遍历的数组变量路径 `[节点ID, 变量名]` |
| `output_selector` | Array[String, String] | ✓ | 输出结果的变量路径 `[节点ID, 变量名]` |
| `start_node_id` | String | ✓ | 子流程的起始节点ID |

## 特殊变量

在迭代子流程中可用：

| 变量 | 格式 | 说明 |
|------|------|------|
| 当前元素 | `{{#<start_node_id>.item#}}` | 当前迭代的数组元素值 |
| 当前索引 | `{{#<start_node_id>.index#}}` | 当前迭代的索引（从0开始） |

## 输出

迭代完成后，`output_selector` 指定的变量将包含一个数组，数组中的每个元素对应一次迭代的输出结果。

访问格式：`{{#<output_node_id>.<output_var_name>#}}`

## 示例

### 最小示例
```json
{
  "id": "iteration_1",
  "node_type": "iteration",
  "data": {
    "iterator_selector": ["extractor_1", "items"],
    "output_selector": ["llm_2", "results"],
    "start_node_id": "llm_1"
  }
}
```

### 完整流程示例
```
Start → ParameterExtractor → Iteration → Answer
                                  ↓
                            LLM_1 → LLM_2
                            (子流程)
```

## 执行流程

1. 从 `iterator_selector` 获取数组
2. 对数组中每个元素：
   - 设置 `#start_node_id.item#` = 当前元素
   - 设置 `#start_node_id.index#` = 当前索引
   - 执行子流程（从 `start_node_id` 开始）
   - 收集 `output_selector` 节点的输出
3. 将所有输出聚合到数组
4. 继续执行后续节点

## 常见错误

| 错误信息 | 原因 | 解决方法 |
|---------|------|---------|
| `iterator_selector must have exactly 2 elements` | 数组长度不是2 | 确保格式为 `["node_id", "var_name"]` |
| `Iterator variable 'xxx' not found or not an array` | 变量不存在或不是数组 | 检查变量路径和类型 |
| `Sub-flow execution failed` | 子流程执行出错 | 检查子流程节点配置 |

## 注意事项

- ✓ 迭代数组必须是字符串数组或对象数组
- ✓ 子流程必须到达 `output_selector` 指定的节点
- ✓ 避免过大的数组导致性能问题
- ✓ 子流程中的任何错误都会导致整个迭代失败

## 相关文档

- [完整使用指南](./iteration_node_guide.md)
- [参数提取器节点](./parameter_extractor_quick_reference.md)
- [LLM节点](./llm_node_text_variable.md)
