# 阿里云OSS文件上传实现

## 概述

已成功实现基于阿里云OSS的文件上传功能，文件将被存储到阿里云OSS上，并返回可下载的URL。

## 实现内容

### 1. 添加依赖

在 `Cargo.toml` 中添加了阿里云OSS客户端依赖：
```toml
aliyun-oss-client = "0.11"
```

### 2. 配置项

在 `src/config/mod.rs` 中添加了OSS配置结构：
```rust
pub struct OssConfig {
    pub endpoint: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub upload_path: String,
    pub download_domain: String,
}
```

### 3. 环境变量

在 `.env` 和 `.env.example` 中添加了以下配置项：
```bash
# OSS Configuration
OSS_ENDPOINT=oss-cn-beijing.aliyuncs.com
OSS_ACCESS_KEY_ID=your-access-key-id
OSS_ACCESS_KEY_SECRET=your-access-key-secret
OSS_BUCKET=your-bucket-name
OSS_UPLOAD_PATH=uploads
OSS_DOWNLOAD_DOMAIN=https://your-bucket-name.oss-cn-beijing.aliyuncs.com
```

### 4. OSS文件存储实现

创建了 `src/infrastructure/repositories/oss_file_repository_impl.rs`，实现了 `FileRepository` trait：

- **store_file**: 上传文件到OSS，返回下载URL
- **delete_file**: 删除指定租户和文件ID下的所有文件
- **get_file_url**: 获取文件的下载URL

文件路径格式：`{upload_path}/{tenant_id}/{file_id}/{filename}`

### 5. 服务集成

在 `src/presentation/server.rs` 中，将文件服务切换为使用OSS存储：
```rust
let file_repository: Arc<dyn FileRepository> = Arc::new(
    OssFileRepositoryImpl::new(self.config.oss.clone())
        .expect("Failed to initialize OSS client")
);
```

## API接口

### 上传文件

**端点**: `POST /api/files/upload`

**请求**: multipart/form-data
- `file`: 要上传的文件

**响应**:
```json
{
  "url": "https://your-bucket.oss-cn-beijing.aliyuncs.com/uploads/tenant_id/file_id/filename.ext",
  "filename": "filename.ext",
  "size": 12345,
  "content_type": "image/png"
}
```

## 配置说明

- **OSS_ENDPOINT**: OSS服务的endpoint，如 `oss-cn-beijing.aliyuncs.com`
- **OSS_ACCESS_KEY_ID**: 阿里云访问密钥ID
- **OSS_ACCESS_KEY_SECRET**: 阿里云访问密钥Secret
- **OSS_BUCKET**: OSS存储桶名称
- **OSS_UPLOAD_PATH**: 文件上传的基础路径，默认为 `uploads`
- **OSS_DOWNLOAD_DOMAIN**: 文件下载域名，用于拼接完整的下载URL

## 使用示例

1. 配置环境变量（在 `.env` 文件中）
2. 启动服务
3. 使用multipart/form-data格式上传文件到 `/api/files/upload`
4. 获取返回的URL用于下载文件

## 注意事项

1. 确保OSS bucket已创建并配置了正确的访问权限
2. 确保OSS_DOWNLOAD_DOMAIN配置正确，可以是OSS默认域名或自定义域名
3. 文件会按照 `{upload_path}/{tenant_id}/{file_id}/{filename}` 的路径结构存储
4. 需要有效的阿里云访问密钥才能使用OSS服务
