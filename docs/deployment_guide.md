# Agent Platform Deployment Guide

## Overview

This guide covers deploying the Agent Platform in various environments, from local development to production Kubernetes clusters.

## Prerequisites

- Docker and Docker Compose (for containerized deployment)
- MySQL 8.0+ database
- Redis 6.0+ cache server
- Rust 1.75+ (for building from source)
- Kubernetes cluster (for production deployment)

## Quick Start with Docker Compose

The fastest way to get started is using Docker Compose:

```bash
# Clone the repository
git clone <repository-url>
cd agent-platform

# Copy environment file
cp .env.example .env

# Edit .env with your configuration
nano .env

# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f backend
```

The application will be available at:
- Backend API: http://localhost:8080
- Frontend: http://localhost:3000

## Environment Configuration

### Required Environment Variables

```bash
# Database Configuration
DATABASE_URL=mysql://agent_user:agent_password@localhost:3306/agent_platform

# Redis Configuration
REDIS_URL=redis://localhost:6379

# JWT Configuration
JWT_SECRET=your-secret-key-change-in-production
JWT_EXPIRATION_HOURS=24

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Logging
RUST_LOG=info
```

### Optional Environment Variables

```bash
# Rate Limiting
RATE_LIMIT_MAX_REQUESTS=1000
RATE_LIMIT_WINDOW_SECONDS=60

# Cache TTL
CACHE_DEFAULT_TTL_SECONDS=300

# Database Connection Pool
DB_MAX_CONNECTIONS=100
DB_MIN_CONNECTIONS=10
```

## Building from Source

### Development Build

```bash
# Install dependencies
cargo build

# Run migrations
cargo run --bin migrate

# Start development server
cargo run
```

### Production Build

```bash
# Build optimized binary
cargo build --release

# The binary will be at target/release/agent-platform
./target/release/agent-platform
```

## Docker Deployment

### Building Docker Image

```bash
# Build the image
docker build -t agent-platform:latest .

# Run the container
docker run -d \
  --name agent-platform \
  -p 8080:8080 \
  -e DATABASE_URL=mysql://user:pass@host:3306/db \
  -e REDIS_URL=redis://host:6379 \
  -e JWT_SECRET=your-secret \
  agent-platform:latest
```

### Docker Compose Production Setup

```yaml
version: '3.8'

services:
  backend:
    image: agent-platform:latest
    environment:
      DATABASE_URL: ${DATABASE_URL}
      REDIS_URL: ${REDIS_URL}
      JWT_SECRET: ${JWT_SECRET}
    ports:
      - "8080:8080"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Kubernetes Deployment

### Prerequisites

- Kubernetes cluster (1.20+)
- kubectl configured
- Helm (optional, for easier management)

### Deploy to Kubernetes

```bash
# Create namespace
kubectl create namespace agent-platform

# Create secrets
kubectl create secret generic agent-platform-secrets \
  --from-literal=database-url='mysql://user:pass@mysql:3306/agent_platform' \
  --from-literal=redis-url='redis://redis:6379' \
  --from-literal=jwt-secret='your-secret-key' \
  -n agent-platform

# Apply deployment
kubectl apply -f k8s/deployment.yaml -n agent-platform

# Check deployment status
kubectl get pods -n agent-platform
kubectl get svc -n agent-platform

# View logs
kubectl logs -f deployment/agent-platform -n agent-platform
```

### Scaling

```bash
# Manual scaling
kubectl scale deployment agent-platform --replicas=5 -n agent-platform

# Horizontal Pod Autoscaler is configured automatically
kubectl get hpa -n agent-platform
```

## Database Setup

### Initial Setup

```bash
# Create database
mysql -u root -p -e "CREATE DATABASE agent_platform CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Create user
mysql -u root -p -e "CREATE USER 'agent_user'@'%' IDENTIFIED BY 'agent_password';"
mysql -u root -p -e "GRANT ALL PRIVILEGES ON agent_platform.* TO 'agent_user'@'%';"
mysql -u root -p -e "FLUSH PRIVILEGES;"
```

### Running Migrations

Migrations are automatically run on application startup. To run manually:

```bash
# Using cargo
cargo run --bin migrate

# Using the binary
./agent-platform migrate
```

### Database Backup

```bash
# Run backup script
./scripts/backup-database.sh

# Backups are stored in ./backups/ directory
# Retention: 7 days by default
```

### Database Restore

```bash
# List available backups
ls -lh ./backups/

# Restore from backup
./scripts/restore-database.sh ./backups/agent_platform_20240101_120000.sql.gz
```

## Monitoring and Health Checks

### Health Check Endpoints

- **Basic Health**: `GET /health`
- **Detailed Health**: `GET /health/detailed`
- **Liveness Probe**: `GET /health/live`
- **Readiness Probe**: `GET /health/ready`
- **Metrics**: `GET /metrics`
- **Prometheus Metrics**: `GET /metrics/prometheus`

### Example Health Check

```bash
# Check if service is healthy
curl http://localhost:8080/health

# Get detailed status
curl http://localhost:8080/health/detailed

# Get metrics
curl http://localhost:8080/metrics
```

### Prometheus Integration

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'agent-platform'
    static_configs:
      - targets: ['agent-platform:8080']
    metrics_path: '/metrics/prometheus'
    scrape_interval: 15s
```

## Performance Tuning

### Database Optimization

```sql
-- Add indexes for frequently queried columns
CREATE INDEX idx_flows_tenant_status ON flows(tenant_id, status);
CREATE INDEX idx_executions_status_date ON flow_executions(status, started_at);
CREATE INDEX idx_audit_tenant_date ON audit_logs(tenant_id, created_at);
```

### Redis Configuration

```bash
# Increase max memory
redis-cli CONFIG SET maxmemory 2gb
redis-cli CONFIG SET maxmemory-policy allkeys-lru

# Enable persistence
redis-cli CONFIG SET save "900 1 300 10 60 10000"
```

### Application Tuning

```bash
# Increase connection pool size
export DB_MAX_CONNECTIONS=200

# Adjust cache TTL
export CACHE_DEFAULT_TTL_SECONDS=600

# Configure rate limiting
export RATE_LIMIT_MAX_REQUESTS=2000
```

## Troubleshooting

### Common Issues

#### Database Connection Failed

```bash
# Check database is running
mysql -h localhost -u agent_user -p agent_platform

# Verify connection string
echo $DATABASE_URL

# Check network connectivity
telnet mysql-host 3306
```

#### Redis Connection Failed

```bash
# Check Redis is running
redis-cli ping

# Verify connection string
echo $REDIS_URL

# Test connection
redis-cli -h redis-host -p 6379 ping
```

#### Application Won't Start

```bash
# Check logs
docker-compose logs backend

# Verify environment variables
docker-compose exec backend env | grep DATABASE_URL

# Check migrations
docker-compose exec backend ./agent-platform migrate --check
```

### Performance Issues

```bash
# Check database slow queries
mysql -u root -p -e "SELECT * FROM mysql.slow_log ORDER BY query_time DESC LIMIT 10;"

# Monitor Redis memory
redis-cli INFO memory

# Check application metrics
curl http://localhost:8080/metrics
```

## Security Considerations

### Production Checklist

- [ ] Change default JWT_SECRET
- [ ] Use strong database passwords
- [ ] Enable TLS/SSL for database connections
- [ ] Configure firewall rules
- [ ] Enable Redis authentication
- [ ] Set up rate limiting
- [ ] Configure CORS properly
- [ ] Enable audit logging
- [ ] Regular security updates
- [ ] Backup encryption

### Recommended Security Settings

```bash
# Strong JWT secret (32+ characters)
JWT_SECRET=$(openssl rand -base64 32)

# Enable TLS for database
DATABASE_URL=mysql://user:pass@host:3306/db?ssl-mode=REQUIRED

# Redis with password
REDIS_URL=redis://:password@host:6379

# Restrict CORS
CORS_ALLOWED_ORIGINS=https://yourdomain.com
```

## Backup and Recovery

### Automated Backups

Set up a cron job for regular backups:

```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * /path/to/agent-platform/scripts/backup-database.sh

# Add weekly backup to remote storage
0 3 * * 0 /path/to/backup-to-s3.sh
```

### Disaster Recovery

1. **Database Recovery**:
   ```bash
   ./scripts/restore-database.sh ./backups/latest.sql.gz
   ```

2. **Redis Recovery**:
   ```bash
   redis-cli --rdb /path/to/dump.rdb
   ```

3. **Application State**:
   - Redeploy from Docker image
   - Restore configuration from version control
   - Verify health checks pass

## Upgrading

### Rolling Update (Kubernetes)

```bash
# Update image
kubectl set image deployment/agent-platform \
  agent-platform=agent-platform:v2.0.0 \
  -n agent-platform

# Monitor rollout
kubectl rollout status deployment/agent-platform -n agent-platform

# Rollback if needed
kubectl rollout undo deployment/agent-platform -n agent-platform
```

### Zero-Downtime Update (Docker Compose)

```bash
# Pull new image
docker-compose pull backend

# Recreate with new image
docker-compose up -d --no-deps --build backend

# Verify
docker-compose ps
curl http://localhost:8080/health
```

## Support

For issues and questions:
- GitHub Issues: <repository-url>/issues
- Documentation: <docs-url>
- Email: support@example.com
