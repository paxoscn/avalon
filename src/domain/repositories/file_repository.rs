use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait FileRepository: Send + Sync {
    /// Store a file and return its download URL
    async fn store_file(
        &self,
        tenant_id: &str,
        user_id: &str,
        file_id: &str,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> Result<String>;
    
    /// Delete a file
    async fn delete_file(&self, tenant_id: &str, file_id: &str) -> Result<()>;
    
    /// Get file metadata
    async fn get_file_url(&self, tenant_id: &str, file_id: &str) -> Result<String>;
}
