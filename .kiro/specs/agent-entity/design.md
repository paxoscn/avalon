# Agent Entity Design Document

## Overview

本设计文档描述了数字人(Agent)实体的完整技术实现方案。Agent是一个可配置的AI助手实体，具有知识库、工具、工作流等资源配置能力，支持创建者权限管理和用户雇佣关系。系统采用DDD(领域驱动设计)架构，遵循现有代码库的设计模式。

## Architecture

### 层次结构

```
Presentation Layer (API)
    ↓
Application Layer (Services & DTOs)
    ↓
Domain Layer (Entities, Value Objects, Repositories)
    ↓
Infrastructure Layer (Database, Repositories Implementation)
```

### 核心组件

1. **Domain Layer**
   - `Agent` 实体：核心业务实体
   - `AgentId` 值对象：Agent唯一标识符
   - `AgentRepository` 接口：数据访问抽象
   - `AgentEmploymentRepository` 接口：雇佣关系数据访问
   - `AgentAllocationRepository` 接口：分配关系数据访问

2. **Infrastructure Layer**
   - 数据库实体映射
   - Repository实现
   - 数据库迁移脚本

3. **Application Layer**
   - `AgentApplicationService`：业务逻辑编排
   - DTOs：数据传输对象

4. **Presentation Layer**
   - REST API handlers
   - 路由配置

## Components and Interfaces

### 1. Domain Entities

#### Agent Entity

```rust
// src/domain/entities/agent.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{AgentId, TenantId, UserId, ConfigId, MCPToolId, FlowId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub tenant_id: TenantId,
    pub name: String,
    pub avatar: Option<String>,
    pub knowledge_base_ids: Vec<ConfigId>,  // Vector config IDs
    pub mcp_tool_ids: Vec<MCPToolId>,
    pub flow_ids: Vec<FlowId>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,  // Max 3
    pub source_agent_id: Option<AgentId>,
    pub creator_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    pub fn new(
        tenant_id: TenantId,
        name: String,
        system_prompt: String,
        creator_id: UserId,
    ) -> Result<Self, String>;
    
    pub fn update_name(&mut self, name: String) -> Result<(), String>;
    pub fn update_avatar(&mut self, avatar: Option<String>);
    pub fn update_system_prompt(&mut self, prompt: String) -> Result<(), String>;
    pub fn update_additional_settings(&mut self, settings: Option<String>);
    pub fn set_preset_questions(&mut self, questions: Vec<String>) -> Result<(), String>;
    pub fn add_knowledge_base(&mut self, config_id: ConfigId);
    pub fn remove_knowledge_base(&mut self, config_id: &ConfigId);
    pub fn add_mcp_tool(&mut self, tool_id: MCPToolId);
    pub fn remove_mcp_tool(&mut self, tool_id: &MCPToolId);
    pub fn add_flow(&mut self, flow_id: FlowId);
    pub fn remove_flow(&mut self, flow_id: &FlowId);
    pub fn is_creator(&self, user_id: &UserId) -> bool;
    pub fn can_modify(&self, user_id: &UserId) -> bool;
    pub fn copy_from(&self, new_creator_id: UserId) -> Self;
    pub fn validate(&self) -> Result<(), String>;
}
```

#### AgentEmployment Entity

```rust
// src/domain/entities/agent_employment.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{AgentId, UserId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEmployment {
    pub agent_id: AgentId,
    pub user_id: UserId,
    pub employed_at: DateTime<Utc>,
}

impl AgentEmployment {
    pub fn new(agent_id: AgentId, user_id: UserId) -> Self;
}
```

#### AgentAllocation Entity

```rust
// src/domain/entities/agent_allocation.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{AgentId, UserId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentAllocation {
    pub agent_id: AgentId,
    pub user_id: UserId,
    pub employed_at: DateTime<Utc>,
}

impl AgentAllocation {
    pub fn new(agent_id: AgentId, user_id: UserId) -> Self;
}
```

### 2. Value Objects

#### AgentId

```rust
// src/domain/value_objects/ids.rs (添加到现有文件)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
```

### 3. Repository Interfaces

```rust
// src/domain/repositories/agent_repository.rs

use async_trait::async_trait;
use crate::domain::entities::Agent;
use crate::domain::value_objects::{AgentId, TenantId, UserId};
use crate::error::Result;

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: &AgentId) -> Result<Option<Agent>>;
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Agent>>;
    async fn find_by_creator(&self, creator_id: &UserId) -> Result<Vec<Agent>>;
    async fn find_employed_by_user(&self, user_id: &UserId) -> Result<Vec<Agent>>;
    async fn save(&self, agent: &Agent) -> Result<()>;
    async fn delete(&self, id: &AgentId) -> Result<()>;
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64>;
    async fn find_by_tenant_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Agent>>;
}

#[async_trait]
pub trait AgentEmploymentRepository: Send + Sync {
    async fn employ(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    async fn terminate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    async fn is_employed(&self, agent_id: &AgentId, user_id: &UserId) -> Result<bool>;
    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<UserId>>;
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<AgentId>>;
}

#[async_trait]
pub trait AgentAllocationRepository: Send + Sync {
    async fn allocate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    async fn terminate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    async fn is_allocated(&self, agent_id: &AgentId, user_id: &UserId) -> Result<bool>;
    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<UserId>>;
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<AgentId>>;
}
```

### 4. Database Entities

```rust
// src/infrastructure/database/entities/agent.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub knowledge_base_ids: Json,  // Array of UUIDs
    pub mcp_tool_ids: Json,        // Array of UUIDs
    pub flow_ids: Json,            // Array of UUIDs
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Json,    // Array of strings
    pub source_agent_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatorId",
        to = "super::user::Column::Id"
    )]
    Creator,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::SourceAgentId",
        to = "Column::Id"
    )]
    SourceAgent,
}

impl ActiveModelBehavior for ActiveModel {}
```

```rust
// src/infrastructure/database/entities/agent_employment.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agent_employments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub agent_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub employed_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent::Entity",
        from = "Column::AgentId",
        to = "super::agent::Column::Id"
    )]
    Agent,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
```

```rust
// src/infrastructure/database/entities/agent_allocation.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agent_allocations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub agent_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub allocated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent::Entity",
        from = "Column::AgentId",
        to = "super::agent::Column::Id"
    )]
    Agent,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
```

### 5. Application Services

```rust
// src/application/services/agent_application_service.rs

pub struct AgentApplicationService {
    agent_repo: Arc<dyn AgentRepository>,
    employment_repo: Arc<dyn AgentEmploymentRepository>,
    allocation_repo: Arc<dyn AgentAllocationRepository>,
    vector_config_repo: Arc<dyn VectorConfigRepository>,
    mcp_tool_repo: Arc<dyn MCPToolRepository>,
    flow_repo: Arc<dyn FlowRepository>,
}

impl AgentApplicationService {
    // CRUD operations
    pub async fn create_agent(&self, dto: CreateAgentDto, creator_id: UserId) -> Result<AgentDto>;
    pub async fn get_agent(&self, id: AgentId, user_id: UserId) -> Result<AgentDetailDto>;
    pub async fn update_agent(&self, id: AgentId, dto: UpdateAgentDto, user_id: UserId) -> Result<AgentDto>;
    pub async fn delete_agent(&self, id: AgentId, user_id: UserId) -> Result<()>;
    pub async fn list_agents(&self, tenant_id: TenantId, user_id: UserId, pagination: PaginationParams) -> Result<PaginatedResponse<AgentCardDto>>;
    
    // Copy operation
    pub async fn copy_agent(&self, source_id: AgentId, user_id: UserId) -> Result<AgentDto>;
    
    // Employment operations
    pub async fn employ_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;
    pub async fn terminate_employment(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;
    pub async fn list_employed_agents(&self, user_id: UserId, pagination: PaginationParams) -> Result<PaginatedResponse<AgentCardDto>>;
    
    // Allocation operations
    pub async fn allocate_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;
    pub async fn terminate_allocation(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;
    pub async fn list_allocated_agents(&self, user_id: UserId, pagination: PaginationParams) -> Result<PaginatedResponse<AgentCardDto>>;
    
    // Resource management
    pub async fn add_knowledge_base(&self, agent_id: AgentId, config_id: ConfigId, user_id: UserId) -> Result<()>;
    pub async fn remove_knowledge_base(&self, agent_id: AgentId, config_id: ConfigId, user_id: UserId) -> Result<()>;
    pub async fn add_mcp_tool(&self, agent_id: AgentId, tool_id: MCPToolId, user_id: UserId) -> Result<()>;
    pub async fn remove_mcp_tool(&self, agent_id: AgentId, tool_id: MCPToolId, user_id: UserId) -> Result<()>;
    pub async fn add_flow(&self, agent_id: AgentId, flow_id: FlowId, user_id: UserId) -> Result<()>;
    pub async fn remove_flow(&self, agent_id: AgentId, flow_id: FlowId, user_id: UserId) -> Result<()>;
}
```

### 6. DTOs

```rust
// src/application/dto/agent_dto.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentDto {
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub knowledge_base_ids: Vec<Uuid>,
    pub mcp_tool_ids: Vec<Uuid>,
    pub flow_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentDto {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub system_prompt: Option<String>,
    pub additional_settings: Option<String>,
    pub preset_questions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardDto {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt_preview: String,  // First 200 chars
    pub creator_name: String,
    pub is_employed: bool,
    pub is_allocated: bool,
    pub is_creator: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetailDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub knowledge_bases: Vec<VectorConfigSummaryDto>,
    pub mcp_tools: Vec<MCPToolSummaryDto>,
    pub flows: Vec<FlowSummaryDto>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent: Option<AgentSourceDto>,
    pub creator: UserSummaryDto,
    pub is_employed: bool,
    pub is_allocated: bool,
    pub is_creator: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSourceDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummaryDto {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfigSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}
```

### 7. API Handlers

```rust
// src/presentation/handlers/agent_handlers.rs

pub async fn create_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Extension(tenant_id): Extension<TenantId>,
    Json(dto): Json<CreateAgentDto>,
) -> Result<Json<AgentDto>>;

pub async fn get_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentDetailDto>>;

pub async fn update_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
    Json(dto): Json<UpdateAgentDto>,
) -> Result<Json<AgentDto>>;

pub async fn delete_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<StatusCode>;

pub async fn list_agents(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Extension(tenant_id): Extension<TenantId>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AgentCardDto>>>;

pub async fn copy_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentDto>>;

pub async fn employ_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<StatusCode>;

pub async fn terminate_employment(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<StatusCode>;

pub async fn list_employed_agents(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AgentCardDto>>>;

pub async fn allocate_agent(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<StatusCode>;

pub async fn terminate_allocation(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<StatusCode>;

pub async fn list_allocated_agents(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AgentCardDto>>>;

// Resource management endpoints
pub async fn add_knowledge_base(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path((agent_id, config_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode>;

pub async fn remove_knowledge_base(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path((agent_id, config_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode>;

pub async fn add_mcp_tool(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path((agent_id, tool_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode>;

pub async fn remove_mcp_tool(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path((agent_id, tool_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode>;

pub async fn add_flow(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path((agent_id, flow_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode>;

pub async fn remove_flow(
    State(service): State<Arc<AgentApplicationService>>,
    Extension(user_id): Extension<UserId>,
    Path((agent_id, flow_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode>;
```

## Data Models

### Database Schema

#### agents table

```sql
CREATE TABLE agents (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    avatar TEXT,
    knowledge_base_ids JSONB NOT NULL DEFAULT '[]',
    mcp_tool_ids JSONB NOT NULL DEFAULT '[]',
    flow_ids JSONB NOT NULL DEFAULT '[]',
    system_prompt TEXT NOT NULL,
    additional_settings TEXT,
    preset_questions JSONB NOT NULL DEFAULT '[]',
    source_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT preset_questions_max_3 CHECK (jsonb_array_length(preset_questions) <= 3)
);

CREATE INDEX idx_agents_tenant_id ON agents(tenant_id);
CREATE INDEX idx_agents_creator_id ON agents(creator_id);
CREATE INDEX idx_agents_source_agent_id ON agents(source_agent_id);
CREATE INDEX idx_agents_created_at ON agents(created_at DESC);
```

#### agent_employments table

```sql
CREATE TABLE agent_employments (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    employed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (agent_id, user_id)
);

CREATE INDEX idx_agent_employments_user_id ON agent_employments(user_id);
CREATE INDEX idx_agent_employments_agent_id ON agent_employments(agent_id);
```

#### agent_allocations table

```sql
CREATE TABLE agent_allocations (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    allocated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (agent_id, user_id)
);

CREATE INDEX idx_agent_allocations_user_id ON agent_allocations(user_id);
CREATE INDEX idx_agent_allocations_agent_id ON agent_allocations(agent_id);
```

## Error Handling

### Error Types

```rust
// 在 src/error/mod.rs 中添加

pub enum PlatformError {
    // ... existing errors ...
    
    // Agent specific errors
    AgentNotFound(String),
    AgentUnauthorized(String),
    AgentValidationError(String),
    AgentAlreadyEmployed(String),
    AgentNotEmployed(String),
    AgentAlreadyAllocated(String),
    AgentNotAllocated(String),
    PresetQuestionsLimitExceeded,
}
```

### Authorization Checks

所有修改操作必须验证：
1. 用户是否为Agent的创建者
2. 如果不是，返回 `AgentUnauthorized` 错误

## Testing Strategy

### Unit Tests

1. **Domain Entity Tests**
   - Agent创建和验证逻辑
   - 预设问题数量限制
   - 权限检查方法
   - 复制功能

2. **Repository Tests**
   - CRUD操作
   - 分页查询
   - 雇佣关系管理

3. **Application Service Tests**
   - 业务逻辑编排
   - 权限验证
   - 错误处理

### Integration Tests

1. **API Tests**
   - 完整的CRUD流程
   - 权限验证
   - 雇佣关系管理
   - 资源关联管理

2. **Database Tests**
   - 外键约束
   - 级联删除
   - 索引性能

## API Routes

```
POST   /api/v1/agents                          - 创建Agent
GET    /api/v1/agents                          - 列出Agents（分页，卡片样式）
GET    /api/v1/agents/{id}                      - 获取Agent详情
PUT    /api/v1/agents/{id}                      - 更新Agent
DELETE /api/v1/agents/{id}                      - 删除Agent
POST   /api/v1/agents/{id}/copy                 - 复制Agent

POST   /api/v1/agents/{id}/employ               - 雇佣Agent
DELETE /api/v1/agents/{id}/employ               - 终止雇佣
GET    /api/v1/agents/employed                 - 列出已雇佣的Agents

POST   /api/v1/agents/{id}/allocate               - 分配Agent
DELETE /api/v1/agents/{id}/allocate               - 终止分配
GET    /api/v1/agents/allocated                 - 列出已分配的Agents

POST   /api/v1/agents/{id}/knowledge-bases/{config_id}   - 添加知识库
DELETE /api/v1/agents/{id}/knowledge-bases/{config_id}   - 移除知识库
POST   /api/v1/agents/{id}/mcp-tools/{tool_id}           - 添加MCP工具
DELETE /api/v1/agents/{id}/mcp-tools/{tool_id}           - 移除MCP工具
POST   /api/v1/agents/{id}/flows/{flow_id}               - 添加Flow
DELETE /api/v1/agents/{id}/flows/{flow_id}               - 移除Flow
```

## Implementation Notes

### 1. 权限控制

- 所有修改操作（UPDATE, DELETE, 资源管理）必须验证 `agent.creator_id == user_id`
- 查看操作允许同租户内所有用户访问
- 雇佣操作允许任何用户执行

### 2. 数据验证

- Agent名称：非空，最大255字符
- 系统提示词：非空
- 预设问题：最多3个
- Avatar：可选，URL格式

### 3. 复制功能

- 复制时创建新的Agent ID
- 设置 `source_agent_id` 指向原Agent
- 设置 `creator_id` 为执行复制的用户
- 复制所有配置属性和资源关联

### 4. 卡片样式数据

列表API返回的卡片数据应包含：
- Agent基本信息（ID, 名称, 头像）
- 系统提示词预览（前200字符）
- 创建者信息
- 当前用户是否已雇佣
- 当前用户是否为创建者

### 5. 级联删除

- 删除Agent时自动删除所有雇佣关系
- 删除用户时自动删除其创建的Agents
- 删除租户时自动删除所有Agents

### 6. 性能优化

- 使用索引优化查询性能
- 列表查询支持分页
- 详情查询使用JOIN减少数据库往返
