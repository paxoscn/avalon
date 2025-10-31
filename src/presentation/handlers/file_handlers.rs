use axum::{
    extract::{Multipart, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    application::services::FileApplicationService,
    error::{PlatformError, Result},
    presentation::extractors::AuthenticatedUser,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub content_type: String,
}

/// Upload a file
pub async fn upload_file(
    user: AuthenticatedUser,
    State(service): State<Arc<dyn FileApplicationService>>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>> {
    // Extract file from multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        PlatformError::ValidationError(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let filename = field
                .file_name()
                .ok_or_else(|| PlatformError::ValidationError("Missing filename".to_string()))?
                .to_string();
            
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            
            let data = field.bytes().await.map_err(|e| {
                PlatformError::ValidationError(format!("Failed to read file data: {}", e))
            })?;
            
            let size = data.len() as u64;
            
            // Upload file through service
            let url = service
                .upload_file(&user.tenant_id.to_string(), &user.user_id.to_string(), filename.clone(), content_type.clone(), data.to_vec())
                .await?;
            
            return Ok(Json(FileUploadResponse {
                url,
                filename,
                size,
                content_type,
            }));
        }
    }
    
    Err(PlatformError::ValidationError("No file field found in multipart form".to_string()))
}
