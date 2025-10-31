# 迭代节点实现总结

## 概述

成功实现了迭代节点 (Iteration Node) 功能，支持对数组进行遍历并对每个元素执行子流程，最后将结果聚合到输出数组中。

## 实现的文件

### 1. 核心实现

#### `src/domain/value_objects/flow_definition.rs`
- 在 `NodeType` 枚举中添加了 `Iteration` 类型
```rust
pub enum NodeType {
    // ... 其他类型
    Iteration,
}
```

#### `src/domain/services/iteration_node_executor.rs` (新文件)
- 实现了 `IterationNodeExecutor` 结构体
- 实现了 `NodeExecutor` trait
- 负责准备迭代配置并存储到执行状态中

主要功能：
- 验证 `iterator_selector`、`output_selector` 和 `start_node_id` 参数
- 从状态中获取要遍历的数组
- 将迭代配置存储到状态中供执行引擎使用

#### `src/domain/services/execution_engine.rs`
- 在 `get_next_nodes` 方法中添加了对 `NodeType::Iteration` 的处理
- 添加了 `execute_iteration` 方法来处理迭代逻辑

`execute_iteration` 方法实现：
- 遍历迭代数组中的每个元素
- 为每个元素设置 `item` 和 `index` 变量
- 执行子流程（从 `start_node_id` 到 `output_selector` 指定的节点）
- 收集每次迭代的输出
- 将所有输出聚合到一个数组中

#### `src/domain/services/execution_engine_factory.rs`
- 在工厂方法中注册 `IterationNodeExecutor`
- 添加了必要的导入

#### `src/domain/services/mod.rs`
- 导出 `iteration_node_executor` 模块

### 2. 测试文件

#### `tests/iteration_node_test.rs` (新文件)
- 测试迭代节点的基本功能
- 测试数据结构的正确性
- 验证流程定义的有效性

### 3. 文档

#### `docs/iteration_node_guide.md` (新文件)
完整的使用指南，包含：
- 节点概述
- 数据结构说明
- 执行逻辑详解
- 使用示例
- 变量访问方式
- 错误处理
- 最佳实践
- 技术实现细节

#### `docs/iteration_node_quick_reference.md` (新文件)
快速参考文档，包含：
- 节点类型和数据结构
- 参数说明表格
- 特殊变量列表
- 最小示例
- 执行流程图
- 常见错误表格

## 数据结构

### 节点配置
```json
{
  "id": "iteration_1",
  "node_type": "iteration",
  "data": {
    "iterator_selector": ["node_id", "variable_name"],
    "output_selector": ["node_id", "variable_name"],
    "start_node_id": "sub_flow_start_node_id"
  }
}
```

### 参数说明

| 参数 | 类型 | 说明 |
|------|------|------|
| `iterator_selector` | Array[String, String] | 要遍历的数组变量路径 |
| `output_selector` | Array[String, String] | 输出结果的变量路径 |
| `start_node_id` | String | 子流程的起始节点ID |

## 执行流程

1. **准备阶段** (IterationNodeExecutor)
   - 验证配置参数
   - 获取迭代数组
   - 存储迭代配置到状态

2. **迭代阶段** (ExecutionEngine.execute_iteration)
   - 遍历数组中的每个元素
   - 为每个元素：
     - 设置 `#start_node_id.item#` = 当前元素
     - 设置 `#start_node_id.index#` = 当前索引
     - 执行子流程
     - 收集输出

3. **聚合阶段**
   - 将所有输出合并到数组
   - 存储到 `#output_node_id.output_var_name#`

4. **继续执行**
   - 执行迭代节点之后的节点

## 特殊变量

在迭代子流程中可用：
- `{{#start_node_id.item#}}` - 当前迭代的元素值
- `{{#start_node_id.index#}}` - 当前迭代的索引（从0开始）

## 使用示例

### 典型流程
```
Start → ParameterExtractor → Iteration → Answer
                                  ↓
                            LLM_1 → LLM_2
                            (子流程)
```

### 示例场景
处理检查项列表：
1. Start节点接收用户输入
2. ParameterExtractor提取检查项数组
3. Iteration节点遍历每个检查项
4. 子流程中的LLM节点处理每个检查项
5. Answer节点展示聚合后的结果

## 测试结果

```bash
running 2 tests
test test_iteration_node_data_structure ... ok
test test_iteration_node_basic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

## 技术特点

1. **类型安全**: 使用Rust的类型系统确保配置正确
2. **错误处理**: 完善的错误验证和错误消息
3. **状态管理**: 通过ExecutionState管理迭代状态
4. **子流程隔离**: 每次迭代独立执行子流程
5. **结果聚合**: 自动收集和聚合所有迭代结果

## 限制和注意事项

1. **数组类型**: `iterator_selector` 必须指向一个数组类型的变量
2. **子流程终点**: 子流程必须执行到 `output_selector` 指定的节点
3. **性能考虑**: 大数组可能导致执行时间较长
4. **错误传播**: 子流程中的任何错误都会导致整个迭代失败
5. **最大迭代次数**: 子流程有最大迭代次数限制（100次）

## 未来改进

1. **并行执行**: 支持并行处理多个迭代项
2. **条件过滤**: 支持在迭代前过滤数组元素
3. **批处理**: 支持分批处理大数组
4. **进度跟踪**: 提供迭代进度的实时反馈
5. **错误恢复**: 支持部分失败时继续执行其他迭代

## 相关文档

- [完整使用指南](./docs/iteration_node_guide.md)
- [快速参考](./docs/iteration_node_quick_reference.md)
- [参数提取器节点](./docs/parameter_extractor_quick_reference.md)
- [执行引擎实现](./docs/execution_engine_implementation.md)

## 总结

迭代节点的实现为流程引擎提供了强大的批处理能力，使得可以对数组中的每个元素执行复杂的子流程，并自动聚合结果。这对于处理列表数据、批量操作等场景非常有用。

实现遵循了现有的代码架构和设计模式，与其他节点类型保持一致，易于维护和扩展。
