# Production Deployment Guide

Comprehensive guide for deploying Cobalt Stack to production environments with security hardening, performance optimization, and operational best practices.

## Pre-Deployment Checklist

### Infrastructure Requirements

**Minimum Specifications**:
- **CPU**: 4 cores (2 cores for backend, 2 for database/services)
- **RAM**: 8GB (4GB backend, 2GB database, 1GB Redis, 1GB frontend)
- **Disk**: 50GB SSD (with log rotation and database growth)
- **Network**: 100 Mbps minimum, 1 Gbps recommended
- **OS**: Ubuntu 22.04 LTS or Debian 12 (recommended)

**Recommended Specifications** (1000+ concurrent users):
- **CPU**: 8 cores
- **RAM**: 16GB
- **Disk**: 200GB SSD
- **Network**: 1 Gbps with redundancy
- **Load Balancer**: Nginx or Traefik with multiple backend instances

### Required Services

- [x] Docker 24.0+ and Docker Compose V2
- [x] Reverse proxy (Nginx, Traefik, Caddy)
- [x] SSL/TLS certificates (Let's Encrypt recommended)
- [x] Domain name with DNS configured
- [x] Email service (SendGrid, AWS SES, Mailgun)
- [x] Monitoring solution (Prometheus + Grafana, DataDog)
- [x] Backup solution (automated daily backups)

### Security Checklist

- [x] Firewall configured (UFW, iptables)
- [x] SSH key authentication enabled (password auth disabled)
- [x] Fail2ban installed and configured
- [x] System packages updated (`apt update && apt upgrade`)
- [x] Non-root user created for deployments
- [x] Secret management strategy (environment files, vault)
- [x] SSL/TLS certificates issued and configured
- [x] Security headers configured in reverse proxy
- [x] Rate limiting configured
- [x] Database backups automated

## Environment Setup

### System Preparation

**1. Update System**:
```bash
sudo apt update
sudo apt upgrade -y
sudo apt install -y curl git ufw fail2ban
```

**2. Install Docker**:
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group (avoid sudo)
sudo usermod -aG docker $USER
newgrp docker

# Verify installation
docker --version
docker compose version
```

**3. Configure Firewall**:
```bash
# Enable UFW
sudo ufw enable

# Allow SSH (change port if non-standard)
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS (reverse proxy)
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Deny direct access to application ports
# (traffic should go through reverse proxy)
sudo ufw deny 3000/tcp
sudo ufw deny 3001/tcp

# Check status
sudo ufw status verbose
```

**4. Configure Fail2ban**:
```bash
sudo systemctl enable fail2ban
sudo systemctl start fail2ban

# Check status
sudo fail2ban-client status
```

### Application Setup

**1. Clone Repository**:
```bash
cd /opt
sudo git clone <repository-url> cobalt-stack
sudo chown -R $USER:$USER cobalt-stack
cd cobalt-stack
```

**2. Generate Secrets**:
```bash
# Database password (32 characters)
DB_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/")
echo "DATABASE_PASSWORD=$DB_PASSWORD"

# Redis password (32 characters)
REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/")
echo "REDIS_PASSWORD=$REDIS_PASSWORD"

# JWT secret (64 characters for extra security)
JWT_SECRET=$(openssl rand -base64 64 | tr -d "=+/")
echo "JWT_SECRET=$JWT_SECRET"
```

**3. Create Production Environment File**:
```bash
# Create .env.production
cat > .env.production <<EOF
# REQUIRED SECRETS (generated above)
DATABASE_PASSWORD=$DB_PASSWORD
REDIS_PASSWORD=$REDIS_PASSWORD
JWT_SECRET=$JWT_SECRET

# Database Configuration
DATABASE_NAME=cobalt_prod
DATABASE_USER=postgres

# Application Configuration
BACKEND_PORT=3000
IMAGE_TAG=v1.0.0
RUST_LOG=info

# Email Configuration (REQUIRED for production)
EMAIL_MOCK=false
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USER=apikey
SMTP_PASSWORD=<your-sendgrid-api-key>
EMAIL_VERIFICATION_EXPIRY_SECONDS=86400

# Frontend Configuration
NEXT_PUBLIC_API_URL=https://api.yourdomain.com
FRONTEND_URL=https://yourdomain.com
EOF

# Secure the file (readable only by owner)
chmod 600 .env.production
```

**4. Build Images**:
```bash
# Build with production optimizations
DOCKER_BUILDKIT=1 docker compose -f docker-compose.prod.yml build

# Tag images with version
docker tag cobalt-stack-backend:latest cobalt-stack-backend:v1.0.0
```

**5. Start Services**:
```bash
# Load environment variables
set -a
source .env.production
set +a

# Start all services
docker compose -f docker-compose.prod.yml up -d

# Verify all services are running
docker compose -f docker-compose.prod.yml ps

# Check logs
docker compose -f docker-compose.prod.yml logs -f
```

**6. Initialize Database**:
```bash
# Migrations run automatically on backend startup
# Verify migrations completed:
docker compose -f docker-compose.prod.yml logs backend | grep "migration"

# Create admin user
docker compose -f docker-compose.prod.yml exec backend /app/cobalt-stack-backend seed-admin

# Follow prompts to set admin email and password
```

**7. Verify Deployment**:
```bash
# Health check
curl http://localhost:3000/health

# Expected response:
# {"status":"healthy","timestamp":"2025-10-27T12:00:00Z"}

# Test API
curl http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"TestPass123!"}'
```

## Security Hardening

### Secrets Management

**Environment File Security**:
```bash
# Restrict access to environment files
chmod 600 .env.production

# Audit access
ls -la .env.production

# Store backup securely (encrypted)
gpg -c .env.production  # Creates .env.production.gpg
rm .env.production       # Remove plaintext
```

**Using Docker Secrets** (Swarm or Kubernetes):
```bash
# Create secrets
echo "$DATABASE_PASSWORD" | docker secret create db_password -
echo "$JWT_SECRET" | docker secret create jwt_secret -

# Reference in compose file:
secrets:
  db_password:
    external: true
  jwt_secret:
    external: true

services:
  backend:
    secrets:
      - db_password
      - jwt_secret
```

**External Secrets Management**:
- **HashiCorp Vault**: Centralized secret storage
- **AWS Secrets Manager**: Cloud-based secrets
- **Azure Key Vault**: Azure secrets management
- **GCP Secret Manager**: Google Cloud secrets

### Network Security

**Docker Network Isolation**:
```yaml
# Production network configuration
networks:
  cobalt-prod-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

**Database Access Restriction**:
```yaml
# NO port mapping in production
services:
  postgres:
    # ports: # REMOVE THIS
    networks:
      - cobalt-prod-network  # Internal network only
```

**Rate Limiting** (Application Level - Future):
```rust
// Backend rate limiting
use tower::limit::RateLimitLayer;

let app = Router::new()
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)));
```

### Application Security

**Security Headers** (via reverse proxy):
```nginx
# Nginx configuration
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" always;
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
```

**CORS Configuration**:
```rust
// Production CORS (backend/src/main.rs)
let cors = CorsLayer::new()
    .allow_origin(vec![
        "https://yourdomain.com".parse().unwrap(),
    ])
    .allow_methods([GET, POST, PUT, DELETE, OPTIONS])
    .allow_credentials(true);
```

**Cookie Security**:
```rust
// Secure, HttpOnly, SameSite cookies
Cookie::build("refresh_token", token)
    .http_only(true)
    .secure(true)  // HTTPS only in production
    .same_site(SameSite::Lax)
    .path("/")
    .max_age(Duration::days(7))
    .finish()
```

## Reverse Proxy Configuration

### Nginx

**Installation**:
```bash
sudo apt install -y nginx certbot python3-certbot-nginx
```

**Configuration** (`/etc/nginx/sites-available/cobalt-stack`):
```nginx
# Redirect HTTP to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name yourdomain.com;

    # SSL certificates (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Frontend (Next.js)
    location / {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Backend API
    location /api/ {
        proxy_pass http://localhost:3000/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support (future)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Health check endpoint
    location /health {
        proxy_pass http://localhost:3000/health;
        access_log off;
    }

    # API documentation
    location /swagger-ui {
        proxy_pass http://localhost:3000/swagger-ui;
    }

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req zone=api_limit burst=20 nodelay;

    # Client body size limit
    client_max_body_size 10M;
}
```

**Enable Configuration**:
```bash
# Test configuration
sudo nginx -t

# Enable site
sudo ln -s /etc/nginx/sites-available/cobalt-stack /etc/nginx/sites-enabled/

# Reload Nginx
sudo systemctl reload nginx
```

**SSL Certificate** (Let's Encrypt):
```bash
# Obtain certificate
sudo certbot --nginx -d yourdomain.com -d www.yourdomain.com

# Verify auto-renewal
sudo certbot renew --dry-run

# Auto-renewal is configured via systemd timer
sudo systemctl status certbot.timer
```

### Traefik

**Docker Compose Integration**:
```yaml
version: "3.8"

services:
  traefik:
    image: traefik:v2.10
    command:
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@yourdomain.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
      - "./letsencrypt:/letsencrypt"

  backend:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.backend.rule=Host(`yourdomain.com`) && PathPrefix(`/api`)"
      - "traefik.http.routers.backend.entrypoints=websecure"
      - "traefik.http.routers.backend.tls.certresolver=letsencrypt"
      - "traefik.http.services.backend.loadbalancer.server.port=3000"

  frontend:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.frontend.rule=Host(`yourdomain.com`)"
      - "traefik.http.routers.frontend.entrypoints=websecure"
      - "traefik.http.routers.frontend.tls.certresolver=letsencrypt"
      - "traefik.http.services.frontend.loadbalancer.server.port=3001"
```

## Performance Optimization

### Resource Limits

**Docker Compose Configuration**:
```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '2.0'      # Max 2 CPU cores
          memory: 2G       # Max 2GB RAM
        reservations:
          cpus: '0.5'      # Reserve 0.5 CPU
          memory: 512M     # Reserve 512MB RAM

  postgres:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 1G
```

### Database Optimization

**PostgreSQL Configuration** (`postgresql.conf`):
```conf
# Memory
shared_buffers = 1GB                # 25% of system RAM
effective_cache_size = 3GB          # 75% of system RAM
maintenance_work_mem = 256MB
work_mem = 16MB

# Write-ahead log
wal_buffers = 16MB
checkpoint_completion_target = 0.9
max_wal_size = 2GB

# Query planner
random_page_cost = 1.1              # SSD
effective_io_concurrency = 200      # SSD

# Connections
max_connections = 100
```

**Apply Custom Configuration**:
```yaml
services:
  postgres:
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./postgresql.conf:/etc/postgresql/postgresql.conf:ro
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
```

**Database Indexing**:
```sql
-- Create indexes on frequently queried columns
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_email_verifications_user_id ON email_verifications(user_id);
```

### Redis Optimization

**Redis Configuration**:
```conf
# Memory management
maxmemory 512mb
maxmemory-policy allkeys-lru

# Persistence (optional - restart loses data)
save ""                              # Disable RDB snapshots
appendonly no                        # Disable AOF

# Performance
tcp-backlog 511
timeout 300
```

**Apply Configuration**:
```yaml
services:
  redis:
    command: redis-server --maxmemory 512mb --maxmemory-policy allkeys-lru
```

### Backend Optimization

**Rust Compiler Optimizations** (`Cargo.toml`):
```toml
[profile.release]
opt-level = 3              # Maximum optimization
lto = "fat"                # Link-time optimization
codegen-units = 1          # Single codegen unit for better optimization
strip = true               # Strip debug symbols
panic = "abort"            # Smaller binary size
```

**Connection Pooling** (SeaORM):
```rust
let db = Database::connect(&database_url)
    .await?
    .set_max_connections(20)
    .set_min_connections(5)
    .set_connect_timeout(Duration::from_secs(30))
    .set_idle_timeout(Duration::from_secs(600));
```

### Frontend Optimization

**Next.js Configuration** (`next.config.js`):
```javascript
module.exports = {
  output: "standalone",               // Minimal production output
  compress: true,                     // Gzip compression
  poweredByHeader: false,             // Remove X-Powered-By header
  reactStrictMode: true,
  swcMinify: true,                    // Fast minification

  // Image optimization
  images: {
    domains: ["yourdomain.com"],
    formats: ["image/avif", "image/webp"],
  },

  // Performance
  experimental: {
    optimizeCss: true,
  },
};
```

## Scaling Strategies

### Horizontal Scaling

**Multiple Backend Instances**:
```yaml
services:
  backend:
    deploy:
      replicas: 3              # 3 backend instances
    labels:
      - "traefik.enable=true"
      # Traefik load balances automatically
```

**Load Balancer Configuration** (Nginx):
```nginx
upstream backend {
    least_conn;                       # Load balancing method
    server localhost:3001;
    server localhost:3002;
    server localhost:3003;
}

server {
    location /api/ {
        proxy_pass http://backend/api/;
    }
}
```

### Database Replication

**Read Replicas** (PostgreSQL):
```yaml
services:
  postgres-primary:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: replicator
      POSTGRES_PASSWORD: replicator_password

  postgres-replica:
    image: postgres:15-alpine
    environment:
      PGUSER: replicator
      PGPASSWORD: replicator_password
```

**Application Configuration**:
```rust
// Read from replica, write to primary
let write_db = Database::connect(&primary_url).await?;
let read_db = Database::connect(&replica_url).await?;
```

### Caching Strategies

**Application-Level Caching**:
```rust
// Cache frequently accessed data in Redis
let user = redis.get::<User>(&user_id).await?;
if user.is_none() {
    let user = db.find_user(&user_id).await?;
    redis.set(&user_id, &user, Duration::from_secs(3600)).await?;
}
```

**HTTP Caching** (Nginx):
```nginx
# Cache static assets
location ~* \.(jpg|jpeg|png|gif|ico|css|js)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# Cache API responses (with validation)
location /api/ {
    proxy_cache api_cache;
    proxy_cache_valid 200 5m;
    proxy_cache_key "$request_uri";
}
```

## Backup & Recovery

### Database Backups

**Automated Backups** (Cron Job):
```bash
#!/bin/bash
# /opt/cobalt-stack/scripts/backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/opt/backups/cobalt"
mkdir -p $BACKUP_DIR

# Backup PostgreSQL
docker compose -f /opt/cobalt-stack/docker-compose.prod.yml exec -T postgres \
  pg_dump -U postgres cobalt_prod | gzip > $BACKUP_DIR/db_$DATE.sql.gz

# Backup Redis (optional - stateless)
docker compose -f /opt/cobalt-stack/docker-compose.prod.yml exec -T redis \
  redis-cli --rdb /data/dump.rdb
docker cp cobalt-redis-prod:/data/dump.rdb $BACKUP_DIR/redis_$DATE.rdb

# Encrypt backup
gpg -c $BACKUP_DIR/db_$DATE.sql.gz

# Delete old backups (keep 30 days)
find $BACKUP_DIR -name "db_*.sql.gz" -mtime +30 -delete

# Upload to S3 (optional)
# aws s3 cp $BACKUP_DIR/db_$DATE.sql.gz.gpg s3://your-bucket/backups/
```

**Schedule Backup**:
```bash
# Add to crontab
sudo crontab -e

# Daily backup at 2 AM
0 2 * * * /opt/cobalt-stack/scripts/backup.sh
```

### Restore Procedure

**Restore from Backup**:
```bash
# Stop services
docker compose -f docker-compose.prod.yml down

# Restore database
gunzip < /opt/backups/cobalt/db_20251027_020000.sql.gz | \
  docker compose -f docker-compose.prod.yml exec -T postgres psql -U postgres cobalt_prod

# Restart services
docker compose -f docker-compose.prod.yml up -d
```

## Monitoring & Observability

### Health Monitoring

**Uptime Monitoring**:
- **UptimeRobot**: Free external monitoring
- **Pingdom**: Professional monitoring
- **Custom**: Simple script with curl + cron

**Custom Health Check Script**:
```bash
#!/bin/bash
# /opt/cobalt-stack/scripts/health-check.sh

ENDPOINT="https://yourdomain.com/health"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $ENDPOINT)

if [ $RESPONSE -ne 200 ]; then
    echo "Health check failed: HTTP $RESPONSE"
    # Send alert (email, Slack, PagerDuty)
    curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
      -d '{"text":"Cobalt Stack health check failed!"}'
fi
```

### Log Aggregation

**Syslog Driver** (Docker):
```yaml
services:
  backend:
    logging:
      driver: syslog
      options:
        syslog-address: "tcp://loghost:514"
        tag: "cobalt-backend"
```

**Centralized Logging** (Loki):
```yaml
services:
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"

  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
```

### Metrics Collection

**Prometheus Integration** (Future):
```rust
// Backend metrics endpoint
use prometheus::{Encoder, TextEncoder, Counter, Registry};

let requests = Counter::new("http_requests_total", "Total HTTP requests").unwrap();

// Expose /metrics endpoint
async fn metrics() -> Response {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Response::new(buffer.into())
}
```

## Troubleshooting

### Common Production Issues

**1. High Memory Usage**:
```bash
# Check memory usage
docker stats

# Increase limits in docker-compose.prod.yml
# Optimize queries (avoid SELECT * FROM large_table)
# Enable query result pagination
```

**2. Slow Database Queries**:
```bash
# Enable slow query log (PostgreSQL)
docker compose exec postgres psql -U postgres -c "ALTER SYSTEM SET log_min_duration_statement = 1000;"

# Analyze slow queries
docker compose exec postgres psql -U postgres -c "SELECT query, calls, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

# Add indexes
CREATE INDEX CONCURRENTLY idx_table_column ON table(column);
```

**3. Connection Pool Exhausted**:
```bash
# Check active connections
docker compose exec postgres psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"

# Increase max_connections in PostgreSQL
# Optimize connection pool size in backend
```

**4. Disk Space Issues**:
```bash
# Check disk usage
df -h

# Clean Docker resources
docker system prune -a
docker volume prune

# Rotate logs more aggressively
# Compress old database backups
```

## Deployment Automation

### CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    tags:
      - "v*"

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Set up SSH
        run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.SSH_PRIVATE_KEY }}" > ~/.ssh/id_rsa
          chmod 600 ~/.ssh/id_rsa

      - name: Deploy to server
        run: |
          ssh user@yourserver.com << 'EOF'
            cd /opt/cobalt-stack
            git pull origin main
            docker compose -f docker-compose.prod.yml build
            docker compose -f docker-compose.prod.yml up -d
          EOF
```

### Rolling Updates

**Zero-Downtime Deployment**:
```bash
# Build new images
docker compose -f docker-compose.prod.yml build

# Scale up new version
docker compose -f docker-compose.prod.yml up -d --scale backend=2 --no-recreate

# Wait for new containers to be healthy
sleep 30

# Scale down old version
docker compose -f docker-compose.prod.yml up -d --scale backend=1
```

## Maintenance

### Regular Maintenance Tasks

**Weekly**:
- Review logs for errors
- Check disk space usage
- Verify backup integrity
- Update Docker images (security patches)

**Monthly**:
- Analyze database query performance
- Review and optimize slow queries
- Clean up old logs and backups
- Update application dependencies
- Security audit (vulnerabilities, access logs)

**Quarterly**:
- Review and update SSL certificates (Let's Encrypt auto-renews)
- Disaster recovery drill (restore from backup)
- Load testing (simulate traffic spikes)
- Security penetration testing

### Update Procedure

**Application Updates**:
```bash
# Pull latest code
cd /opt/cobalt-stack
git fetch --tags
git checkout v1.1.0

# Rebuild images
DOCKER_BUILDKIT=1 docker compose -f docker-compose.prod.yml build

# Rolling update
docker compose -f docker-compose.prod.yml up -d
```

**Database Migrations**:
```bash
# Migrations run automatically on backend startup
# Or manually:
docker compose -f docker-compose.prod.yml exec backend /app/cobalt-stack-backend migrate up

# Rollback (if needed)
docker compose -f docker-compose.prod.yml exec backend /app/cobalt-stack-backend migrate down
```

## References

- [Docker Production Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Nginx Security Best Practices](https://www.nginx.com/blog/nginx-security-best-practices/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [OWASP Security Guidelines](https://owasp.org/www-project-top-ten/)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
