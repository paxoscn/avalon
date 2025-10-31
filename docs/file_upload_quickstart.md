# 文件上传功能快速启动指南

## 快速开始

### 1. 启动后端服务

```bash
# 确保uploads目录存在
mkdir -p uploads

# 启动服务器
cargo run --release
```

服务器将在 `http://localhost:8080` 启动。

### 2. 启动前端

```bash
cd frontend
npm install
npm run dev
```

前端将在 `http://localhost:5173` 启动。

### 3. 创建支持文件上传的Flow

1. 登录系统
2. 创建新Flow或编辑现有Flow
3. 在Flow的start节点中添加`file-list`类型变量：

```json
{
  "node_type": "start",
  "data": {
    "variables": [
      {
        "variable": "documents",
        "type": "file-list",
        "default": [],
        "description": "上传的文档列表"
      }
    ]
  }
}
```

### 4. 测试文件上传

1. 进入Flow测试页面
2. 找到`documents`变量（显示为文件上传按钮）
3. 点击"选择文件"按钮
4. 选择一个或多个文件
5. 文件自动上传，显示在列表中
6. 点击"执行Flow"

### 5. 验证文件URL

执行Flow后，在执行详情中可以看到传递的文件URL：

```json
{
  "input_data": {
    "documents": [
      "http://localhost:8080/files/tenant-id/file-id-1/document1.pdf",
      "http://localhost:8080/files/tenant-id/file-id-2/document2.pdf"
    ]
  }
}
```

## 使用API直接上传

### cURL示例

```bash
# 1. 登录获取token
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "test-tenant",
    "username": "admin",
    "password": "password"
  }' | jq -r '.token')

# 2. 上传文件
curl -X POST http://localhost:8080/api/files/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/path/to/your/file.pdf"
```

### JavaScript示例

```javascript
// 1. 上传文件
const formData = new FormData();
formData.append('file', fileInput.files[0]);

const response = await fetch('http://localhost:8080/api/files/upload', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`
  },
  body: formData
});

const result = await response.json();
console.log('File URL:', result.url);

// 2. 在Flow执行中使用
await fetch(`http://localhost:8080/api/flows/${flowId}/execute`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    input_data: {
      documents: [result.url]
    }
  })
});
```

## 常见问题

### Q: 上传失败，提示"Failed to read multipart field"

A: 确保请求的Content-Type是`multipart/form-data`，并且表单字段名为`file`。

### Q: 文件上传成功但无法下载

A: 检查：
1. `uploads`目录是否存在且有读写权限
2. 文件路径是否正确
3. 服务器配置的base_url是否正确

### Q: 如何限制文件大小？

A: 在`file_handlers.rs`中添加大小检查：

```rust
let size = data.len() as u64;
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

if size > MAX_FILE_SIZE {
    return Err(PlatformError::ValidationError(
        "File size exceeds 10MB limit".to_string()
    ));
}
```

### Q: 如何限制文件类型？

A: 在`file_handlers.rs`中添加类型检查：

```rust
const ALLOWED_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "application/pdf",
];

if !ALLOWED_TYPES.contains(&content_type.as_str()) {
    return Err(PlatformError::ValidationError(
        "File type not allowed".to_string()
    ));
}
```

### Q: 如何使用云存储（S3/OSS）？

A: 实现新的`FileRepository`：

```rust
pub struct S3FileRepository {
    client: S3Client,
    bucket: String,
}

#[async_trait]
impl FileRepository for S3FileRepository {
    async fn store_file(...) -> Result<String> {
        // 上传到S3
        // 返回S3 URL
    }
}
```

然后在`server.rs`中替换实现：

```rust
let file_repository = Arc::new(S3FileRepository::new(
    s3_client,
    "my-bucket".to_string(),
));
```

## 运行测试

```bash
# 确保服务器正在运行
./scripts/test_file_upload.sh
```

## 下一步

- 查看 [API文档](./file_upload_api.md) 了解详细的API规范
- 查看 [功能说明](./file_upload_feature.md) 了解实现细节
- 查看 [实现总结](./file_upload_implementation_summary.md) 了解架构设计

## 需要帮助？

如有问题，请查看：
1. 服务器日志：检查错误信息
2. 浏览器控制台：检查前端错误
3. 网络请求：使用开发者工具查看请求详情
