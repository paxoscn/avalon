# 迭代节点 (Iteration Node) - 实现说明

## 功能概述

迭代节点是一个强大的流程控制节点，用于遍历数组中的每个元素，并对每个元素执行一个子流程。所有迭代的结果会被自动聚合到一个输出数组中。

## 核心特性

✅ **数组遍历**: 支持遍历字符串数组和对象数组  
✅ **子流程执行**: 为每个元素执行独立的子流程  
✅ **结果聚合**: 自动收集和聚合所有迭代结果  
✅ **变量访问**: 在子流程中访问当前元素和索引  
✅ **错误处理**: 完善的错误验证和错误消息  
✅ **类型安全**: 使用Rust类型系统确保配置正确  

## 快速开始

### 1. 节点配置

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

### 2. 在子流程中使用

```json
{
  "id": "llm_1",
  "node_type": "llm",
  "data": {
    "prompt_template": [
      {
        "role": "user",
        "text": "处理项目：{{#llm_1.item#}}，索引：{{#llm_1.index#}}"
      }
    ]
  }
}
```

### 3. 访问聚合结果

```json
{
  "id": "answer_1",
  "node_type": "answer",
  "data": {
    "answer": "所有结果：{{#llm_2.results#}}"
  }
}
```

## 文件结构

```
src/domain/
├── value_objects/
│   └── flow_definition.rs          # 添加 NodeType::Iteration
├── services/
│   ├── iteration_node_executor.rs  # 新增：迭代节点执行器
│   ├── execution_engine.rs         # 修改：添加迭代逻辑
│   ├── execution_engine_factory.rs # 修改：注册执行器
│   └── mod.rs                       # 修改：导出模块

tests/
└── iteration_node_test.rs           # 新增：测试文件

docs/
├── iteration_node_guide.md          # 新增：完整使用指南
├── iteration_node_quick_reference.md      # 新增：快速参考（英文）
└── iteration_node_quick_reference_zh.md   # 新增：快速参考（中文）

ITERATION_NODE_IMPLEMENTATION.md     # 实现总结
ITERATION_NODE_README.md             # 本文件
```

## 使用场景

### 场景1：批量处理检查项
```
用户输入 → 提取检查项 → 迭代处理每个检查项 → 汇总结果
```

### 场景2：多文档分析
```
文档列表 → 迭代分析每个文档 → 生成综合报告
```

### 场景3：数据转换
```
原始数据 → 迭代转换每条记录 → 输出转换后的数据集
```

## API参考

### 数据结构

```rust
pub struct IterationNodeData {
    pub iterator_selector: [String; 2],  // [节点ID, 变量名]
    pub output_selector: [String; 2],    // [节点ID, 变量名]
    pub start_node_id: String,           // 子流程起始节点ID
}
```

### 特殊变量

| 变量 | 类型 | 说明 |
|------|------|------|
| `#<start_node_id>.item#` | Any | 当前迭代的元素值 |
| `#<start_node_id>.index#` | Number | 当前迭代的索引（从0开始） |
| `#<output_node_id>.<var_name>#` | Array | 聚合后的结果数组 |

## 执行流程

```
1. 验证配置参数
   ↓
2. 获取迭代数组
   ↓
3. 对每个元素：
   ├─ 设置 item 变量
   ├─ 设置 index 变量
   ├─ 执行子流程
   └─ 收集输出
   ↓
4. 聚合所有结果
   ↓
5. 继续执行后续节点
```

## 测试

运行测试：
```bash
cargo test --test iteration_node_test
```

测试结果：
```
running 2 tests
test test_iteration_node_data_structure ... ok
test test_iteration_node_basic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

## 错误处理

| 错误 | 原因 | 解决方法 |
|------|------|---------|
| `iterator_selector must have exactly 2 elements` | 配置格式错误 | 使用 `["节点ID", "变量名"]` 格式 |
| `Iterator variable not found or not an array` | 变量不存在或类型错误 | 检查变量路径和类型 |
| `Sub-flow execution failed` | 子流程执行失败 | 检查子流程节点配置 |
| `Sub-flow exceeded maximum iterations` | 子流程超时 | 简化子流程或增加限制 |

## 性能考虑

- **数组大小**: 建议单次迭代不超过100个元素
- **子流程复杂度**: 保持子流程简单高效
- **超时设置**: 子流程最大迭代次数为100次
- **内存使用**: 大数组会占用更多内存

## 最佳实践

1. ✅ 使用清晰的变量命名
2. ✅ 保持子流程简洁
3. ✅ 添加适当的错误处理
4. ✅ 验证迭代结果
5. ✅ 控制数组大小
6. ✅ 监控执行时间

## 限制

- 迭代数组必须是数组类型
- 子流程必须到达输出节点
- 子流程错误会导致整个迭代失败
- 不支持并行执行（当前版本）

## 未来改进

- [ ] 支持并行迭代
- [ ] 支持条件过滤
- [ ] 支持批处理模式
- [ ] 添加进度跟踪
- [ ] 支持错误恢复

## 文档

- 📖 [完整使用指南](./docs/iteration_node_guide.md) - 详细的使用说明和示例
- 📋 [快速参考（英文）](./docs/iteration_node_quick_reference.md) - 简洁的API参考
- 📋 [快速参考（中文）](./docs/iteration_node_quick_reference_zh.md) - 中文版快速参考
- 📝 [实现总结](./ITERATION_NODE_IMPLEMENTATION.md) - 技术实现细节

## 示例代码

完整的示例代码请参见：
- `tests/iteration_node_test.rs` - 单元测试
- `docs/iteration_node_guide.md` - 使用示例

## 技术栈

- **语言**: Rust
- **异步运行时**: Tokio
- **序列化**: Serde JSON
- **测试框架**: Cargo Test

## 版本历史

### v0.1.0 (当前版本)
- ✅ 实现基本迭代功能
- ✅ 支持数组遍历
- ✅ 支持子流程执行
- ✅ 支持结果聚合
- ✅ 完整的文档和测试

## 贡献

欢迎提交问题和改进建议！

## 许可证

与主项目保持一致

---

**注意**: 这是一个新功能，如有任何问题或建议，请及时反馈。
