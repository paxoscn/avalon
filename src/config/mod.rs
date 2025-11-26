use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub bcrypt_cost: u32,
    pub cors: CorsConfig,
    pub downloading_base_url: String,
    pub oss: OssConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OssConfig {
    pub endpoint: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub upload_path: String,
    pub download_domain: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_all_localhost: bool,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost:3306/agent_platform".to_string());
        
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let host = env::var("APP_SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        let port = env::var("APP_SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string());

        let bcrypt_cost = env::var("BCRYPT_COST")
            .unwrap_or_else(|_| "12".to_string())
            .parse::<u32>()
            .unwrap_or(12);

        // CORS configuration
        let allow_all_localhost = env::var("CORS_ALLOW_ALL_LOCALHOST")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| String::new())
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        
        let downloading_base_url = env::var("APP_DOWNLOADING_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

        // OSS configuration
        let oss_endpoint = env::var("OSS_ENDPOINT")
            .unwrap_or_else(|_| "oss-cn-beijing.aliyuncs.com".to_string());
        let oss_access_key_id = env::var("OSS_ACCESS_KEY_ID")
            .unwrap_or_else(|_| String::new());
        let oss_access_key_secret = env::var("OSS_ACCESS_KEY_SECRET")
            .unwrap_or_else(|_| String::new());
        let oss_bucket = env::var("OSS_BUCKET")
            .unwrap_or_else(|_| "my-bucket".to_string());
        let oss_upload_path = env::var("OSS_UPLOAD_PATH")
            .unwrap_or_else(|_| "uploads".to_string());
        let oss_download_domain = env::var("OSS_DOWNLOAD_DOMAIN")
            .unwrap_or_else(|_| format!("https://{}.{}", oss_bucket, oss_endpoint));

        Ok(AppConfig {
            server: ServerConfig { host, port },
            database_url,
            redis_url,
            jwt_secret,
            bcrypt_cost,
            cors: CorsConfig {
                allowed_origins,
                allow_all_localhost,
            },
            downloading_base_url,
            oss: OssConfig {
                endpoint: oss_endpoint,
                access_key_id: oss_access_key_id,
                access_key_secret: oss_access_key_secret,
                bucket: oss_bucket,
                upload_path: oss_upload_path,
                download_domain: oss_download_domain,
            },
        })
    }
}