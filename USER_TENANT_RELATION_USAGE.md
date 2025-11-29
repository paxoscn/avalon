# User-Tenant 多对多关联表使用指南

## 概述

已创建 `user_tenant_relations` 表来支持 User 与 Tenant 之间的多对多关系。这允许一个用户属于多个租户，一个租户也可以拥有多个用户。

## 数据库结构

### 表结构
```sql
CREATE TABLE user_tenant_relations (
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, tenant_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
```

### 索引
- 主键: `(user_id, tenant_id)`
- 索引: `idx_user_tenant_relation_user_id` on `user_id`
- 索引: `idx_user_tenant_relation_tenant_id` on `tenant_id`

## 使用示例

### 1. 添加用户到租户

```rust
use sea_orm::*;
use crate::infrastructure::database::entities::user_tenant_relation;

// 创建关联
let relation = user_tenant_relation::ActiveModel {
    user_id: Set(user_id),
    tenant_id: Set(tenant_id),
    created_at: Set(Utc::now()),
};

relation.insert(db).await?;
```

### 2. 查询用户所属的所有租户

```rust
use sea_orm::*;
use crate::infrastructure::database::entities::{user, user_tenant_relation, tenant};

// 通过关联表查询
let tenants = user_tenant_relation::Entity::find()
    .filter(user_tenant_relation::Column::UserId.eq(user_id))
    .find_also_related(tenant::Entity)
    .all(db)
    .await?;
```

### 3. 查询租户下的所有用户

```rust
use sea_orm::*;
use crate::infrastructure::database::entities::{user, user_tenant_relation, tenant};

// 通过关联表查询
let users = user_tenant_relation::Entity::find()
    .filter(user_tenant_relation::Column::TenantId.eq(tenant_id))
    .find_also_related(user::Entity)
    .all(db)
    .await?;
```

### 4. 检查用户是否属于某个租户

```rust
use sea_orm::*;
use crate::infrastructure::database::entities::user_tenant_relation;

let exists = user_tenant_relation::Entity::find()
    .filter(user_tenant_relation::Column::UserId.eq(user_id))
    .filter(user_tenant_relation::Column::TenantId.eq(tenant_id))
    .one(db)
    .await?
    .is_some();
```

### 5. 移除用户与租户的关联

```rust
use sea_orm::*;
use crate::infrastructure::database::entities::user_tenant_relation;

user_tenant_relation::Entity::delete_many()
    .filter(user_tenant_relation::Column::UserId.eq(user_id))
    .filter(user_tenant_relation::Column::TenantId.eq(tenant_id))
    .exec(db)
    .await?;
```

### 6. 获取用户在所有租户中的关联数量

```rust
use sea_orm::*;
use crate::infrastructure::database::entities::user_tenant_relation;

let count = user_tenant_relation::Entity::find()
    .filter(user_tenant_relation::Column::UserId.eq(user_id))
    .count(db)
    .await?;
```

## 注意事项

1. **级联删除**: 当用户或租户被删除时，相关的关联记录会自动删除
2. **唯一性**: 同一个用户和租户的组合只能存在一条记录（通过复合主键保证）
3. **原有关系**: User 表中仍保留 `tenant_id` 字段，表示用户的主租户。多对多关系表用于支持用户访问多个租户的场景
4. **时间戳**: `created_at` 字段记录关联创建的时间

## 迁移

运行以下命令应用数据库迁移：

```bash
# 运行迁移
cargo run --bin migrator up

# 或使用 sea-orm-cli
sea-orm-cli migrate up
```

## 文件清单

- **实体**: `src/infrastructure/database/entities/user_tenant_relation.rs`
- **迁移**: `src/infrastructure/database/migrations/m20241129_000002_create_user_tenant_relations.rs`
- **更新的文件**:
  - `src/infrastructure/database/entities/user.rs` - 添加关联关系
  - `src/infrastructure/database/entities/tenant.rs` - 添加关联关系
  - `src/infrastructure/database/entities/mod.rs` - 导出新实体
  - `src/infrastructure/database/migrations/mod.rs` - 导出新迁移
  - `src/infrastructure/database/migrator.rs` - 注册新迁移
