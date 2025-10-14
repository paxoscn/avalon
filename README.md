# Agent Platform

A comprehensive, production-ready agent platform built with Rust, featuring flow execution, multi-model integration, MCP tool support, and enterprise-grade monitoring.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)]()

## 🚀 Features

### Core Capabilities
- **🔄 Flow Execution**: Execute complex agent workflows with support for Dify DSL import
- **🤖 Multi-Model Support**: Integrate with various LLM providers (OpenAI, Claude, Local LLMs)
- **📊 Vector Database Integration**: Support for Pinecone, Weaviate, ChromaDB, Qdrant, and Milvus
- **🔧 MCP Tool Support**: Convert HTTP APIs to MCP tools with version management
- **🏢 Multi-Tenant Architecture**: Secure tenant isolation with JWT authentication
- **📝 Version Management**: Full version control for flows and tools with rollback support
- **📈 Audit & Monitoring**: Comprehensive audit logging and execution history
- **⚡ High Performance**: Async Rust with connection pooling and Redis caching
- **🔒 Security**: Rate limiting, JWT authentication, and tenant isolation

### Enterprise Features
- **Health Checks**: Kubernetes-ready liveness and readiness probes
- **Metrics**: Prometheus-compatible metrics endpoint
- **Horizontal Scaling**: Stateless design for easy scaling
- **Database Backups**: Automated backup and restore scripts
- **Docker Support**: Full containerization with Docker Compose
- **Kubernetes Ready**: Production-ready K8s manifests with HPA

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Installation](#installation)
- [Configuration](#configuration)
- [Deployment](#deployment)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Testing](#testing)
- [Monitoring](#monitoring)
- [Contributing](#contributing)
- [License](#license)

## ⚡ Quick Start

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd agent-platform

# Start all services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f backend
```

Access the application:
- Backend API: http://localhost:8080
- Frontend: http://localhost:3000
- Health Check: http://localhost:8080/health

### Using Cargo (Development)

```bash
# Install dependencies
cargo build

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Run migrations
cargo run --bin migrate

# Start server
cargo run

# Or with hot reload
cargo watch -x run
```

## 🏗️ Architecture

The project follows Domain-Driven Design (DDD) principles with a clean, layered architecture:

```
┌─────────────────────────────────────────┐
│         Presentation Layer              │
│    (REST API, Middleware, Routes)       │
├─────────────────────────────────────────┤
│         Application Layer               │
│  (Services, DTOs, Commands, Queries)    │
├─────────────────────────────────────────┤
│           Domain Layer                  │
│ (Entities, Value Objects, Services)     │
├─────────────────────────────────────────┤
│       Infrastructure Layer              │
│ (Database, Cache, External Services)    │
└─────────────────────────────────────────┘
```

### Key Design Patterns
- **Repository Pattern**: Abstract data access
- **Service Layer**: Business logic encapsulation
- **Dependency Injection**: Loose coupling
- **CQRS**: Separate read and write operations
- **Event Sourcing**: Audit trail and history

## 🛠️ Technology Stack

### Backend
- **Language**: Rust 1.75+
- **Web Framework**: Axum (async, high-performance)
- **Database**: MySQL 8.0+ with SeaORM
- **Cache**: Redis 6.0+
- **Authentication**: JWT with bcrypt
- **Serialization**: Serde (JSON/YAML)
- **Async Runtime**: Tokio
- **HTTP Client**: Reqwest

### Frontend
- **Framework**: React 18+ with TypeScript
- **State Management**: Zustand
- **UI**: Tailwind CSS + Headless UI
- **Build Tool**: Vite
- **HTTP Client**: Axios

### DevOps
- **Containerization**: Docker & Docker Compose
- **Orchestration**: Kubernetes
- **Monitoring**: Prometheus metrics
- **Logging**: Structured logging with tracing

## 📦 Installation

### Prerequisites

- **Rust**: 1.75 or higher
- **MySQL**: 8.0 or higher
- **Redis**: 6.0 or higher
- **Docker**: 20.10+ (optional)
- **Node.js**: 18+ (for frontend)

### Step-by-Step Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd agent-platform
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   ```

3. **Configure environment variables**
   ```env
   DATABASE_URL=mysql://agent_user:agent_password@localhost:3306/agent_platform
   REDIS_URL=redis://localhost:6379
   JWT_SECRET=$(openssl rand -base64 32)
   RUST_LOG=info
   SERVER_HOST=0.0.0.0
   SERVER_PORT=8080
   ```

4. **Set up database**
   ```bash
   # Create database
   mysql -u root -p -e "CREATE DATABASE agent_platform CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
   
   # Create user
   mysql -u root -p -e "CREATE USER 'agent_user'@'%' IDENTIFIED BY 'agent_password';"
   mysql -u root -p -e "GRANT ALL PRIVILEGES ON agent_platform.* TO 'agent_user'@'%';"
   mysql -u root -p -e "FLUSH PRIVILEGES;"
   ```

5. **Build and run**
   ```bash
   # Build
   cargo build --release
   
   # Run migrations (automatic on startup)
   # Start server
   ./target/release/agent-platform
   ```

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | MySQL connection string | - | Yes |
| `REDIS_URL` | Redis connection string | - | Yes |
| `JWT_SECRET` | Secret for JWT signing | - | Yes |
| `SERVER_HOST` | Server bind address | 0.0.0.0 | No |
| `SERVER_PORT` | Server port | 8080 | No |
| `RUST_LOG` | Log level | info | No |
| `DB_MAX_CONNECTIONS` | Max DB connections | 100 | No |
| `RATE_LIMIT_MAX_REQUESTS` | Rate limit per window | 1000 | No |
| `CACHE_DEFAULT_TTL_SECONDS` | Cache TTL | 300 | No |
| `CORS_ALLOW_ALL_LOCALHOST` | Allow all localhost origins | true | No |
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed origins | - | No |

### Configuration Files

- `config/default.yaml`: Default configuration
- `config/local.yaml`: Local overrides (gitignored)
- `.env`: Environment-specific variables

## 🚢 Deployment

### Docker Deployment

```bash
# Build image
docker build -t agent-platform:latest .

# Run container
docker run -d \
  --name agent-platform \
  -p 8080:8080 \
  -e DATABASE_URL=mysql://user:pass@host:3306/db \
  -e REDIS_URL=redis://host:6379 \
  -e JWT_SECRET=your-secret \
  agent-platform:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# Scale backend
docker-compose up -d --scale backend=3

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Kubernetes

```bash
# Create namespace
kubectl create namespace agent-platform

# Create secrets
kubectl create secret generic agent-platform-secrets \
  --from-literal=database-url='mysql://...' \
  --from-literal=redis-url='redis://...' \
  --from-literal=jwt-secret='...' \
  -n agent-platform

# Deploy
kubectl apply -f k8s/deployment.yaml -n agent-platform

# Check status
kubectl get pods -n agent-platform
kubectl get svc -n agent-platform

# Scale
kubectl scale deployment agent-platform --replicas=5 -n agent-platform
```

See [Deployment Guide](docs/deployment_guide.md) for detailed instructions.

## 📚 API Documentation

### Authentication

```bash
# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "tenant-uuid",
    "username": "user",
    "password": "pass"
  }'

# Refresh token
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Flow Management

```bash
# List flows
curl http://localhost:8080/api/flows?tenant_id=uuid \
  -H "Authorization: Bearer YOUR_TOKEN"

# Create flow
curl -X POST http://localhost:8080/api/flows \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Flow",
    "description": "Flow description",
    "definition": {"nodes": [], "edges": []}
  }'

# Execute flow
curl -X POST http://localhost:8080/api/flows/{id}/execute \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"variables": {}}'
```

See [API Documentation](docs/api_documentation.md) for complete API reference.

## 💻 Development

### Project Structure

```
agent-platform/
├── src/
│   ├── main.rs                 # Entry point
│   ├── config/                 # Configuration
│   ├── domain/                 # Domain layer (DDD)
│   ├── application/            # Application layer
│   ├── infrastructure/         # Infrastructure layer
│   └── presentation/           # Presentation layer
├── frontend/                   # React frontend
├── tests/                      # Integration tests
├── docs/                       # Documentation
├── scripts/                    # Utility scripts
├── k8s/                        # Kubernetes manifests
├── Dockerfile                  # Docker configuration
├── docker-compose.yml          # Docker Compose
└── Cargo.toml                  # Rust dependencies
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test api_integration_tests -- --test-threads=1

# Performance tests
cargo test --test performance_tests --release -- --nocapture

# With coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Security audit
cargo audit

# Check dependencies
cargo outdated
```

## 📊 Monitoring

### Health Checks

```bash
# Basic health
curl http://localhost:8080/health

# Detailed health
curl http://localhost:8080/health/detailed

# Liveness probe (K8s)
curl http://localhost:8080/health/live

# Readiness probe (K8s)
curl http://localhost:8080/health/ready
```

### Metrics

```bash
# JSON metrics
curl http://localhost:8080/metrics

# Prometheus metrics
curl http://localhost:8080/metrics/prometheus
```

### Database Backup

```bash
# Manual backup
./scripts/backup-database.sh

# Restore from backup
./scripts/restore-database.sh ./backups/backup_file.sql.gz
```

## 📖 Documentation

- [User Guide](docs/user_guide.md) - Complete user documentation
- [API Documentation](docs/api_documentation.md) - REST API reference
- [Deployment Guide](docs/deployment_guide.md) - Deployment instructions
- [CORS Configuration](docs/cors_configuration.md) - CORS setup and troubleshooting
- [Routes Integration](docs/routes_integration_summary.md) - Route modules status and integration guide
- [Adding Routes](docs/adding_routes.md) - How to integrate route modules
- [Testing Guide](docs/testing_guide.md) - Testing strategies
- [Troubleshooting](docs/troubleshooting.md) - Common issues and solutions

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Write tests for new features
- Update documentation
- Ensure CI passes
- Keep commits atomic and well-described

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum)
- Database ORM by [SeaORM](https://www.sea-ql.org/SeaORM/)
- Inspired by modern agent platforms

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-repo/issues)
- **Documentation**: [docs.example.com](https://docs.example.com)
- **Email**: support@example.com

---

**Made with ❤️ using Rust**