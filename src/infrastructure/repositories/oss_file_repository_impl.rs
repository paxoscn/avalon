use async_trait::async_trait;
use aliyun_oss_client::Client;

use crate::{
    config::OssConfig,
    domain::repositories::FileRepository,
    error::{PlatformError, Result},
};

pub struct OssFileRepositoryImpl {
    client: Client,
    upload_path: String,
    download_domain: String,
}

impl OssFileRepositoryImpl {
    pub fn new(config: OssConfig) -> Result<Self> {
        use aliyun_oss_client::{KeyId, KeySecret, EndPoint, BucketName};
        
        let key_id: KeyId = config.access_key_id.clone().into();
        let key_secret: KeySecret = config.access_key_secret.clone().into();
        let endpoint: EndPoint = config.endpoint.clone().try_into()
            .map_err(|e| PlatformError::InternalError(format!("Invalid endpoint: {:?}", e)))?;
        let bucket: BucketName = config.bucket.clone().try_into()
            .map_err(|e| PlatformError::InternalError(format!("Invalid bucket name: {:?}", e)))?;
        
        let client = Client::new(key_id, key_secret, endpoint, bucket);
        
        Ok(Self {
            client,
            upload_path: config.upload_path,
            download_domain: config.download_domain,
        })
    }
    
    fn build_object_path(&self, tenant_id: &str, file_id: &str, filename: &str) -> String {
        format!("{}/{}/{}/{}", self.upload_path, tenant_id, file_id, filename)
    }
    
    fn build_download_url(&self, object_path: &str) -> String {
        format!("{}/{}", self.download_domain.trim_end_matches('/'), object_path)
    }
}

#[async_trait]
impl FileRepository for OssFileRepositoryImpl {
    async fn store_file(
        &self,
        tenant_id: &str,
        _user_id: &str,
        file_id: &str,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> Result<String> {
        use aliyun_oss_client::file::Files;
        
        let object_path = self.build_object_path(tenant_id, file_id, filename);
        let content_type_static: &'static str = Box::leak(content_type.to_string().into_boxed_str());
        let object_path_static: &'static str = Box::leak(object_path.clone().into_boxed_str());
        
        // Upload to OSS using put_content
        self.client.clone()
            .put_content(data, object_path_static, |_| {
                Some(content_type_static)
            })
            .await
            .map_err(|e| PlatformError::InternalError(format!("Failed to upload to OSS: {}", e)))?;
        
        Ok(self.build_download_url(&object_path))
    }
    
    async fn delete_file(&self, tenant_id: &str, file_id: &str) -> Result<()> {
        use aliyun_oss_client::{QueryKey, QueryValue, ObjectPath};
        use aliyun_oss_client::file::Files;
        
        // List and delete all files under the tenant_id/file_id prefix
        let prefix = format!("{}/{}/{}/", self.upload_path, tenant_id, file_id);
        let prefix_key: QueryKey = "prefix".into();
        let prefix_value: QueryValue = prefix.into();
        
        // List objects with prefix
        let objects = self.client.clone()
            .get_object_list(vec![(prefix_key, prefix_value)])
            .await
            .map_err(|e| PlatformError::InternalError(format!("Failed to list objects: {}", e)))?;
        
        // Delete each object
        #[allow(deprecated)]
        for object in objects.object_list {
            let path: &ObjectPath = object.as_ref();
            let path_str: &str = path.as_ref();
            let path_static: &'static str = Box::leak(path_str.to_string().into_boxed_str());
            self.client.clone()
                .delete_object(path_static)
                .await
                .map_err(|e| PlatformError::InternalError(format!("Failed to delete object: {}", e)))?;
        }
        
        Ok(())
    }
    
    async fn get_file_url(&self, tenant_id: &str, file_id: &str) -> Result<String> {
        use aliyun_oss_client::{QueryKey, QueryValue, ObjectPath};
        
        let prefix = format!("{}/{}/{}/", self.upload_path, tenant_id, file_id);
        let prefix_key: QueryKey = "prefix".into();
        let prefix_value: QueryValue = prefix.into();
        
        // List objects with prefix
        let objects = self.client.clone()
            .get_object_list(vec![(prefix_key, prefix_value)])
            .await
            .map_err(|e| PlatformError::InternalError(format!("Failed to list objects: {}", e)))?;
        
        #[allow(deprecated)]
        if let Some(object) = objects.object_list.first() {
            let path: &ObjectPath = object.as_ref();
            let path_str: &str = path.as_ref();
            Ok(self.build_download_url(path_str))
        } else {
            Err(PlatformError::NotFound("File not found".to_string()))
        }
    }
}
