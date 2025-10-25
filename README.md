# Cobalt Stack

Modern full-stack application template with Rust backend (Axum + SeaORM) and TypeScript frontend (Next.js + Bun).

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ ([install](https://rustup.rs/))
- **Docker** & **Docker Compose** ([install](https://docs.docker.com/get-docker/))
- **Make** (usually pre-installed on macOS/Linux)
- **uv/uvx** for Python tooling ([install](https://docs.astral.sh/uv/))

### Initial Setup

```bash
# Clone and setup
git clone <your-repo-url> cobalt-stack
cd cobalt-stack

# Copy environment files and install dependencies
make setup
```

This will:
- Copy `backend/.env.example` to `backend/.env`
- Install Rust dependencies with `cargo build`

### Development

#### Run Full Stack (Docker Compose)

```bash
# Start all services (PostgreSQL, Redis, Backend)
make dev

# Access the API
curl http://localhost:3000/health
# {"status":"healthy"}
```

#### Run Backend Only (Hot Reload)

```bash
# Installs cargo-watch if not present
make dev-backend

# Backend runs on http://localhost:3000
# Auto-reloads on file changes
```

## 🧪 Testing (TDD Workflow)

### Run Tests

```bash
# Run all tests
make test

# Run tests in watch mode (auto-rerun on changes)
make test-watch

# Run tests with coverage report
make test-coverage
# Open backend/coverage/index.html in browser
```

### TDD Workflow Example

```bash
# 1. RED: Write failing test
cd backend
cargo test test_new_feature  # Should fail

# 2. GREEN: Implement minimum code to pass
# Edit src/handlers/your_handler.rs
cargo test test_new_feature  # Should pass

# 3. REFACTOR: Improve code quality
cargo clippy
cargo fmt
```

## 📦 Building

### Local Build

```bash
# Build optimized release binary
make build

# Binary location: backend/target/release/cobalt-stack-backend
```

### Docker Build

```bash
# Build development image
make docker-build

# Build production image
make docker-build-prod
```

**Docker Optimization Features:**
- **cargo-chef**: Separates dependency compilation for layer caching
- **BuildKit Cache Mounts**: Persistent cache for cargo registry, git, and target
- **Distroless Runtime**: Minimal image (~50MB) with gcr.io/distroless/cc-debian12
- **Non-root Execution**: Runs as UID 65534 for security

## 🗄️ Database Migrations

```bash
# Run all pending migrations
make migrate

# Create new migration
cd backend
sea-orm-cli migrate generate create_users_table
```

## 📝 OpenAPI Schema

```bash
# Generate OpenAPI schema (utoipa)
make generate-openapi
```

## 🛠️ Development Tools

### Code Quality

```bash
# Run linter (clippy)
make lint

# Format code
make fmt

# Check formatting without changes
make fmt-check

# Run all CI checks (fmt, lint, test)
make ci
```

### Cleanup

```bash
# Clean build artifacts and stop containers
make clean
```

## 📁 Project Structure

```
cobalt-stack/
├── backend/
│   ├── src/
│   │   ├── handlers/      # HTTP request handlers (thin adapters)
│   │   ├── models/        # SeaORM database models
│   │   ├── services/      # Business logic layer (DDD)
│   │   ├── middleware/    # Request/response middleware
│   │   ├── utils/         # Utility functions
│   │   ├── openapi/       # OpenAPI schema generation
│   │   └── main.rs        # Application entry point
│   ├── migration/         # SeaORM migrations
│   ├── tests/             # Integration tests
│   ├── Cargo.toml         # Backend dependencies
│   └── Dockerfile         # Optimized multi-stage build
├── frontend/              # Next.js + Bun (coming soon)
├── docker-compose.yml     # Development environment
├── docker-compose.prod.yml # Production environment
├── Makefile               # Task automation
└── openspec/              # OpenSpec proposals and specs
```

## 🏗️ Architecture

### Backend (Rust)

- **Framework**: Axum 0.7+ (async web framework)
- **ORM**: SeaORM 1.1+ (PostgreSQL)
- **Runtime**: Tokio (async runtime)
- **Testing**: cargo-test + cargo-tarpaulin (coverage)
- **API Docs**: utoipa (OpenAPI 3.0 generation)

### Domain-Driven Design

The backend follows DDD principles with clear separation:

- **Handlers** (`src/handlers/`): Thin HTTP adapters, no business logic
- **Services** (`src/services/`): Business logic, domain rules, orchestration
- **Models** (`src/models/`): SeaORM entities, database schema mapping

**Test Coverage Requirements:**
- Domain logic (services): **100% coverage** (MANDATORY)
- Application layer (handlers): **80%+ coverage**
- Integration tests for all API endpoints

### Frontend (TypeScript + Bun)

- **Framework**: Next.js 14+ (App Router)
- **Runtime**: Bun (fast JavaScript runtime)
- **UI Components**: shadcn/ui + Tailwind CSS
- **Type Safety**: TypeScript strict mode
- **API Client**: Generated from OpenAPI schema

## 🐳 Docker Environments

### Development (`docker-compose.yml`)

- **PostgreSQL**: Port 5432 exposed, persistent volume
- **Redis**: Port 6379 exposed, no password
- **Backend**: Hot reload, debug logging, CORS enabled

### Production (`docker-compose.prod.yml`)

- **Security**: Password-protected Redis, non-root execution
- **Resource Limits**: 2 CPU, 2GB RAM for backend
- **Logging**: JSON structured logs with rotation (20MB × 5 files)
- **Health Checks**: All services with automatic restart

**Required Environment Variables for Production:**
```bash
DATABASE_PASSWORD  # PostgreSQL password
REDIS_PASSWORD     # Redis password
JWT_SECRET         # JWT signing secret
```

## 🔧 Configuration

### Environment Variables

Copy `backend/.env.example` to `backend/.env` and configure:

```bash
# Server
PORT=3000
RUST_LOG=debug  # info for production

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/cobalt_dev

# Redis (optional)
REDIS_URL=redis://localhost:6379

# Security
JWT_SECRET=change-me-in-production

# CORS
CORS_ORIGINS=http://localhost:3001,http://localhost:3000
```

## 🚨 Troubleshooting

### Common Issues

**1. Port Already in Use**
```bash
# Check what's using port 3000
lsof -i :3000
# Kill the process or change PORT in .env
```

**2. Database Connection Failed**
```bash
# Ensure PostgreSQL is running
docker-compose ps

# Check connection string in .env
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/cobalt_dev
```

**3. cargo-watch Not Found**
```bash
# Install manually
cargo install cargo-watch
```

**4. Migration Errors**
```bash
# Reset database (WARNING: destroys data)
docker-compose down -v
docker-compose up -d postgres
make migrate
```

**5. Docker Build Slow**
```bash
# Enable BuildKit for cache mounts
export DOCKER_BUILDKIT=1
make docker-build
```

### Debug Mode

```bash
# Backend with verbose logging
RUST_LOG=trace cargo run

# Test specific module
cargo test handlers::health -- --nocapture
```

## 📚 Resources

- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [SeaORM Guide](https://www.sea-ql.org/SeaORM/docs/index/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [OpenSpec Workflow](./openspec/README.md)
- [Docker Optimization Guide](https://syu-m-5151.hatenablog.com/entry/2025/10/17/070250)

## 📄 License

[Your License Here]

## 🤝 Contributing

1. Create OpenSpec proposal for significant changes
2. Follow TDD workflow (Red → Green → Refactor)
3. Ensure all tests pass and coverage meets requirements
4. Run `make ci` before submitting PR

---

**Built with ❤️ using Rust, Axum, SeaORM, and Next.js**
