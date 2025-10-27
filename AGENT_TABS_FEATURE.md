# Agent List Tabs Feature

## 概述
为 AgentListPage 添加了三个 tab 来区分不同类型的 Agent：
- Created (Created by me)
- Employed (Employed by me)
- Visible (Visible to me - all agents in tenant)

## 后端更改

### 1. Application Service
**文件**: `src/application/services/agent_application_service.rs`

添加了新的方法：
```rust
async fn list_created_agents(
    &self,
    user_id: UserId,
    params: PaginationParams,
) -> Result<PaginatedResponse<AgentCardDto>>;
```

该方法使用 `agent_repo.find_by_creator()` 来获取用户创建的所有 agents，并支持分页。

### 2. Handler
**文件**: `src/presentation/handlers/agent_handlers.rs`

添加了新的 handler：
```rust
pub async fn list_created_agents(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<AgentListQuery>,
) -> Result<impl IntoResponse>
```

### 3. Routes
**文件**: `src/presentation/routes/agent_routes.rs`

添加了新的路由：
```rust
.route("/agents/created", get(agent_handlers::list_created_agents))
```

## 前端更改

### 1. Agent Service
**文件**: `frontend/src/services/agent.service.ts`

- 更新了 `listEmployedAgents()` 方法，使其返回 `ListAgentsResponse` 并支持分页参数
- 添加了新的 `listCreatedAgents()` 方法

```typescript
async listEmployedAgents(params?: ListAgentsParams): Promise<ListAgentsResponse>
async listCreatedAgents(params?: ListAgentsParams): Promise<ListAgentsResponse>
```

### 2. Agent List Page
**文件**: `frontend/src/pages/AgentListPage.tsx`

主要更改：
- 添加了 `TabType` 类型定义：`'created' | 'employed' | 'visible'`
- 添加了 `activeTab` 状态来跟踪当前选中的 tab
- 添加了 tab 切换 UI（使用中文标签）
- 更新了 `loadAgents()` 函数，根据 `activeTab` 调用不同的 API
- 当 tab 切换时，自动重置页码为 1

## API 端点

### 新增端点
- `GET /agents/created` - 获取当前用户创建的 agents（支持分页）

### 现有端点
- `GET /agents` - 获取租户内所有可见的 agents（支持分页）
- `GET /agents/employed` - 获取当前用户雇佣的 agents（现在支持分页）

## 使用说明

1. 启动后端服务
2. 启动前端服务
3. 访问 Agents 页面
4. 点击不同的 tab 查看不同类型的 agents：
   - **Created**: 显示你创建的所有 agents
   - **Employed**: 显示你雇佣的所有 agents
   - **Visible**: 显示租户内所有可见的 agents

## 技术细节

- 所有三个列表都支持分页（每页 12 个 agents）
- Tab 切换时会自动重置到第一页
- 使用相同的 agent card 组件显示
- 保持了原有的所有功能（编辑、删除、复制、雇佣等）
