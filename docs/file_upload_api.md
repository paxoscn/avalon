# 文件上传 API 文档

## 概述

文件上传功能允许用户上传文件并获取下载URL，这些URL可以在Flow执行时作为参数传递。

## API 端点

### 上传文件

**端点**: `POST /api/files/upload`

**认证**: 需要 Bearer Token

**请求格式**: `multipart/form-data`

**请求参数**:
- `file`: 要上传的文件（必需）

**响应格式**: JSON

**成功响应** (200 OK):
```json
{
  "url": "http://localhost:8080/files/tenant-id/file-id/filename.ext",
  "filename": "filename.ext",
  "size": 12345,
  "content_type": "image/png"
}
```

**错误响应**:
- `400 Bad Request`: 请求格式错误或缺少文件
- `401 Unauthorized`: 未提供认证令牌或令牌无效
- `500 Internal Server Error`: 服务器内部错误

## 使用示例

### cURL 示例

```bash
curl -X POST http://localhost:8080/api/files/upload \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -F "file=@/path/to/your/file.pdf"
```

### JavaScript/Fetch 示例

```javascript
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
```

### React 示例

```typescript
const handleFileUpload = async (file: File) => {
  const formData = new FormData();
  formData.append('file', file);

  try {
    const response = await apiClient.post('/files/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    
    return response.data.url;
  } catch (error) {
    console.error('Upload failed:', error);
    throw error;
  }
};
```

## 文件存储

### 存储位置

文件默认存储在服务器的 `/tmp/uploads` 目录下，按以下结构组织：

```
uploads/
  └── {tenant_id}/
      └── {file_id}/
          └── {filename}
```

### 访问文件

上传的文件可以通过以下URL访问：

```
http://localhost:8080/files/{tenant_id}/{file_id}/{filename}
```

注意：文件下载不需要认证。

## 配置

### 修改存储路径

在 `src/presentation/server.rs` 中修改文件存储路径：

```rust
let file_repository = Arc::new(FileRepositoryImpl::new(
    std::path::PathBuf::from("/tmp/uploads"),  // 修改此路径
    format!("http://{}:{}", self.config.server.host, self.config.server.port),
));
```

### 修改基础URL

如果使用反向代理或CDN，可以修改基础URL：

```rust
let file_repository = Arc::new(FileRepositoryImpl::new(
    std::path::PathBuf::from("/tmp/uploads"),
    "https://your-cdn.com".to_string(),  // 使用自定义URL
));
```

## 在 Flow 中使用

### 1. 定义 file-list 类型变量

在Flow的start节点中定义变量：

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

### 2. 上传文件

使用前端界面或API上传文件，获取URL列表。

### 3. 执行 Flow

将文件URL数组作为参数传递：

```json
{
  "input_data": {
    "documents": [
      "http://localhost:8080/files/tenant-id/file-id-1/doc1.pdf",
      "http://localhost:8080/files/tenant-id/file-id-2/doc2.pdf"
    ]
  }
}
```

## 安全考虑

1. **文件大小限制**: 建议在生产环境中添加文件大小限制
2. **文件类型验证**: 建议验证上传的文件类型
3. **病毒扫描**: 建议在生产环境中集成病毒扫描
4. **访问控制**: 当前文件下载不需要认证，根据需求可能需要添加访问控制
5. **存储配额**: 建议实现租户级别的存储配额管理

## 未来改进

- [ ] 添加文件大小限制配置
- [ ] 添加文件类型白名单/黑名单
- [ ] 实现文件删除API
- [ ] 添加文件元数据存储（数据库）
- [ ] 支持云存储（S3, OSS等）
- [ ] 实现文件访问权限控制
- [ ] 添加文件上传进度跟踪
- [ ] 实现文件压缩和优化
- [ ] 添加文件预览功能
