# Docker Compose Chat Feature Setup

## Quick Start with Docker Compose

The easiest way to run the chat feature is with Docker Compose, which automatically sets up PostgreSQL, Redis, backend, and frontend.

### Prerequisites

- Docker installed and running
- Docker Compose V2 installed
- Your SambaNova API key (already configured)

### Step 1: Verify Configuration

Your `.env` file at the project root is already configured with:

```bash
# Already set in .env
FEATURE_CHAT_ENABLED=true
SAMBANOVA_API_KEY=33938d57-5ac4-47f2-b663-c41d5db55d8a
CHAT_RATE_LIMIT_PER_MINUTE=20
CHAT_DAILY_MESSAGE_QUOTA=100
```

### Step 2: Start All Services

```bash
# Start all services (PostgreSQL, Redis, Backend, Frontend)
docker compose up -d

# Expected output:
# ✔ Container cobalt-postgres-dev    Started
# ✔ Container cobalt-redis-dev       Started
# ✔ Container cobalt-backend-dev     Started
# ✔ Container cobalt-frontend-dev    Started
```

### Step 3: Run Database Migrations

```bash
# Run migrations inside the backend container
docker compose exec backend sea-orm-cli migrate up

# Expected output:
# Applying migration 'm20250127_000001_create_chat_tables'
# Migration applied successfully
```

### Step 4: Verify Services

```bash
# Check all containers are running
docker compose ps

# Expected output:
# NAME                     STATUS        PORTS
# cobalt-backend-dev       Up (healthy)  0.0.0.0:2750->3000/tcp
# cobalt-frontend-dev      Up (healthy)  0.0.0.0:2727->3001/tcp
# cobalt-postgres-dev      Up (healthy)  0.0.0.0:2800->5432/tcp
# cobalt-redis-dev         Up (healthy)  0.0.0.0:2900->6379/tcp
```

### Step 5: Access the Application

- **Frontend**: http://localhost:2727
- **Chat Feature**: http://localhost:2727/chat
- **Backend API**: http://localhost:2750
- **API Docs**: http://localhost:2750/swagger-ui
- **Traefik Dashboard**: http://localhost:8080

### Step 6: Test Chat Feature

1. **Register/Login**: Create an account or login at http://localhost:2727
2. **Navigate to Chat**: Go to http://localhost:2727/chat
3. **Create Session**: Click "New Chat"
4. **Send Message**: Type a message and press Enter
5. **See Streaming**: Watch the LLM response stream in real-time

## Configuration Details

### Environment Variables

All chat-related environment variables are automatically passed from `.env` to the backend container:

```yaml
# From docker-compose.yml backend service
environment:
  FEATURE_CHAT_ENABLED: ${FEATURE_CHAT_ENABLED:-false}
  SAMBANOVA_API_KEY: ${SAMBANOVA_API_KEY}
  SAMBANOVA_API_BASE: ${SAMBANOVA_API_BASE:-https://api.sambanova.ai/v1}
  SAMBANOVA_MODEL: ${SAMBANOVA_MODEL:-Llama-4-Maverick-17B-128E-Instruct}
  CHAT_MAX_CONTEXT_MESSAGES: ${CHAT_MAX_CONTEXT_MESSAGES:-20}
  CHAT_MAX_TOKENS: ${CHAT_MAX_TOKENS:-2048}
  CHAT_MAX_MESSAGE_LENGTH: ${CHAT_MAX_MESSAGE_LENGTH:-4000}
  CHAT_DAILY_MESSAGE_QUOTA: ${CHAT_DAILY_MESSAGE_QUOTA:-100}
  CHAT_RATE_LIMIT_PER_MINUTE: ${CHAT_RATE_LIMIT_PER_MINUTE:-20}
  VALKEY_URL: redis://redis:6379
```

### Port Mapping

Your custom ports are configured in `.env`:

```bash
FRONTEND_PORT=2727   # Frontend accessible at localhost:2727
BACKEND_PORT=2750    # Backend API at localhost:2750
POSTGRES_PORT=2800   # PostgreSQL at localhost:2800
REDIS_PORT=2900      # Redis at localhost:2900
```

### Internal Service URLs

Docker Compose creates an internal network where services communicate using service names:

- Backend connects to database: `postgresql://postgres:postgres@postgres:5432/cobalt_dev`
- Backend connects to Redis: `redis://redis:6379`
- Frontend connects to backend: Via `NEXT_PUBLIC_API_URL` (external URL)

## Common Operations

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
docker compose logs -f frontend

# Last 100 lines
docker compose logs --tail=100 backend
```

### Restart Services

```bash
# Restart all
docker compose restart

# Restart specific service
docker compose restart backend
docker compose restart frontend
```

### Rebuild After Code Changes

```bash
# Rebuild and restart
docker compose up -d --build

# Rebuild specific service
docker compose up -d --build backend
docker compose up -d --build frontend
```

### Stop Services

```bash
# Stop all (keeps data)
docker compose stop

# Stop and remove containers (keeps data)
docker compose down

# Stop and remove everything including volumes (DELETES DATA!)
docker compose down -v
```

### Access Container Shell

```bash
# Backend container
docker compose exec backend /bin/sh

# Frontend container
docker compose exec frontend /bin/sh

# PostgreSQL
docker compose exec postgres psql -U postgres -d cobalt_dev
```

## Testing Chat Feature

### Test Rate Limiting

```bash
# Get access token (login first via frontend or API)
TOKEN="your-jwt-token-here"

# Send 21 messages rapidly to test per-minute limit
for i in {1..21}; do
  echo "Sending message $i"
  curl -X POST http://localhost:2750/api/v1/chat/sessions/<SESSION_ID>/messages \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"content\": \"Test message $i\"}" \
    --no-buffer
  echo ""
done

# After 20 messages, should receive 429 error
```

### Monitor Rate Limits in Redis

```bash
# Connect to Redis container
docker compose exec redis redis-cli

# List all chat rate limit keys
KEYS ratelimit:chat:*

# Check specific user's per-minute counter
GET ratelimit:chat:user:<USER_UUID>:minute

# Check daily quota
GET quota:chat:user:<USER_UUID>:daily

# Check TTL (time to live)
TTL ratelimit:chat:user:<USER_UUID>:minute
TTL quota:chat:user:<USER_UUID>:daily
```

### Database Queries

```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U postgres -d cobalt_dev

# Check chat tables
\dt chat*

# Count sessions
SELECT COUNT(*) FROM chat_sessions;

# Count messages
SELECT COUNT(*) FROM chat_messages;

# View recent messages
SELECT
  cs.title,
  cm.role,
  LEFT(cm.content, 50) as content_preview,
  cm.created_at
FROM chat_messages cm
JOIN chat_sessions cs ON cs.id = cm.session_id
ORDER BY cm.created_at DESC
LIMIT 10;
```

## Troubleshooting

### Backend Not Starting

**Check logs:**
```bash
docker compose logs backend
```

**Common issues:**

1. **Database not ready**
   ```
   Error: Connection refused (postgres)
   ```
   Solution: Wait for postgres to be healthy (check with `docker compose ps`)

2. **Redis not ready**
   ```
   Error: Connection refused (redis)
   ```
   Solution: Wait for redis to be healthy

3. **Missing environment variable**
   ```
   Error: SAMBANOVA_API_KEY not set
   ```
   Solution: Check `.env` file has `SAMBANOVA_API_KEY` set

### Chat Feature Not Working

1. **Check if feature is enabled:**
   ```bash
   docker compose exec backend env | grep FEATURE_CHAT
   # Should show: FEATURE_CHAT_ENABLED=true
   ```

2. **Check SambaNova API key:**
   ```bash
   docker compose exec backend env | grep SAMBANOVA_API_KEY
   # Should show your API key
   ```

3. **Check migrations ran:**
   ```bash
   docker compose exec postgres psql -U postgres -d cobalt_dev -c "\dt chat*"
   # Should show: chat_sessions, chat_messages
   ```

4. **Check backend logs for errors:**
   ```bash
   docker compose logs backend | grep -i "chat\|error"
   ```

### Rate Limiting Not Working

1. **Verify Redis connection:**
   ```bash
   docker compose exec redis redis-cli ping
   # Should return: PONG
   ```

2. **Check VALKEY_URL:**
   ```bash
   docker compose exec backend env | grep VALKEY_URL
   # Should show: VALKEY_URL=redis://redis:6379
   ```

3. **Monitor Redis keys:**
   ```bash
   docker compose exec redis redis-cli MONITOR
   # Should show SET/GET operations when sending messages
   ```

### Frontend Not Connecting to Backend

1. **Check NEXT_PUBLIC_API_URL:**
   ```bash
   docker compose exec frontend env | grep NEXT_PUBLIC_API_URL
   ```

2. **Verify backend is accessible:**
   ```bash
   curl http://localhost:2750/health
   # Should return: 200 OK
   ```

3. **Check CORS configuration:**
   ```bash
   docker compose exec backend env | grep CORS_ORIGINS
   # Should include frontend URL
   ```

## Production Deployment

For production, use `docker-compose.prod.yml`:

### Step 1: Create Production .env

```bash
# Copy and modify for production
cp .env .env.production

# Edit .env.production with production values:
# - Strong JWT_SECRET
# - Production DATABASE_PASSWORD
# - Production REDIS_PASSWORD
# - Production domains
# - EMAIL_MOCK=false with real SMTP
```

### Step 2: Deploy

```bash
# Use production compose file
docker compose -f docker-compose.prod.yml up -d

# Run migrations
docker compose -f docker-compose.prod.yml exec backend sea-orm-cli migrate up
```

### Step 3: Monitor

```bash
# Check logs
docker compose -f docker-compose.prod.yml logs -f

# Check resource usage
docker stats
```

## Performance Optimization

### Resource Limits

The production compose file includes resource limits:

```yaml
deploy:
  resources:
    limits:
      cpus: '2.0'
      memory: 2G
    reservations:
      cpus: '0.5'
      memory: 512M
```

### Health Checks

All services have health checks configured:

- **PostgreSQL**: `pg_isready` every 10s
- **Redis**: `redis-cli ping` every 10s
- **Backend**: HTTP health endpoint every 30s
- **Frontend**: HTTP root endpoint every 30s

### Logging

Production logging is configured with rotation:

```yaml
logging:
  driver: "json-file"
  options:
    max-size: "20m"
    max-file: "5"
```

## Backup & Restore

### Backup Database

```bash
# Create backup
docker compose exec postgres pg_dump -U postgres cobalt_dev > backup.sql

# Or with timestamp
docker compose exec postgres pg_dump -U postgres cobalt_dev > backup_$(date +%Y%m%d_%H%M%S).sql
```

### Restore Database

```bash
# Restore from backup
docker compose exec -T postgres psql -U postgres cobalt_dev < backup.sql
```

### Backup Redis Data

```bash
# Redis automatically persists to /data in container
# Copy Redis data directory
docker compose exec redis redis-cli SAVE
docker compose cp redis:/data/dump.rdb ./redis_backup.rdb
```

## Monitoring

### Health Check Endpoints

```bash
# Backend health
curl http://localhost:2750/health

# PostgreSQL
docker compose exec postgres pg_isready -U postgres

# Redis
docker compose exec redis redis-cli ping
```

### Resource Usage

```bash
# Container stats
docker stats

# Disk usage
docker system df

# Volume usage
docker volume ls
```

### Log Analysis

```bash
# Error count
docker compose logs backend | grep -i error | wc -l

# Rate limit violations
docker compose logs backend | grep "429" | wc -l

# Recent chat requests
docker compose logs backend | grep "chat/sessions" | tail -20
```

## Summary

✅ **Docker Compose is configured** with all chat feature variables
✅ **Your .env has** `FEATURE_CHAT_ENABLED=true` and your API key
✅ **Both dev and prod** compose files are updated
✅ **Redis/Valkey** configured for rate limiting

**To start the chat feature:**

```bash
# One command to rule them all
docker compose up -d

# Run migrations
docker compose exec backend sea-orm-cli migrate up

# Access chat
open http://localhost:2727/chat
```

That's it! Your chat feature is ready to go with Docker Compose! 🚀
