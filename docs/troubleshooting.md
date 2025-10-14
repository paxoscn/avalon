# Agent Platform Troubleshooting Guide

## Quick Diagnostic Checklist

Before diving into specific issues, run through this quick checklist:

- [ ] Check service health: `curl http://localhost:8080/health/detailed`
- [ ] Verify database connection: `mysql -h localhost -u agent_user -p agent_platform`
- [ ] Test Redis connection: `redis-cli ping`
- [ ] Check logs: `docker-compose logs -f backend` or `kubectl logs -f deployment/agent-platform`
- [ ] Verify environment variables are set correctly
- [ ] Ensure all required services are running

## Common Issues and Solutions

### 1. Application Won't Start

#### Symptom
```
Error: Failed to connect to database
```

#### Diagnosis
```bash
# Check if MySQL is running
docker-compose ps mysql
# or
systemctl status mysql

# Test database connection
mysql -h localhost -u agent_user -p agent_platform -e "SELECT 1;"

# Check environment variable
echo $DATABASE_URL
```

#### Solutions

**Solution 1: Database not running**
```bash
# Start MySQL
docker-compose up -d mysql
# or
systemctl start mysql
```

**Solution 2: Wrong credentials**
```bash
# Update .env file with correct credentials
DATABASE_URL=mysql://correct_user:correct_password@localhost:3306/agent_platform

# Restart application
docker-compose restart backend
```

**Solution 3: Database doesn't exist**
```bash
# Create database
mysql -u root -p -e "CREATE DATABASE agent_platform CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Run migrations
cargo run --bin migrate
```

### 2. Redis Connection Issues

#### Symptom
```
Error: Redis connection refused
```

#### Diagnosis
```bash
# Check if Redis is running
docker-compose ps redis
redis-cli ping

# Check Redis logs
docker-compose logs redis
```

#### Solutions

**Solution 1: Redis not running**
```bash
# Start Redis
docker-compose up -d redis
# or
systemctl start redis
```

**Solution 2: Wrong Redis URL**
```bash
# Update .env
REDIS_URL=redis://localhost:6379

# If Redis has password
REDIS_URL=redis://:password@localhost:6379
```

**Solution 3: Redis out of memory**
```bash
# Check memory usage
redis-cli INFO memory

# Increase max memory
redis-cli CONFIG SET maxmemory 2gb

# Set eviction policy
redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

### 3. Authentication Failures

#### Symptom
```
401 Unauthorized
```

#### Diagnosis
```bash
# Test login endpoint
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "your-tenant-id",
    "username": "testuser",
    "password": "password"
  }'

# Check JWT secret is set
echo $JWT_SECRET
```

#### Solutions

**Solution 1: Invalid credentials**
- Verify username and password are correct
- Check tenant_id matches user's tenant
- Ensure user exists in database:
  ```sql
  SELECT * FROM users WHERE username = 'testuser';
  ```

**Solution 2: JWT secret not set**
```bash
# Set JWT secret
export JWT_SECRET=$(openssl rand -base64 32)

# Or in .env file
echo "JWT_SECRET=$(openssl rand -base64 32)" >> .env

# Restart application
docker-compose restart backend
```

**Solution 3: Token expired**
- Tokens expire after 24 hours by default
- Use refresh token endpoint:
  ```bash
  curl -X POST http://localhost:8080/api/auth/refresh \
    -H "Authorization: Bearer YOUR_TOKEN"
  ```

### 4. Flow Execution Failures

#### Symptom
```
Flow execution failed with status: failed
```

#### Diagnosis
```bash
# Get execution details
curl http://localhost:8080/api/executions/{execution_id} \
  -H "Authorization: Bearer YOUR_TOKEN"

# Check audit logs
curl "http://localhost:8080/api/audit?resource_type=flow_execution&resource_id={execution_id}" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

#### Solutions

**Solution 1: Missing required inputs**
- Check flow definition for required variables
- Ensure all inputs are provided in execution request
- Verify input data types match expected types

**Solution 2: LLM API failure**
```bash
# Test LLM configuration
curl -X POST http://localhost:8080/api/config/llm/{config_id}/test \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello, world!"}'

# Check API key is valid
# Verify rate limits haven't been exceeded
# Try alternative LLM configuration
```

**Solution 3: Vector database error**
```bash
# Test vector configuration
curl -X POST http://localhost:8080/api/config/vector/{config_id}/test \
  -H "Authorization: Bearer YOUR_TOKEN"

# Check vector database is accessible
# Verify collection/index exists
# Ensure documents are uploaded
```

**Solution 4: MCP tool failure**
```bash
# Test MCP tool
curl -X POST http://localhost:8080/api/mcp/tools/{tool_id}/call \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"parameters": {"test": "value"}}'

# Verify endpoint URL is correct
# Check API credentials
# Test endpoint directly with curl
```

### 5. Performance Issues

#### Symptom
- Slow response times
- High CPU/memory usage
- Database connection pool exhausted

#### Diagnosis
```bash
# Check application metrics
curl http://localhost:8080/metrics

# Monitor database connections
mysql -u root -p -e "SHOW PROCESSLIST;"

# Check Redis memory
redis-cli INFO memory

# View slow queries
mysql -u root -p -e "SELECT * FROM mysql.slow_log ORDER BY query_time DESC LIMIT 10;"
```

#### Solutions

**Solution 1: Database query optimization**
```sql
-- Add missing indexes
CREATE INDEX idx_flows_tenant_status ON flows(tenant_id, status);
CREATE INDEX idx_executions_status_date ON flow_executions(status, started_at);
CREATE INDEX idx_audit_tenant_date ON audit_logs(tenant_id, created_at);
CREATE INDEX idx_messages_session ON chat_messages(session_id, created_at);

-- Analyze tables
ANALYZE TABLE flows, flow_executions, chat_sessions, chat_messages;
```

**Solution 2: Increase connection pool**
```bash
# Update .env
DB_MAX_CONNECTIONS=200
DB_MIN_CONNECTIONS=20

# Restart application
docker-compose restart backend
```

**Solution 3: Enable caching**
```bash
# Verify Redis is working
redis-cli ping

# Check cache hit rate
redis-cli INFO stats | grep keyspace

# Clear cache if needed
redis-cli FLUSHDB
```

**Solution 4: Scale horizontally**
```bash
# Docker Compose
docker-compose up -d --scale backend=3

# Kubernetes
kubectl scale deployment agent-platform --replicas=5
```

### 6. Memory Leaks

#### Symptom
- Memory usage continuously increases
- Application becomes unresponsive
- OOM (Out of Memory) errors

#### Diagnosis
```bash
# Monitor memory usage
docker stats agent-platform-backend

# Check for memory leaks in logs
docker-compose logs backend | grep -i "memory\|oom"

# Get heap dump (if available)
# Use tools like valgrind or heaptrack
```

#### Solutions

**Solution 1: Restart application**
```bash
# Quick fix - restart
docker-compose restart backend

# Set up automatic restart on OOM
docker-compose up -d --restart=unless-stopped
```

**Solution 2: Increase memory limits**
```yaml
# docker-compose.yml
services:
  backend:
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 512M
```

**Solution 3: Fix application code**
- Review code for unclosed connections
- Check for large data structures in memory
- Ensure proper cleanup of resources
- Use connection pooling correctly

### 7. Migration Failures

#### Symptom
```
Error: Migration failed at version X
```

#### Diagnosis
```bash
# Check migration status
cargo run --bin migrate -- status

# View migration history
mysql -u agent_user -p agent_platform -e "SELECT * FROM seaql_migrations;"

# Check for partial migrations
mysql -u agent_user -p agent_platform -e "SHOW TABLES;"
```

#### Solutions

**Solution 1: Rollback and retry**
```bash
# Rollback last migration
cargo run --bin migrate -- down

# Retry migration
cargo run --bin migrate -- up
```

**Solution 2: Manual fix**
```bash
# Connect to database
mysql -u agent_user -p agent_platform

# Check what's wrong
SHOW CREATE TABLE problematic_table;

# Fix manually if needed
ALTER TABLE problematic_table ADD COLUMN missing_column VARCHAR(255);

# Mark migration as complete
INSERT INTO seaql_migrations (version, applied_at) VALUES ('version_number', NOW());
```

**Solution 3: Fresh start (development only)**
```bash
# WARNING: This deletes all data
mysql -u root -p -e "DROP DATABASE agent_platform;"
mysql -u root -p -e "CREATE DATABASE agent_platform CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Run migrations
cargo run --bin migrate
```

### 8. Docker Issues

#### Symptom
- Containers won't start
- Network connectivity issues
- Volume mount problems

#### Diagnosis
```bash
# Check container status
docker-compose ps

# View logs
docker-compose logs

# Inspect container
docker inspect agent-platform-backend

# Check networks
docker network ls
docker network inspect agent-network
```

#### Solutions

**Solution 1: Clean and rebuild**
```bash
# Stop all containers
docker-compose down

# Remove volumes (WARNING: deletes data)
docker-compose down -v

# Rebuild images
docker-compose build --no-cache

# Start fresh
docker-compose up -d
```

**Solution 2: Fix network issues**
```bash
# Recreate network
docker network rm agent-network
docker network create agent-network

# Restart containers
docker-compose up -d
```

**Solution 3: Fix volume permissions**
```bash
# Check volume permissions
docker-compose exec backend ls -la /app

# Fix ownership
docker-compose exec backend chown -R appuser:appuser /app
```

### 9. Kubernetes Issues

#### Symptom
- Pods in CrashLoopBackOff
- Service not accessible
- Persistent volume issues

#### Diagnosis
```bash
# Check pod status
kubectl get pods -n agent-platform

# View pod logs
kubectl logs -f deployment/agent-platform -n agent-platform

# Describe pod for events
kubectl describe pod <pod-name> -n agent-platform

# Check service
kubectl get svc -n agent-platform
kubectl describe svc agent-platform -n agent-platform
```

#### Solutions

**Solution 1: Fix pod issues**
```bash
# Delete failing pod (will be recreated)
kubectl delete pod <pod-name> -n agent-platform

# Check resource limits
kubectl describe pod <pod-name> -n agent-platform | grep -A 5 "Limits"

# Increase resources if needed
kubectl edit deployment agent-platform -n agent-platform
```

**Solution 2: Fix service issues**
```bash
# Check endpoints
kubectl get endpoints agent-platform -n agent-platform

# Verify selector matches pods
kubectl get pods -n agent-platform --show-labels

# Recreate service if needed
kubectl delete svc agent-platform -n agent-platform
kubectl apply -f k8s/deployment.yaml
```

**Solution 3: Fix secrets**
```bash
# Verify secrets exist
kubectl get secrets -n agent-platform

# Recreate secrets
kubectl delete secret agent-platform-secrets -n agent-platform
kubectl create secret generic agent-platform-secrets \
  --from-literal=database-url='...' \
  --from-literal=redis-url='...' \
  --from-literal=jwt-secret='...' \
  -n agent-platform

# Restart pods
kubectl rollout restart deployment/agent-platform -n agent-platform
```

## Debugging Tools

### Logging

```bash
# Set log level
export RUST_LOG=debug

# View structured logs
docker-compose logs backend | jq

# Follow logs in real-time
docker-compose logs -f --tail=100 backend
```

### Database Debugging

```sql
-- Check table sizes
SELECT 
  table_name,
  ROUND(((data_length + index_length) / 1024 / 1024), 2) AS "Size (MB)"
FROM information_schema.TABLES
WHERE table_schema = 'agent_platform'
ORDER BY (data_length + index_length) DESC;

-- Find slow queries
SELECT * FROM mysql.slow_log 
ORDER BY query_time DESC 
LIMIT 10;

-- Check locks
SHOW OPEN TABLES WHERE In_use > 0;

-- Active connections
SHOW PROCESSLIST;
```

### Redis Debugging

```bash
# Monitor commands in real-time
redis-cli MONITOR

# Get all keys (use carefully in production)
redis-cli KEYS '*'

# Check specific key
redis-cli GET "key_name"

# Get key TTL
redis-cli TTL "key_name"

# Memory usage by key
redis-cli --bigkeys
```

### Network Debugging

```bash
# Test connectivity
curl -v http://localhost:8080/health

# Check DNS resolution
nslookup mysql
nslookup redis

# Test port connectivity
telnet mysql 3306
telnet redis 6379

# Trace route
traceroute mysql
```

## Getting Help

### Before Asking for Help

1. Check this troubleshooting guide
2. Review application logs
3. Check health endpoints
4. Verify configuration
5. Try basic solutions (restart, etc.)

### Information to Provide

When reporting issues, include:

1. **Environment**:
   - OS and version
   - Docker/Kubernetes version
   - Application version

2. **Error Details**:
   - Complete error message
   - Stack trace if available
   - Relevant log entries

3. **Steps to Reproduce**:
   - What you were trying to do
   - Steps that led to the error
   - Expected vs actual behavior

4. **Configuration**:
   - Environment variables (redact secrets)
   - Docker Compose or K8s manifests
   - Any custom configurations

5. **Diagnostics**:
   - Health check output
   - Database connection test
   - Redis connection test

### Support Channels

- **GitHub Issues**: https://github.com/your-repo/issues
- **Documentation**: https://docs.example.com
- **Email**: support@example.com
- **Community Forum**: https://forum.example.com

## Preventive Measures

### Monitoring

1. Set up health check monitoring
2. Configure alerts for failures
3. Monitor resource usage
4. Track error rates
5. Review audit logs regularly

### Maintenance

1. Regular database backups
2. Update dependencies
3. Review and optimize queries
4. Clean up old data
5. Test disaster recovery procedures

### Best Practices

1. Use version control for configurations
2. Document custom changes
3. Test in staging before production
4. Keep secrets secure
5. Follow security best practices

---

**Last Updated**: 2024-01-01  
**Version**: 1.0.0
