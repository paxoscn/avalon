# Agent发布状态功能实现

## 概述

为Agent实体添加了发布状态功能，只有已发布的agent才能被其他用户看到和雇佣。

## 后端实现

### 1. 数据模型变更

#### Agent实体新增字段
- `is_published: bool` - 是否已发布
- `published_at: Option<DateTime<Utc>>` - 发布时间

#### 数据库迁移
创建了迁移文件 `m20241127_000001_add_published_to_agents.rs`，添加：
- `is_published` 列（默认值为 false）
- `published_at` 列（可为空）
- `idx_agents_is_published` 索引（提升查询性能）

### 2. 领域层

#### Agent实体方法
```rust
// 发布agent
pub fn publish(&mut self) -> Result<(), String>

// 取消发布agent
pub fn unpublish(&mut self) -> Result<(), String>
```

#### 仓储接口
```rust
// 查询已发布的agents
async fn find_by_tenant_published(&self, tenant_id: &TenantId) -> Result<Vec<Agent>>
```

### 3. 应用层

#### 应用服务接口
```rust
// 发布agent
async fn publish_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>

// 取消发布agent
async fn unpublish_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>
```

#### DTO更新
所有Agent相关的DTO都添加了发布状态字段：
- `AgentDto`
- `AgentCardDto`
- `AgentDetailDto`

### 4. API接口

#### 新增端点

**发布Agent**
```
POST /agents/{agent_id}/publish
```
- 权限：仅创建者可以发布
- 响应：204 No Content

**取消发布Agent**
```
POST /agents/{agent_id}/unpublish
```
- 权限：仅创建者可以取消发布
- 响应：204 No Content

#### 修改的端点

**列表查询**
```
GET /agents
```
- 现在只返回已发布的agents（`is_published = true`）
- 不显示已被雇佣的agents（`employer_id IS NULL`）

## 前端集成指南

### 1. Agent卡片显示

在agent列表和详情页面，可以显示发布状态：

```typescript
interface AgentCardDto {
  id: string;
  name: string;
  avatar?: string;
  greeting?: string;
  system_prompt_preview: string;
  creator_name: string;
  is_employer: boolean;
  is_allocated: boolean;
  is_creator: boolean;
  is_fired: boolean;
  fired_at?: string;
  is_published: boolean;      // 新增
  published_at?: string;       // 新增
  created_at: string;
}
```

### 2. 发布按钮实现

#### 仅创建者可见
```typescript
// 在agent详情页或管理页面
{agent.is_creator && (
  <div className="publish-controls">
    {agent.is_published ? (
      <button onClick={() => unpublishAgent(agent.id)}>
        取消发布
      </button>
    ) : (
      <button onClick={() => publishAgent(agent.id)}>
        发布
      </button>
    )}
  </div>
)}
```

#### API调用示例
```typescript
// 发布agent
async function publishAgent(agentId: string) {
  try {
    await fetch(`/api/agents/${agentId}/publish`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
    });
    // 刷新agent数据
    await refreshAgent();
  } catch (error) {
    console.error('发布失败:', error);
  }
}

// 取消发布agent
async function unpublishAgent(agentId: string) {
  try {
    await fetch(`/api/agents/${agentId}/unpublish`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
    });
    // 刷新agent数据
    await refreshAgent();
  } catch (error) {
    console.error('取消发布失败:', error);
  }
}
```

### 3. 状态标识显示

#### 发布状态徽章
```typescript
function PublishStatusBadge({ agent }: { agent: AgentCardDto }) {
  if (!agent.is_published) {
    return <span className="badge badge-draft">未发布</span>;
  }
  
  return (
    <span className="badge badge-published">
      已发布
      {agent.published_at && (
        <span className="text-xs ml-1">
          ({formatDate(agent.published_at)})
        </span>
      )}
    </span>
  );
}
```

### 4. 列表筛选

在"我创建的agents"页面，可以显示所有agents（包括未发布的）：

```typescript
// 获取我创建的agents（包括未发布的）
GET /agents/created

// 获取市场上的agents（仅已发布的）
GET /agents
```

### 5. 权限控制

```typescript
function AgentActions({ agent }: { agent: AgentDetailDto }) {
  return (
    <div className="agent-actions">
      {/* 创建者可以编辑和发布 */}
      {agent.is_creator && (
        <>
          <button onClick={() => editAgent(agent.id)}>编辑</button>
          {agent.is_published ? (
            <button onClick={() => unpublishAgent(agent.id)}>
              取消发布
            </button>
          ) : (
            <button onClick={() => publishAgent(agent.id)}>
              发布
            </button>
          )}
        </>
      )}
      
      {/* 其他用户只能雇佣已发布的agent */}
      {!agent.is_creator && agent.is_published && (
        <button onClick={() => employAgent(agent.id)}>
          雇佣
        </button>
      )}
    </div>
  );
}
```

### 6. UI/UX建议

1. **发布前提示**
   - 发布前确认agent配置完整
   - 提示发布后其他用户可见

2. **状态可视化**
   - 未发布：灰色或草稿标识
   - 已发布：绿色或公开标识
   - 显示发布时间

3. **操作反馈**
   - 发布/取消发布成功后显示提示
   - 操作失败时显示错误信息

4. **列表区分**
   - "我的agents"：显示所有（包括未发布）
   - "市场"：仅显示已发布的agents

## 业务逻辑

### 发布规则
1. 只有创建者可以发布/取消发布agent
2. 新创建的agent默认为未发布状态
3. 已发布的agent才会出现在公共列表中
4. 未发布的agent只有创建者可以看到和使用

### 雇佣规则
1. 只能雇佣已发布的agent
2. 雇佣会创建agent的副本，副本默认未发布
3. 被雇佣的agent副本不会出现在公共列表中

### 查询优化
- 添加了 `is_published` 索引
- 公共列表查询条件：`is_published = true AND employer_id IS NULL`

## 数据库迁移

运行迁移：
```bash
# 应用迁移
cargo run --bin migrator up

# 回滚迁移（如需要）
cargo run --bin migrator down
```

## 测试建议

### 后端测试
1. 测试发布/取消发布功能
2. 测试权限控制（非创建者不能发布）
3. 测试列表查询（只返回已发布的）
4. 测试雇佣功能（只能雇佣已发布的）

### 前端测试
1. 测试发布按钮显示和隐藏
2. 测试发布状态标识显示
3. 测试列表筛选功能
4. 测试权限控制UI

## 注意事项

1. **向后兼容**：现有的agents默认为未发布状态，需要手动发布
2. **权限检查**：所有发布相关操作都会验证用户是否为创建者
3. **性能优化**：使用索引提升查询性能
4. **数据一致性**：发布状态变更会更新 `updated_at` 时间戳
