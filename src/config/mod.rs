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
        })
    }
}