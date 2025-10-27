# Agent 管理功能

## 概述

已成功在前端添加 Agent（智能代理）的完整增删改查功能。Agent 是具有自定义能力和知识的 AI 助手。

## 新增文件

### 1. 服务层
- `src/services/agent.service.ts` - Agent API 服务，包含所有 CRUD 操作和资源管理

### 2. 页面组件
- `src/pages/AgentListPage.tsx` - Agent 列表页面，支持分页、删除、复制和雇佣操作
- `src/pages/AgentDetailPage.tsx` - Agent 详情页面，支持创建和编辑 Agent

### 3. 类型定义
- 在 `src/types/index.ts` 中添加了 `Agent` 和 `AgentEmployment` 接口

### 4. 路由配置
- 更新 `src/router.tsx`，添加 `/agents` 和 `/agents/:id` 路由

### 5. 导航菜单
- 更新 `src/components/layout/Sidebar.tsx`，添加 Agents 导航项

## 功能特性

### Agent 列表页面 (`/agents`)
- 卡片式展示所有 Agent
- 显示 Agent 头像、名称、创建时间
- 显示关联的知识库、工具和流程数量
- 支持分页浏览
- 操作按钮：
  - **Edit** - 编辑 Agent
  - **Employ** - 雇佣 Agent
  - **Copy** - 复制 Agent
  - **Delete** - 删除 Agent

### Agent 详情页面 (`/agents/:id` 或 `/agents/new`)
- **基本信息**
  - Agent 名称（必填）
  - 头像 URL（可选）
  - 系统提示词（必填）
  - 额外设置（JSON 格式，可选）

- **预设问题**
  - 最多 3 个预设问题
  - 用户可快速选择的常用问题

- **知识库配置**
  - 选择关联的向量存储配置
  - 支持多选
  - 用于知识检索

- **MCP 工具配置**
  - 选择 Agent 可使用的工具
  - 支持多选
  - 显示工具描述

- **流程配置**
  - 选择 Agent 可执行的流程
  - 支持多选
  - 显示流程描述

## API 接口

Agent 服务支持以下操作：

```typescript
// 基础 CRUD
listAgents(params?)          // 获取 Agent 列表（支持分页）
getAgent(id)                 // 获取单个 Agent
createAgent(request)         // 创建新 Agent
updateAgent(id, request)     // 更新 Agent
deleteAgent(id)              // 删除 Agent

// 特殊操作
copyAgent(id)                // 复制 Agent
employAgent(id)              // 雇佣 Agent
terminateEmployment(id)      // 终止雇佣
listEmployedAgents()         // 获取已雇佣的 Agent

// 资源管理
addKnowledgeBase(agentId, configId)      // 添加知识库
removeKnowledgeBase(agentId, configId)   // 移除知识库
addMcpTool(agentId, toolId)              // 添加工具
removeMcpTool(agentId, toolId)           // 移除工具
addFlow(agentId, flowId)                 // 添加流程
removeFlow(agentId, flowId)              // 移除流程
```

## 使用方法

1. **访问 Agent 列表**
   - 点击侧边栏的 "Agents" 菜单项
   - 或直接访问 `/agents`

2. **创建新 Agent**
   - 在列表页点击 "Create Agent" 按钮
   - 填写必填信息（名称、系统提示词）
   - 可选配置头像、预设问题、关联资源
   - 点击 "Create Agent" 保存

3. **编辑 Agent**
   - 在列表页点击 Agent 卡片的 "Edit" 按钮
   - 修改配置信息
   - 点击 "Update Agent" 保存

4. **管理 Agent 资源**
   - 在编辑页面勾选/取消勾选知识库、工具或流程
   - 系统会自动保存更改（编辑模式）
   - 或在创建时一次性配置（新建模式）

5. **其他操作**
   - **Copy**: 快速复制现有 Agent 配置
   - **Employ**: 将 Agent 添加到个人工作空间
   - **Delete**: 删除不需要的 Agent

## 技术实现

- 使用 React + TypeScript
- 基于现有的设计系统和组件库
- 遵循项目的代码风格和架构模式
- 支持错误处理和加载状态
- 响应式设计，支持移动端

## 后端 API 要求

确保后端已实现以下 API 端点：

- `GET /agents` - 列表（支持分页参数）
- `GET /agents/:id` - 详情
- `POST /agents` - 创建
- `PUT /agents/:id` - 更新
- `DELETE /agents/:id` - 删除
- `POST /agents/:id/copy` - 复制
- `POST /agents/:id/employ` - 雇佣
- `DELETE /agents/:id/employ` - 终止雇佣
- `GET /agents/employed` - 已雇佣列表
- 资源管理端点（knowledge-bases, mcp-tools, flows）

## 注意事项

1. 系统提示词是必填项，用于定义 Agent 的行为
2. 预设问题最多 3 个
3. 头像支持外部 URL
4. 额外设置需要是有效的 JSON 格式
5. 资源关联在编辑模式下实时保存，新建模式下一次性保存
