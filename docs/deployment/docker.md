# Docker Deployment Guide

Comprehensive guide for deploying Cobalt Stack with Docker, including development setup, production configuration, and troubleshooting.

## Overview

Cobalt Stack uses Docker and Docker Compose for containerized deployment, supporting both development and production environments.

**Docker Images**:
- **Backend**: Rust application (multi-stage build, Debian Slim runtime)
- **Frontend**: Next.js application (Bun runtime)
- **PostgreSQL**: Official PostgreSQL 15 Alpine image
- **Redis**: Official Redis 7 Alpine image

## Prerequisites

### Required Software

- **Docker**: Version 20.10 or higher
- **Docker Compose**: Version 2.0 or higher (V2 with `docker compose` command)
- **Docker BuildKit**: Enabled by default in Docker 23.0+

**Installation Verification**:
```bash
docker --version
# Docker version 24.0.0 or higher

docker compose version
# Docker Compose version v2.20.0 or higher
```

### System Requirements

**Development**:
- CPU: 2 cores minimum
- RAM: 4GB minimum (8GB recommended)
- Disk: 10GB free space

**Production**:
- CPU: 4 cores minimum (8 cores recommended)
- RAM: 8GB minimum (16GB recommended)
- Disk: 50GB free space (with log rotation)

## Development Setup

### Quick Start

**1. Clone and Configure**:
```bash
git clone <repository-url> cobalt-stack
cd cobalt-stack

# Copy environment template
cp .env.example .env

# Edit environment variables (optional - defaults work for dev)
nano .env
```

**2. Start Services**:
```bash
# Start all services (backend, frontend, postgres, redis)
docker compose up

# Or run in detached mode (background)
docker compose up -d

# View logs
docker compose logs -f
```

**3. Access Application**:
- Frontend: http://localhost:2727
- Backend API: http://localhost:2750
- API Documentation: http://localhost:2750/swagger-ui
- PostgreSQL: localhost:2800
- Redis: localhost:2900

**4. Stop Services**:
```bash
# Stop containers (preserve volumes)
docker compose stop

# Stop and remove containers (preserve volumes)
docker compose down

# Stop, remove containers AND volumes (clean slate)
docker compose down -v
```

### Development Workflow

#### Rebuilding After Code Changes

**Backend Changes**:
```bash
# Rebuild backend image
docker compose build backend

# Restart backend service
docker compose up -d backend

# View backend logs
docker compose logs -f backend
```

**Frontend Changes**:
```bash
# Rebuild frontend image
docker compose build frontend

# Restart frontend service
docker compose up -d frontend

# View frontend logs
docker compose logs -f frontend
```

**Database Schema Changes**:
```bash
# Run migrations
docker compose exec backend /app/cobalt-stack-backend migrate up

# Or rebuild backend (migrations run on startup)
docker compose up -d --build backend
```

#### Live Development (Without Docker)

**Backend** (with hot reload):
```bash
# Start only dependencies (postgres, redis)
docker compose up postgres redis -d

# Run backend locally with cargo-watch
cd backend
cargo watch -x run
```

**Frontend** (with hot reload):
```bash
# Start backend and dependencies
docker compose up backend postgres redis -d

# Run frontend locally with Bun
cd frontend
bun run dev
```

### Environment Configuration

**Development .env** (root directory):
```bash
# Port Configuration
FRONTEND_PORT=2727
BACKEND_PORT=2750
POSTGRES_PORT=2800
REDIS_PORT=2900

# API URLs
NEXT_PUBLIC_API_URL=http://localhost:2750
FRONTEND_URL=http://localhost:2727

# Database
POSTGRES_DB=cobalt_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres

# Backend
RUST_LOG=debug
EMAIL_MOCK=true
EMAIL_VERIFICATION_EXPIRY_SECONDS=86400

# JWT (auto-generated if not set)
# JWT_SECRET=your-secret-key-min-32-chars
```

**Port Customization**:
- Change `FRONTEND_PORT`, `BACKEND_PORT`, etc. in `.env`
- No code changes required
- Restart containers: `docker compose up -d`

## Production Setup

### Production Configuration

**Production .env** (different from development):
```bash
# REQUIRED: Set these secrets
DATABASE_PASSWORD=<generate-strong-password>
REDIS_PASSWORD=<generate-strong-password>
JWT_SECRET=<generate-min-32-char-secret>

# Optional: Override defaults
DATABASE_NAME=cobalt_prod
DATABASE_USER=postgres
BACKEND_PORT=3000
IMAGE_TAG=v1.0.0
RUST_LOG=info

# Email Configuration (required for production)
EMAIL_MOCK=false
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USER=apikey
SMTP_PASSWORD=<sendgrid-api-key>
```

**Generate Secrets**:
```bash
# Database password (32 chars, alphanumeric + symbols)
openssl rand -base64 32

# Redis password (32 chars)
openssl rand -base64 32

# JWT secret (64 chars for extra security)
openssl rand -base64 64
```

### Production Deployment

**1. Build Images**:
```bash
# Build with BuildKit optimizations
DOCKER_BUILDKIT=1 docker compose -f docker-compose.prod.yml build

# Or use Makefile
make docker-build-prod
```

**2. Start Services**:
```bash
# Start all production services
docker compose -f docker-compose.prod.yml up -d

# Verify all services are healthy
docker compose -f docker-compose.prod.yml ps
```

**3. Run Database Migrations**:
```bash
# Migrations run automatically on backend startup
# Or manually trigger:
docker compose -f docker-compose.prod.yml exec backend /app/cobalt-stack-backend migrate
```

**4. Seed Admin User** (First Time):
```bash
# Create initial admin account
docker compose -f docker-compose.prod.yml exec backend /app/cobalt-stack-backend seed-admin

# Follow prompts to set email and password
```

**5. Health Check**:
```bash
# Backend health
curl http://localhost:3000/health

# Expected response:
# {"status":"healthy","timestamp":"2025-10-27T12:00:00Z"}

# Frontend health
curl http://localhost:3001/

# Expected: HTML response
```

### Production Compose File

**Key Differences from Development** (`docker-compose.prod.yml`):

1. **Security**:
   - Passwords required (not optional)
   - Databases NOT exposed to host (internal network only)
   - Resource limits enforced

2. **Reliability**:
   - `restart: always` policy
   - Health checks with retries
   - Longer startup periods

3. **Performance**:
   - Log rotation (prevent disk fill)
   - Resource reservations and limits
   - Connection pooling optimized

4. **Networking**:
   - Internal network (`cobalt-prod-network`)
   - No direct database access from host
   - Only backend/frontend exposed

## Docker Configuration

### Backend Dockerfile

**Multi-Stage Build** (`backend/Dockerfile`):

```dockerfile
# Stage 1: Builder
FROM rustlang/rust:nightly-bookworm-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY backend backend/

# Build with cache mounts (BuildKit)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p cobalt-stack-backend && \
    cp /app/target/release/cobalt-stack-backend /app/cobalt-stack-backend

# Stage 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary from builder
COPY --from=builder --chown=appuser:appuser /app/cobalt-stack-backend /app/cobalt-stack-backend

EXPOSE 3000
USER appuser
CMD ["/app/cobalt-stack-backend"]
```

**Build Optimization**:
- BuildKit cache mounts (share cache across builds)
- Multi-stage (builder: 2GB, runtime: 150MB)
- Minimal runtime dependencies
- Non-root user (security)

### Frontend Dockerfile

**Multi-Stage Build** (`frontend/Dockerfile`):

```dockerfile
# Stage 1: Dependencies
FROM oven/bun:1 AS deps
WORKDIR /app
COPY package.json bun.lock* ./
RUN bun install

# Stage 2: Builder
FROM oven/bun:1 AS builder
WORKDIR /app

ARG NEXT_PUBLIC_API_URL=http://localhost:2750
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
ENV NEXT_TELEMETRY_DISABLED=1

COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN bun run build

# Stage 3: Runtime
FROM oven/bun:1 AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

# Create non-root user
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Copy built application
COPY --from=builder /app/public ./public
RUN mkdir .next && chown nextjs:nodejs .next
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs
EXPOSE 3001

ENV PORT=3001
ENV HOSTNAME="0.0.0.0"

CMD ["bun", "server.js"]
```

**Build Args**:
- `NEXT_PUBLIC_API_URL`: Backend API URL (embedded in build)
- Pass via `--build-arg NEXT_PUBLIC_API_URL=https://api.example.com`

## Volume Management

### Persistent Volumes

**Development** (`docker-compose.yml`):
```yaml
volumes:
  postgres_data:
    driver: local
  redis_data:
    driver: local
```

**Production** (`docker-compose.prod.yml`):
```yaml
volumes:
  postgres_prod_data:
    driver: local
  redis_prod_data:
    driver: local
```

### Volume Operations

**List Volumes**:
```bash
docker volume ls
```

**Inspect Volume**:
```bash
docker volume inspect cobalt-stack_postgres_data
```

**Backup Database**:
```bash
# Backup PostgreSQL data
docker compose exec postgres pg_dump -U postgres cobalt_dev > backup.sql

# Or backup entire volume
docker run --rm -v cobalt-stack_postgres_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz -C /data .
```

**Restore Database**:
```bash
# Restore from SQL dump
docker compose exec -T postgres psql -U postgres cobalt_dev < backup.sql

# Or restore entire volume
docker run --rm -v cobalt-stack_postgres_data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/postgres-backup.tar.gz -C /data
```

**Remove Volumes** (⚠️ Data Loss):
```bash
# Remove all project volumes
docker compose down -v

# Remove specific volume
docker volume rm cobalt-stack_postgres_data
```

## Container Management

### Service Control

**Start/Stop Individual Services**:
```bash
# Start specific service
docker compose up -d postgres

# Stop specific service
docker compose stop backend

# Restart service
docker compose restart backend

# View service logs
docker compose logs -f backend

# View last 100 lines
docker compose logs --tail=100 backend
```

**Scale Services** (Not applicable for this stack - stateful services):
```bash
# Example for stateless services (future microservices)
docker compose up -d --scale api=3
```

### Container Inspection

**View Running Containers**:
```bash
docker compose ps
```

**Inspect Container**:
```bash
docker compose exec backend env
```

**Shell Access**:
```bash
# Backend (Debian)
docker compose exec backend /bin/bash

# PostgreSQL (Alpine)
docker compose exec postgres /bin/sh

# PostgreSQL client
docker compose exec postgres psql -U postgres -d cobalt_dev
```

**Resource Usage**:
```bash
# Real-time stats
docker stats

# Specific container
docker stats cobalt-backend-dev
```

## Networking

### Network Configuration

**Development Network**:
```yaml
networks:
  cobalt-network:
    driver: bridge
```

**Service Communication**:
- Services communicate via service names (DNS resolution)
- Example: Backend connects to `postgres:5432`, not `localhost:2800`

**Network Inspection**:
```bash
# List networks
docker network ls

# Inspect network
docker network inspect cobalt-stack_cobalt-network

# View connected containers
docker network inspect cobalt-stack_cobalt-network --format='{{range .Containers}}{{.Name}} {{end}}'
```

### Port Mapping

**Format**: `HOST:CONTAINER`

**Examples**:
```yaml
services:
  backend:
    ports:
      - "2750:3000"  # Host port 2750 → Container port 3000

  postgres:
    ports:
      - "${POSTGRES_PORT:-2800}:5432"  # Env var with default
```

**No Port Mapping** (Production):
```yaml
services:
  postgres:
    # No ports section - internal network only
    networks:
      - cobalt-prod-network
```

## Health Checks

### Configuration

**Backend Health Check**:
```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:3000/health || exit 1"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

**PostgreSQL Health Check**:
```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U postgres"]
  interval: 10s
  timeout: 5s
  retries: 5
```

**Redis Health Check**:
```yaml
healthcheck:
  test: ["CMD", "redis-cli", "ping"]
  interval: 10s
  timeout: 3s
  retries: 5
```

### Dependency Management

**Service Dependencies**:
```yaml
services:
  backend:
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
```

**Behavior**:
- Backend waits for postgres and redis to be healthy before starting
- If health check fails, service is marked unhealthy
- Dependent services won't start until dependencies are healthy

### Manual Health Checks

```bash
# Check all service health
docker compose ps

# Inspect specific service health
docker inspect --format='{{.State.Health.Status}}' cobalt-backend-dev

# View health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' cobalt-backend-dev
```

## Logging

### Log Configuration

**Development** (verbose, no rotation):
```yaml
# Logs to stdout (visible with docker compose logs)
# No rotation needed (short-lived dev sessions)
```

**Production** (json-file driver with rotation):
```yaml
logging:
  driver: "json-file"
  options:
    max-size: "20m"      # Max 20MB per log file
    max-file: "5"        # Keep 5 rotated files
    # Total max: 100MB per service
```

### Viewing Logs

**All Services**:
```bash
docker compose logs

# Follow (tail -f)
docker compose logs -f

# Last 50 lines
docker compose logs --tail=50

# Since timestamp
docker compose logs --since 2024-10-27T10:00:00
```

**Specific Service**:
```bash
docker compose logs backend
docker compose logs -f postgres
```

**Production Logs**:
```bash
# JSON-formatted logs
docker compose -f docker-compose.prod.yml logs backend --tail=100

# Parse JSON logs
docker compose -f docker-compose.prod.yml logs backend --tail=100 | jq -r '.message'
```

### Log Aggregation (Future)

**Options**:
- **ELK Stack** (Elasticsearch, Logstash, Kibana)
- **Loki + Grafana**
- **Fluentd + Cloudwatch**

**Configuration Example** (Fluentd):
```yaml
logging:
  driver: "fluentd"
  options:
    fluentd-address: localhost:24224
    tag: cobalt.backend
```

## Troubleshooting

### Common Issues

#### 1. Port Already in Use

**Symptoms**:
```
Error: bind: address already in use
```

**Solution**:
```bash
# Find process using port
lsof -i :2750
# Or on Linux
netstat -tuln | grep 2750

# Kill process or change port in .env
BACKEND_PORT=2751
docker compose up -d
```

#### 2. Database Connection Failed

**Symptoms**:
```
Failed to connect to database: Connection refused
```

**Solution**:
```bash
# Check postgres is running and healthy
docker compose ps postgres

# Check logs
docker compose logs postgres

# Verify DATABASE_URL in backend
docker compose exec backend env | grep DATABASE_URL

# Restart postgres
docker compose restart postgres
```

#### 3. Out of Disk Space

**Symptoms**:
```
Error: no space left on device
```

**Solution**:
```bash
# Check disk usage
df -h

# Clean Docker resources
docker system prune -a

# Remove old volumes (⚠️ data loss)
docker volume prune

# Check volume sizes
docker system df -v
```

#### 4. Build Cache Issues

**Symptoms**:
```
Dependency tree resolution errors
Old code still running after rebuild
```

**Solution**:
```bash
# Rebuild without cache
docker compose build --no-cache

# Remove all caches
docker builder prune -a

# Full clean rebuild
docker compose down -v
docker compose build --no-cache
docker compose up -d
```

#### 5. Permission Denied

**Symptoms**:
```
Permission denied: '/var/lib/postgresql/data'
```

**Solution**:
```bash
# Check volume ownership
docker compose exec postgres ls -la /var/lib/postgresql

# Recreate volume with correct permissions
docker compose down -v
docker compose up -d
```

### Debug Mode

**Enable Verbose Logging**:
```bash
# Development
RUST_LOG=trace docker compose up

# Production
docker compose -f docker-compose.prod.yml exec backend env RUST_LOG=debug /app/cobalt-stack-backend
```

**Shell Access for Debugging**:
```bash
# Backend
docker compose exec backend /bin/bash

# Check environment
env | grep -E "(DATABASE|REDIS|JWT)"

# Test database connection
curl http://localhost:3000/health

# Check file permissions
ls -la /app
```

### Performance Monitoring

**Container Stats**:
```bash
# Real-time resource usage
docker stats

# Export to file
docker stats --no-stream > stats.txt
```

**Health Check Status**:
```bash
# All services
docker compose ps --format "table {{.Name}}\t{{.Status}}"

# Unhealthy services only
docker ps --filter health=unhealthy
```

## Best Practices

### Development

1. **Use docker compose for dependencies only**: Run backend/frontend locally for hot reload
2. **Commit .env.example, not .env**: Keep secrets out of version control
3. **Rebuild after dependency changes**: `docker compose build` when Cargo.toml or package.json changes
4. **Clean volumes periodically**: Avoid stale data with `docker compose down -v`

### Production

1. **Set strong secrets**: Use `openssl rand -base64 32` for all passwords
2. **Enable log rotation**: Prevent disk space issues
3. **Monitor health checks**: Set up alerts for unhealthy services
4. **Backup volumes regularly**: Automate database backups
5. **Limit resource usage**: Set CPU/memory limits in compose file
6. **Use specific image tags**: Avoid `latest`, use version tags (e.g., `v1.0.0`)
7. **Enable HTTPS**: Use reverse proxy (Traefik, nginx) for TLS termination

### Security

1. **Don't expose databases**: Remove port mappings in production
2. **Use non-root users**: Containers run as unprivileged users
3. **Scan images for vulnerabilities**: Use `docker scan` or Trivy
4. **Keep images updated**: Regularly rebuild with latest base images
5. **Use secrets management**: Docker secrets or external vault (HashiCorp Vault)

## References

- [Docker Documentation](https://docs.docker.com)
- [Docker Compose Documentation](https://docs.docker.com/compose)
- [Docker BuildKit](https://docs.docker.com/build/buildkit)
- [Docker Security Best Practices](https://docs.docker.com/engine/security)
