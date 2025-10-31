# 文件上传功能更新日志

## 版本: v0.2.0 - 文件上传功能

### 发布日期: 2024-10-31

### 新增功能

#### 🎉 文件上传支持

为Flow测试页面添加了完整的文件上传功能，支持在Flow执行时传递文件URL。

**主要特性：**

- ✅ 多文件上传支持
- ✅ 文件列表管理（添加/删除）
- ✅ 自动检测`file-list`类型变量
- ✅ 实时上传进度指示
- ✅ 文件信息展示（名称、大小）
- ✅ RESTful API接口
- ✅ 本地文件存储
- ✅ 静态文件服务

### 技术实现

#### 前端 (Frontend)

**新增文件：**
- `frontend/src/services/file.service.ts` - 文件上传服务

**修改文件：**
- `frontend/src/pages/FlowTestPage.tsx` - 添加文件上传UI

**新增功能：**
- 文件选择和上传组件
- 文件列表显示和管理
- 上传状态指示
- 错误处理

#### 后端 (Backend)

**新增文件：**
- `src/presentation/handlers/file_handlers.rs` - 文件上传处理器
- `src/presentation/routes/file_routes.rs` - 文件路由
- `src/application/services/file_service.rs` - 文件应用服务
- `src/domain/repositories/file_repository.rs` - 文件仓储接口
- `src/infrastructure/repositories/file_repository_impl.rs` - 文件仓储实现

**修改文件：**
- `src/presentation/server.rs` - 注册文件路由和静态服务
- `src/presentation/handlers/mod.rs` - 导出文件处理器
- `src/presentation/routes/mod.rs` - 导出文件路由
- `src/domain/repositories/mod.rs` - 导出文件仓储
- `src/infrastructure/repositories/mod.rs` - 导出文件仓储实现
- `src/application/services/mod.rs` - 导出文件服务
- `Cargo.toml` - 添加必要的依赖特性

**依赖更新：**
- `axum` - 添加 `multipart` 特性
- `tower-http` - 添加 `fs` 特性

### API 端点

#### 上传文件
```
POST /api/files/upload
Authorization: Bearer <token>
Content-Type: multipart/form-data

Response:
{
  "url": "http://localhost:8080/files/{tenant_id}/{file_id}/{filename}",
  "filename": "example.pdf",
  "size": 12345,
  "content_type": "application/pdf"
}
```

#### 下载文件
```
GET /files/{tenant_id}/{file_id}/{filename}

Response: 文件内容
```

### 使用方法

#### 1. 在Flow中定义file-list变量

```json
{
  "node_type": "start",
  "data": {
    "variables": [
      {
        "variable": "documents",
        "type": "file-list",
        "default": []
      }
    ]
  }
}
```

#### 2. 在测试页面上传文件

1. 进入Flow测试页面
2. 点击"选择文件"按钮
3. 选择一个或多个文件
4. 文件自动上传并显示在列表中
5. 执行Flow时，文件URL会作为参数传递

#### 3. 通过API上传

```bash
curl -X POST http://localhost:8080/api/files/upload \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -F "file=@/path/to/file.pdf"
```

### 文件存储

**存储位置：** `/tmp/uploads/`

**目录结构：**
```
uploads/
  └── {tenant_id}/
      └── {file_id}/
          └── {filename}
```

**访问URL：**
```
http://localhost:8080/files/{tenant_id}/{file_id}/{filename}
```

### 文档

新增以下文档：

- `docs/file_upload_quickstart.md` - 快速启动指南
- `docs/file_upload_api.md` - API详细文档
- `docs/file_upload_feature.md` - 功能说明
- `docs/file_upload_implementation_summary.md` - 实现总结

### 测试

新增测试脚本：
- `scripts/test_file_upload.sh` - 文件上传功能测试

运行测试：
```bash
./scripts/test_file_upload.sh
```

### 配置

#### 修改存储路径

在 `src/presentation/server.rs` 中：

```rust
let file_repository = Arc::new(FileRepositoryImpl::new(
    std::path::PathBuf::from("/tmp/uploads"),  // 修改此路径
    format!("http://{}:{}", self.config.server.host, self.config.server.port),
));
```

#### 使用云存储

实现自定义的`FileRepository`来支持S3、OSS等云存储服务。

### 安全考虑

⚠️ **生产环境建议：**

1. 添加文件大小限制
2. 实现文件类型白名单
3. 添加病毒扫描
4. 实现文件访问权限控制
5. 使用云存储服务（S3、OSS等）
6. 实现文件清理策略
7. 监控存储空间使用

### 已知限制

- 当前仅支持本地文件存储
- 文件下载不需要认证
- 没有文件大小限制
- 没有文件类型限制
- 没有自动清理机制

### 未来改进计划

- [ ] 支持云存储（S3、OSS、Azure Blob等）
- [ ] 添加文件大小和类型限制配置
- [ ] 实现文件访问权限控制
- [ ] 添加文件元数据存储（数据库）
- [ ] 实现文件删除API
- [ ] 添加文件预览功能
- [ ] 支持断点续传
- [ ] 实现文件压缩和优化
- [ ] 添加文件清理策略
- [ ] 实现存储配额管理

### 迁移指南

如果你已经有现有的Flow：

1. 更新Flow定义，将需要文件上传的变量类型改为`file-list`
2. 前端会自动识别并显示文件上传按钮
3. 无需修改Flow执行逻辑，文件URL会作为字符串数组传递

### 贡献者

- 实现者：AI Assistant
- 审核者：待定

### 相关Issue

- Feature Request: 支持文件上传功能
- Issue: Flow测试需要传递文件参数

### 参考资料

- [Axum Multipart Documentation](https://docs.rs/axum/latest/axum/extract/struct.Multipart.html)
- [Tower HTTP File Serving](https://docs.rs/tower-http/latest/tower_http/services/fs/index.html)

---

## 快速开始

```bash
# 1. 确保uploads目录存在
mkdir -p uploads

# 2. 启动后端
cargo run --release

# 3. 启动前端
cd frontend && npm run dev

# 4. 运行测试
./scripts/test_file_upload.sh
```

详细使用说明请参考 [快速启动指南](docs/file_upload_quickstart.md)。
