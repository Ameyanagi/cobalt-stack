# Installation Guide

Complete installation instructions for Cobalt Stack.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation Methods](#installation-methods)
- [Environment Configuration](#environment-configuration)
- [Development Setup](#development-setup)
- [Production Setup](#production-setup)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements

- **OS**: Linux, macOS, or Windows with WSL2
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 10GB free space
- **CPU**: 2 cores minimum, 4 cores recommended

### Software Requirements

#### Required

- **Docker**: 20.10+ and Docker Compose 2.0+
- **Git**: 2.30+
- **Node.js**: 18.0+ (for frontend development)
- **Rust**: 1.70+ (for backend development)

#### Optional

- **PostgreSQL**: 14+ (if not using Docker)
- **Make**: For convenience commands
- **Curl**: For API testing

## Installation Methods

### Method 1: Docker (Recommended)

Quickest way to get started with all services.

```bash
# Clone repository
git clone https://github.com/yourusername/cobalt-stack.git
cd cobalt-stack

# Copy environment file
cp .env.example .env

# Start all services
docker-compose up -d

# Verify services
docker-compose ps
```

### Method 2: Native Development

For development with hot reloading and debugging.

#### Backend Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib

# Or using Homebrew (macOS)
brew install postgresql
brew services start postgresql

# Clone and setup
git clone https://github.com/yourusername/cobalt-stack.git
cd cobalt-stack/backend

# Install dependencies
cargo build

# Setup database
createdb cobalt
psql cobalt < migrations/init.sql

# Run backend
cargo run
```

#### Frontend Setup

```bash
# Install Node.js (using nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Setup frontend
cd ../frontend
npm install

# Run frontend
npm run dev
```

### Method 3: Production

For production deployment with optimizations.

```bash
# Clone repository
git clone https://github.com/yourusername/cobalt-stack.git
cd cobalt-stack

# Copy production environment
cp .env.example .env.production
nano .env.production  # Edit production settings

# Build and start production services
docker-compose -f docker-compose.prod.yml up -d

# Verify services
docker-compose -f docker-compose.prod.yml ps
```

## Environment Configuration

### Required Environment Variables

Create `.env` file with these settings:

```bash
# Database
DATABASE_URL=postgres://user:password@localhost:5432/cobalt
POSTGRES_USER=cobalt
POSTGRES_PASSWORD=your_secure_password
POSTGRES_DB=cobalt

# Backend
BACKEND_PORT=8080
RUST_LOG=info
JWT_SECRET=your_jwt_secret_here_minimum_32_chars

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_APP_NAME=Cobalt Stack

# Traefik
TRAEFIK_DASHBOARD=true
DOMAIN=localhost
```

### Optional Environment Variables

```bash
# Email (if using email features)
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=noreply@example.com
SMTP_PASSWORD=smtp_password

# Storage (if using file uploads)
S3_BUCKET=cobalt-uploads
S3_REGION=us-east-1
S3_ACCESS_KEY=your_access_key
S3_SECRET_KEY=your_secret_key

# Monitoring
SENTRY_DSN=https://your-sentry-dsn
LOG_LEVEL=debug

# Performance
MAX_CONNECTIONS=100
WORKER_THREADS=4
```

### Environment File Security

```bash
# Protect environment files
chmod 600 .env
chmod 600 .env.production

# Add to .gitignore (already included)
echo ".env" >> .gitignore
echo ".env.production" >> .gitignore
```

## Development Setup

### 1. Database Setup

```bash
# Using Docker
docker-compose up -d postgres

# Or native PostgreSQL
createdb cobalt

# Run migrations
cd backend
cargo run --bin migrate

# Verify database
psql cobalt -c "\dt"
```

### 2. Backend Development

```bash
cd backend

# Install development tools
cargo install cargo-watch
cargo install cargo-tarpaulin

# Start with hot reload
cargo watch -x run

# Or standard run
cargo run

# Run tests
cargo test

# Generate documentation
cargo doc --no-deps --open
```

### 3. Frontend Development

```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev

# Run in different port
PORT=3001 npm run dev

# Run tests
npm test

# Build for production
npm run build
```

### 4. Database Migrations

```bash
# Create new migration
cd backend
cargo run --bin create-migration -- add_users_table

# Run migrations
cargo run --bin migrate

# Rollback last migration
cargo run --bin rollback

# Check migration status
cargo run --bin migration-status
```

## Production Setup

### 1. Security Configuration

```bash
# Generate secure secrets
openssl rand -base64 32  # For JWT_SECRET
openssl rand -base64 32  # For database password

# Update .env.production
nano .env.production
```

### 2. SSL/TLS Setup

```bash
# Using Let's Encrypt with Traefik
# Update docker-compose.prod.yml with your domain

# Or use custom certificates
mkdir -p certs
# Place your cert.pem and key.pem in certs/
```

### 3. Build Production Images

```bash
# Build all services
docker-compose -f docker-compose.prod.yml build

# Or build individually
docker build -t cobalt-backend -f backend/Dockerfile .
docker build -t cobalt-frontend -f frontend/Dockerfile .
```

### 4. Deploy

```bash
# Start production services
docker-compose -f docker-compose.prod.yml up -d

# Check health
curl http://localhost:8080/api/health
curl http://localhost:3000

# View logs
docker-compose -f docker-compose.prod.yml logs -f
```

## Verification

### Health Checks

```bash
# Backend health
curl http://localhost:8080/api/health

# Expected response:
# {"status":"ok","version":"0.1.0"}

# Frontend health
curl http://localhost:3000

# Database health
docker-compose exec postgres pg_isready
```

### Service Status

```bash
# Check all services
docker-compose ps

# Expected output:
# NAME                STATUS              PORTS
# backend             Up                  0.0.0.0:8080->8080/tcp
# frontend            Up                  0.0.0.0:3000->3000/tcp
# postgres            Up                  5432/tcp
# traefik             Up                  0.0.0.0:80->80/tcp
```

### API Testing

```bash
# Test API endpoint
curl http://localhost:8080/api/v1/health

# Create test user
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

## Troubleshooting

### Common Issues

#### Port Already in Use

```bash
# Find process
lsof -i :8080
lsof -i :3000

# Kill process
kill -9 <PID>

# Or change port in .env
BACKEND_PORT=8081
FRONTEND_PORT=3001
```

#### Database Connection Error

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Check connection
psql $DATABASE_URL -c "SELECT 1"

# Reset database
docker-compose down -v
docker-compose up -d postgres
```

#### Docker Build Fails

```bash
# Clean Docker cache
docker system prune -a

# Rebuild without cache
docker-compose build --no-cache

# Check disk space
df -h
```

#### Frontend Build Fails

```bash
# Clear npm cache
npm cache clean --force

# Remove node_modules
rm -rf node_modules .next

# Reinstall
npm install

# Check Node version
node --version  # Should be 18+
```

### Getting Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f postgres

# Last 100 lines
docker-compose logs --tail=100
```

### Reset Everything

```bash
# Stop all services
docker-compose down

# Remove volumes (⚠️ deletes database)
docker-compose down -v

# Remove images
docker-compose down --rmi all

# Start fresh
docker-compose up -d --build
```

## Next Steps

After successful installation:

1. **[Read the Quick Start Guide](./quick-start.md)** - 5-minute overview
2. **[Explore Project Structure](./project-structure.md)** - Understand the codebase
3. **[Check API Documentation](../api/README.md)** - Learn the API
4. **[Follow Development Guides](../guides/README.md)** - Build features

## Related Resources

- [Quick Start Guide](./quick-start.md)
- [Project Structure](./project-structure.md)
- [API Reference](../api/README.md)
- [Deployment Guide](../deployment/README.md)
- [Troubleshooting](../troubleshooting/README.md)

---

**Need help?** Open an issue on GitHub or check our [Troubleshooting Guide](../troubleshooting/README.md).
