# Flow测试页面使用指南

## 概述

Flow测试页面提供了一个友好的界面，让用户可以方便地测试Flow的执行，填入参数并查看执行结果。

## 功能特性

### 1. 访问测试页面

有两种方式访问Flow测试页面：

- **从Flow详情页面**：点击"Test Flow"按钮
- **直接访问**：访问URL `/flows/{flowId}/test`

### 2. Flow信息展示

测试页面会显示以下Flow信息：
- Flow名称
- Flow描述
- 当前版本号
- Flow状态（Active/Draft/Archived）

### 3. 参数输入模式

测试页面支持两种参数输入模式：

#### 表单模式
- 如果Flow定义中包含变量，会自动生成对应的输入框
- 每个变量都有独立的输入字段
- 适合简单的参数输入场景

#### JSON模式
- 提供一个JSON编辑器，可以直接输入完整的变量对象
- 支持复杂的嵌套数据结构
- 实时JSON格式验证
- 适合高级用户和复杂参数场景

可以通过"切换到JSON模式"/"切换到表单模式"按钮在两种模式间切换。

### 4. Session ID（可选）

- 可以指定一个Session ID来关联执行
- 如果留空，系统会自动创建新的会话
- 适用于需要保持会话上下文的场景

### 5. 执行Flow

点击"执行Flow"按钮后：
- 系统会验证输入参数
- 提交执行请求到后端
- 自动跳转到执行详情页面查看实时结果

### 6. Flow定义查看

页面底部会显示当前版本的完整Flow定义，方便用户参考。

## 使用示例

### 示例1：简单变量测试

假设Flow定义了以下变量：
```json
{
  "variables": {
    "username": "",
    "age": ""
  }
}
```

在表单模式下：
1. 在"username"字段输入：`john_doe`
2. 在"age"字段输入：`25`
3. 点击"执行Flow"

### 示例2：复杂JSON测试

切换到JSON模式，输入：
```json
{
  "user": {
    "name": "John Doe",
    "email": "john@example.com",
    "preferences": {
      "theme": "dark",
      "notifications": true
    }
  },
  "filters": ["active", "verified"]
}
```

### 示例3：带Session ID的测试

1. 在"Session ID"字段输入已存在的会话ID：`session-123`
2. 填写其他参数
3. 执行Flow，结果会关联到该会话

## 注意事项

1. **Flow状态**：只有状态为"Active"的Flow才能执行
2. **JSON格式**：在JSON模式下，确保输入的是有效的JSON格式
3. **必填参数**：确保所有必需的参数都已填写
4. **执行结果**：执行后会自动跳转到执行详情页面，可以实时查看执行状态和结果

## 路由配置

新增路由：
- 路径：`/flows/:id/test`
- 组件：`FlowTestPage`

## 相关页面

- **Flow详情页面**：`/flows/:id` - 查看Flow基本信息和历史执行记录
- **Flow执行详情页面**：`/flows/:flowId/executions/:executionId` - 查看具体执行的详细信息
- **Flow版本历史**：`/flows/:id/versions` - 查看Flow的版本历史
