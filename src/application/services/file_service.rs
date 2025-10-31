use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::repositories::FileRepository,
    error::Result,
};

#[async_trait]
pub trait FileApplicationService: Send + Sync {
    async fn upload_file(
        &self,
        tenant_id: &str,
        user_id: &str,
        filename: String,
        content_type: String,
        data: Vec<u8>,
    ) -> Result<String>;
}

pub struct FileApplicationServiceImpl {
    repository: Arc<dyn FileRepository>,
}

impl FileApplicationServiceImpl {
    pub fn new(repository: Arc<dyn FileRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl FileApplicationService for FileApplicationServiceImpl {
    async fn upload_file(
        &self,
        tenant_id: &str,
        user_id: &str,
        filename: String,
        content_type: String,
        data: Vec<u8>,
    ) -> Result<String> {
        // Generate unique file ID
        let file_id = Uuid::new_v4().to_string();
        
        // Store file through repository
        let url = self.repository
            .store_file(tenant_id, user_id, &file_id, &filename, &content_type, data)
            .await?;
        
        Ok(url)
    }
}
