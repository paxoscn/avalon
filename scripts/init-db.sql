-- Database initialization script
-- This script is run when the MySQL container first starts

-- Create database if it doesn't exist
CREATE DATABASE IF NOT EXISTS agent_platform CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

USE agent_platform;

-- Grant privileges
GRANT ALL PRIVILEGES ON agent_platform.* TO 'agent_user'@'%';
FLUSH PRIVILEGES;

-- The actual table creation is handled by SeaORM migrations
-- This script just ensures the database exists and permissions are set
