use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

use crate::{
    domain::repositories::FileRepository,
    error::{PlatformError, Result},
};

pub struct FileRepositoryImpl {
    storage_path: PathBuf,
    base_url: String,
}

impl FileRepositoryImpl {
    pub fn new(storage_path: PathBuf, base_url: String) -> Self {
        Self {
            storage_path,
            base_url,
        }
    }
    
    fn get_file_path(&self, tenant_id: &str, file_id: &str, filename: &str) -> PathBuf {
        self.storage_path
            .join(tenant_id)
            .join(file_id)
            .join(filename)
    }
    
    fn get_file_url(&self, tenant_id: &str, file_id: &str, filename: &str) -> String {
        // format!("{}/files/{}/{}/{}", self.base_url, tenant_id, file_id, filename)
        format!("{}/{}/{}/{}", self.base_url, tenant_id, file_id, filename)
    }
}

#[async_trait]
impl FileRepository for FileRepositoryImpl {
    async fn store_file(
        &self,
        tenant_id: &str,
        _user_id: &str,
        file_id: &str,
        filename: &str,
        _content_type: &str,
        data: Vec<u8>,
    ) -> Result<String> {
        let file_path = self.get_file_path(tenant_id, file_id, filename);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                PlatformError::InternalError(format!("Failed to create directory: {}", e))
            })?;
        }
        
        // Write file
        fs::write(&file_path, data).await.map_err(|e| {
            PlatformError::InternalError(format!("Failed to write file: {}", e))
        })?;
        
        Ok(self.get_file_url(tenant_id, file_id, filename))
    }
    
    async fn delete_file(&self, tenant_id: &str, file_id: &str) -> Result<()> {
        let dir_path = self.storage_path.join(tenant_id).join(file_id);
        
        if dir_path.exists() {
            fs::remove_dir_all(&dir_path).await.map_err(|e| {
                PlatformError::InternalError(format!("Failed to delete file: {}", e))
            })?;
        }
        
        Ok(())
    }
    
    async fn get_file_url(&self, tenant_id: &str, file_id: &str) -> Result<String> {
        let dir_path = self.storage_path.join(tenant_id).join(file_id);
        
        // Find the first file in the directory
        let mut entries = fs::read_dir(&dir_path).await.map_err(|e| {
            PlatformError::NotFound(format!("File not found: {}", e))
        })?;
        
        if let Some(entry) = entries.next_entry().await.map_err(|e| {
            PlatformError::InternalError(format!("Failed to read directory: {}", e))
        })? {
            let filename = entry.file_name().to_string_lossy().to_string();
            Ok(self.get_file_url(tenant_id, file_id, &filename))
        } else {
            Err(PlatformError::NotFound("File not found".to_string()))
        }
    }
}
