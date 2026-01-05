# Environment Variables Reference

Complete reference for all environment variables used in Cobalt Stack, including descriptions, default values, security considerations, and platform-specific notes.

## Overview

Cobalt Stack uses environment variables for configuration across development, staging, and production environments. Variables are loaded from `.env` files and can be overridden via system environment variables.

**Environment File Locations**:
- Root directory: `.env` (development, docker-compose)
- Backend: `backend/.env` (local development without Docker)
- Frontend: `frontend/.env.local` (local frontend development)
- Production: `.env.production` (production deployment)

**Priority** (highest to lowest):
1. System environment variables
2. `.env` file (loaded by docker-compose or application)
3. Default values (hardcoded in application)

## Core Application Variables

### Port Configuration

#### `FRONTEND_PORT`
- **Description**: Port for frontend application on host machine
- **Default**: `2727`
- **Required**: No
- **Type**: Integer (1-65535)
- **Example**: `FRONTEND_PORT=2727`
- **Platform-Specific**:
  - Development: Host port mapped to container
  - Production: Usually behind reverse proxy (Nginx, Traefik)
- **Security**: Low risk (development convenience)

#### `BACKEND_PORT`
- **Description**: Port for backend API on host machine
- **Default**: `2750`
- **Required**: No
- **Type**: Integer (1-65535)
- **Example**: `BACKEND_PORT=2750`
- **Platform-Specific**:
  - Development: Host port mapped to container
  - Production: Behind reverse proxy, not directly exposed
- **Security**: Low risk (internal network)

#### `PORT` (Backend Internal)
- **Description**: Port backend listens on **inside container**
- **Default**: `3000`
- **Required**: No
- **Type**: Integer
- **Example**: `PORT=3000`
- **Notes**: Different from `BACKEND_PORT` (host port)
- **Security**: Internal only

### API URL Configuration

#### `NEXT_PUBLIC_API_URL`
- **Description**: Backend API URL accessible from frontend (public)
- **Default**: `http://localhost:2750`
- **Required**: Yes (production)
- **Type**: URL (http/https)
- **Example**:
  - Development: `NEXT_PUBLIC_API_URL=http://localhost:2750`
  - Production: `NEXT_PUBLIC_API_URL=https://api.yourdomain.com`
- **Platform-Specific**:
  - **IMPORTANT**: Embedded in frontend build (not runtime)
  - Must be set as build arg for Docker builds
  - Cannot be changed after build without rebuilding
- **Security**: Public (visible in browser), do not include secrets

**Docker Build Usage**:
```bash
docker build --build-arg NEXT_PUBLIC_API_URL=https://api.example.com .
```

#### `FRONTEND_URL`
- **Description**: Frontend URL for CORS configuration in backend
- **Default**: `http://localhost:2727`
- **Required**: Yes (production)
- **Type**: URL (http/https)
- **Example**:
  - Development: `FRONTEND_URL=http://localhost:2727`
  - Production: `FRONTEND_URL=https://yourdomain.com`
- **Platform-Specific**:
  - Backend uses this to configure allowed CORS origins
  - Production: Must match actual domain
- **Security**: Medium risk (CORS misconfiguration can allow unauthorized access)

## Database Configuration

### PostgreSQL

#### `DATABASE_URL`
- **Description**: Complete PostgreSQL connection string
- **Default**: Constructed from other variables
- **Required**: Yes
- **Type**: Connection string
- **Format**: `postgresql://USER:PASSWORD@HOST:PORT/DATABASE`
- **Example**:
  - Development: `postgresql://postgres:postgres@localhost:5432/cobalt_dev`
  - Docker: `postgresql://postgres:postgres@postgres:5432/cobalt_dev`
  - Production: `postgresql://postgres:STRONG_PASSWORD@postgres:5432/cobalt_prod`
- **Platform-Specific**:
  - Development: Use `localhost` when running backend locally
  - Docker: Use service name (`postgres`) as host
  - Production: Always use internal service name, never expose externally
- **Security**: ⚠️ **HIGH RISK** - Contains database password

**Alternative**: Set individual components instead:

#### `DATABASE_NAME`
- **Description**: PostgreSQL database name
- **Default**: `cobalt_dev` (development), `cobalt_prod` (production)
- **Required**: No (has default)
- **Type**: String
- **Example**: `DATABASE_NAME=cobalt_prod`
- **Security**: Low risk

#### `DATABASE_USER`
- **Description**: PostgreSQL username
- **Default**: `postgres`
- **Required**: No (has default)
- **Type**: String
- **Example**: `DATABASE_USER=postgres`
- **Security**: Low risk (username without password)

#### `DATABASE_PASSWORD`
- **Description**: PostgreSQL password
- **Default**: `postgres` (development), **REQUIRED** (production)
- **Required**: Yes (production)
- **Type**: String (min 16 chars recommended)
- **Example**: `DATABASE_PASSWORD=X7k9mP2nQ5wR8tY1`
- **Generation**:
  ```bash
  openssl rand -base64 32
  ```
- **Security**: ⚠️ **CRITICAL** - Never commit, use secrets management
- **Best Practices**:
  - Minimum 32 characters
  - Mix of alphanumeric and symbols
  - Unique per environment
  - Rotate quarterly

#### `POSTGRES_PORT`
- **Description**: PostgreSQL port on host machine (development only)
- **Default**: `2800`
- **Required**: No
- **Type**: Integer
- **Example**: `POSTGRES_PORT=2800`
- **Platform-Specific**:
  - Development: Allows direct database access (psql, GUI clients)
  - Production: **Remove port mapping** - internal network only
- **Security**: Medium risk (direct database access)

### Redis/Valkey

#### `REDIS_URL`
- **Description**: Redis connection string
- **Default**: `redis://localhost:6379` (dev), `redis://:PASSWORD@redis:6379` (prod)
- **Required**: Yes
- **Type**: Connection string
- **Format**:
  - No password: `redis://HOST:PORT`
  - With password: `redis://:PASSWORD@HOST:PORT`
- **Example**:
  - Development: `redis://redis:6379`
  - Production: `redis://:X7k9mP2nQ5wR8tY1@redis:6379`
- **Security**: ⚠️ **HIGH RISK** - Contains Redis password (if set)

#### `REDIS_PASSWORD`
- **Description**: Redis authentication password
- **Default**: None (development), **REQUIRED** (production)
- **Required**: Yes (production)
- **Type**: String
- **Example**: `REDIS_PASSWORD=X7k9mP2nQ5wR8tY1`
- **Generation**:
  ```bash
  openssl rand -base64 32
  ```
- **Security**: ⚠️ **CRITICAL** - Never commit
- **Best Practices**:
  - Minimum 32 characters
  - Alphanumeric only (some special chars cause issues)
  - Unique per environment

#### `REDIS_PORT`
- **Description**: Redis port on host machine (development only)
- **Default**: `2900`
- **Required**: No
- **Type**: Integer
- **Example**: `REDIS_PORT=2900`
- **Platform-Specific**:
  - Development: Allows direct Redis access (redis-cli, GUI)
  - Production: **Remove port mapping**
- **Security**: Medium risk

## Authentication Configuration

### JWT Settings

#### `JWT_SECRET`
- **Description**: Secret key for signing JWT tokens
- **Default**: None (auto-generated if not set)
- **Required**: Yes (production - must be consistent)
- **Type**: String (min 32 chars, 64+ recommended)
- **Example**: `JWT_SECRET=X7k9mP2nQ5wR8tY1Z3vB6nM9qP2sR5u8W1x4A7c0E3f6`
- **Generation**:
  ```bash
  openssl rand -base64 64
  ```
- **Security**: ⚠️ **CRITICAL** - Compromise allows token forgery
- **Best Practices**:
  - Minimum 64 characters in production
  - Use cryptographically secure random generation
  - **Never commit to version control**
  - Rotate after security incident
  - Same secret across all backend instances (horizontal scaling)
- **Platform-Specific**:
  - Development: Can be hardcoded for convenience
  - Production: Must be set via secure secrets management

#### `ACCESS_TOKEN_EXPIRY`
- **Description**: Access token lifetime (JWT)
- **Default**: `900` (15 minutes, hardcoded in backend)
- **Required**: No
- **Type**: Integer (seconds)
- **Notes**: Not currently configurable via env var (hardcoded)
- **Security**: Shorter = more secure (limits token theft window)

#### `REFRESH_TOKEN_EXPIRY`
- **Description**: Refresh token lifetime
- **Default**: `604800` (7 days, hardcoded in backend)
- **Required**: No
- **Type**: Integer (seconds)
- **Notes**: Not currently configurable via env var (hardcoded)
- **Security**: Balance security and UX (7 days standard)

## Email Configuration

### Email Service

#### `EMAIL_MOCK`
- **Description**: Mock email sending (print to console instead)
- **Default**: `true`
- **Required**: No
- **Type**: Boolean (`true`, `false`)
- **Example**:
  - Development: `EMAIL_MOCK=true`
  - Production: `EMAIL_MOCK=false`
- **Platform-Specific**:
  - Development: Enables testing without email provider
  - Production: **Must be false** (requires real email service)
- **Security**: Low risk

#### `SMTP_HOST`
- **Description**: SMTP server hostname
- **Default**: None
- **Required**: Yes (production with EMAIL_MOCK=false)
- **Type**: String (hostname or IP)
- **Example**:
  - SendGrid: `smtp.sendgrid.net`
  - AWS SES: `email-smtp.us-east-1.amazonaws.com`
  - Mailgun: `smtp.mailgun.org`
- **Security**: Low risk (public SMTP hostname)

#### `SMTP_PORT`
- **Description**: SMTP server port
- **Default**: `587`
- **Required**: No (has default)
- **Type**: Integer
- **Common Ports**:
  - `587`: STARTTLS (recommended)
  - `465`: SSL/TLS
  - `25`: Unencrypted (not recommended)
- **Example**: `SMTP_PORT=587`
- **Security**: Low risk

#### `SMTP_USER`
- **Description**: SMTP authentication username
- **Default**: None
- **Required**: Yes (production with EMAIL_MOCK=false)
- **Type**: String
- **Example**:
  - SendGrid: `apikey` (literal string)
  - AWS SES: IAM access key ID
  - Mailgun: Mailgun API username
- **Security**: Medium risk (username without password)

#### `SMTP_PASSWORD`
- **Description**: SMTP authentication password or API key
- **Default**: None
- **Required**: Yes (production with EMAIL_MOCK=false)
- **Type**: String
- **Example**: `SMTP_PASSWORD=SG.xxxxxxxxxxxxxxxxxxx` (SendGrid API key)
- **Security**: ⚠️ **HIGH RISK** - Allows email sending
- **Best Practices**:
  - Use API keys instead of passwords when available
  - Restrict API key permissions (send-only)
  - Rotate keys quarterly
  - Never commit to version control

#### `EMAIL_FROM`
- **Description**: Sender email address
- **Default**: `noreply@yourdomain.com`
- **Required**: No (has default, but should be customized)
- **Type**: Email address
- **Example**: `EMAIL_FROM=noreply@yourdomain.com`
- **Security**: Low risk

#### `EMAIL_VERIFICATION_EXPIRY_SECONDS`
- **Description**: Email verification code lifetime
- **Default**: `86400` (24 hours)
- **Required**: No
- **Type**: Integer (seconds)
- **Example**: `EMAIL_VERIFICATION_EXPIRY_SECONDS=86400`
- **Security**: Low risk (shorter = more secure, but affects UX)

## Logging Configuration

#### `RUST_LOG`
- **Description**: Rust application log level
- **Default**: `info`
- **Required**: No
- **Type**: Log level enum
- **Values**:
  - `trace`: Most verbose (everything)
  - `debug`: Debugging information
  - **`info`**: General informational (production default)
  - `warn`: Warnings only
  - `error`: Errors only
- **Example**:
  - Development: `RUST_LOG=debug`
  - Production: `RUST_LOG=info`
- **Advanced Filtering**:
  ```bash
  # Module-specific levels
  RUST_LOG=cobalt_stack_backend=debug,tower_http=info,sea_orm=warn
  ```
- **Security**: Low risk (verbose logs can leak sensitive data)
- **Performance**: `trace` and `debug` impact performance

#### `RUST_BACKTRACE`
- **Description**: Enable Rust panic backtraces
- **Default**: `0` (disabled)
- **Required**: No
- **Type**: Boolean (`0`, `1`, `full`)
- **Example**: `RUST_BACKTRACE=1`
- **Platform-Specific**:
  - Development: Enable for debugging (`RUST_BACKTRACE=1`)
  - Production: Disable (leaks internal paths)
- **Security**: Medium risk (leaks file paths and internal structure)

## Docker Configuration

#### `IMAGE_TAG`
- **Description**: Docker image version tag
- **Default**: `latest`
- **Required**: No (but recommended for production)
- **Type**: String (semantic version recommended)
- **Example**: `IMAGE_TAG=v1.0.0`
- **Platform-Specific**:
  - Development: `latest` is fine
  - Production: **Always use specific version tags** (e.g., `v1.0.0`)
- **Security**: Low risk
- **Best Practices**:
  - Use semantic versioning (v1.2.3)
  - Tag images before deploying
  - Never use `latest` in production

## Environment-Specific Configuration

### Development (.env)

```bash
# Ports (for docker-compose host mapping)
FRONTEND_PORT=2727
BACKEND_PORT=2750
POSTGRES_PORT=2800
REDIS_PORT=2900

# API URLs
NEXT_PUBLIC_API_URL=http://localhost:2750
FRONTEND_URL=http://localhost:2727

# Database (weak credentials OK for dev)
POSTGRES_DB=cobalt_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres

# Backend
RUST_LOG=debug
RUST_BACKTRACE=1

# Email (mock mode)
EMAIL_MOCK=true
EMAIL_VERIFICATION_EXPIRY_SECONDS=86400

# JWT (can be weak for dev, or omit for auto-generation)
# JWT_SECRET=dev-secret-do-not-use-in-production
```

### Production (.env.production)

```bash
# CRITICAL SECRETS (generate with openssl rand -base64 32)
DATABASE_PASSWORD=<GENERATE_STRONG_PASSWORD>
REDIS_PASSWORD=<GENERATE_STRONG_PASSWORD>
JWT_SECRET=<GENERATE_MIN_64_CHAR_SECRET>

# Database
DATABASE_NAME=cobalt_prod
DATABASE_USER=postgres

# Application
BACKEND_PORT=3000
IMAGE_TAG=v1.0.0
RUST_LOG=info

# Email (REQUIRED for production)
EMAIL_MOCK=false
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USER=apikey
SMTP_PASSWORD=<SENDGRID_API_KEY>
EMAIL_FROM=noreply@yourdomain.com
EMAIL_VERIFICATION_EXPIRY_SECONDS=86400

# URLs (must match actual domain)
NEXT_PUBLIC_API_URL=https://api.yourdomain.com
FRONTEND_URL=https://yourdomain.com
```

## Security Best Practices

### Secret Management

**Never Commit Secrets**:
```bash
# .gitignore (ensure these are present)
.env
.env.local
.env.production
.env.*.local
```

**Audit Secrets**:
```bash
# Check for accidentally committed secrets
git log --all --full-history -- .env
git log --all --full-history -- .env.production

# Remove from history if found (destructive!)
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch .env' \
  --prune-empty --tag-name-filter cat -- --all
```

**Secret Generation**:
```bash
# Strong passwords (32 chars)
openssl rand -base64 32

# Extra strong JWT secret (64 chars)
openssl rand -base64 64

# Alphanumeric only (for systems with special char issues)
openssl rand -hex 32
```

**Secret Storage**:
- Development: `.env` file (git-ignored)
- Production: One of:
  - Environment file on server (chmod 600)
  - Docker secrets (Swarm/Kubernetes)
  - External secrets manager (Vault, AWS Secrets Manager)

**Secret Rotation**:
- Quarterly rotation for all passwords
- Immediate rotation after:
  - Security incident
  - Employee departure
  - Potential exposure

### Environment File Security

**File Permissions** (Production):
```bash
# Restrict access to owner only
chmod 600 .env.production

# Verify permissions
ls -la .env.production
# Should show: -rw------- (600)
```

**Ownership**:
```bash
# Ensure owned by deployment user
chown deployuser:deployuser .env.production
```

**Backup Security**:
```bash
# Encrypt backups containing secrets
gpg -c .env.production  # Creates .env.production.gpg
rm .env.production       # Remove plaintext

# Decrypt when needed
gpg .env.production.gpg  # Prompts for passphrase
```

## Troubleshooting

### Common Issues

#### 1. `DATABASE_URL` Connection Failed
**Error**: `Connection refused` or `Unknown host`

**Diagnosis**:
```bash
# Check DATABASE_URL value
echo $DATABASE_URL

# Verify database is running
docker compose ps postgres

# Test connection
docker compose exec postgres psql -U postgres -d cobalt_dev
```

**Solutions**:
- Local development: Use `localhost` as host
- Docker: Use service name (`postgres`) as host
- Verify port (5432 inside container)

#### 2. CORS Errors
**Error**: `Access-Control-Allow-Origin` error in browser

**Diagnosis**:
- Check `FRONTEND_URL` matches actual frontend domain
- Verify backend logs for CORS configuration

**Solutions**:
- Development: Ensure `FRONTEND_URL=http://localhost:2727`
- Production: Set `FRONTEND_URL=https://yourdomain.com`
- Restart backend after changing

#### 3. JWT Token Errors
**Error**: `Invalid token` or `Token expired`

**Diagnosis**:
```bash
# Check JWT_SECRET is set
echo $JWT_SECRET | wc -c  # Should be 32+ characters
```

**Solutions**:
- Ensure `JWT_SECRET` is set and consistent across backend instances
- Generate strong secret: `openssl rand -base64 64`
- Clear browser cookies and local storage
- Verify token expiry times

#### 4. Email Not Sending
**Error**: Verification emails not received

**Diagnosis**:
```bash
# Check EMAIL_MOCK setting
echo $EMAIL_MOCK

# Verify SMTP credentials
echo $SMTP_HOST $SMTP_PORT $SMTP_USER
```

**Solutions**:
- Development: Set `EMAIL_MOCK=true` and check console logs
- Production: Verify SMTP credentials with provider
- Check spam folder
- Test SMTP connection: `telnet smtp.sendgrid.net 587`

#### 5. Variable Not Loading
**Error**: Application uses default instead of env var value

**Diagnosis**:
```bash
# Check if variable is set
echo $VARIABLE_NAME

# Check docker-compose loads .env
docker compose config | grep VARIABLE_NAME
```

**Solutions**:
- Ensure `.env` file exists in correct location
- Check variable name spelling (case-sensitive)
- Restart containers: `docker compose up -d`
- For build-time vars (NEXT_PUBLIC_*), rebuild: `docker compose build`

## Validation Checklist

**Before Production Deployment**:
- [ ] All critical secrets generated (DATABASE_PASSWORD, REDIS_PASSWORD, JWT_SECRET)
- [ ] Secrets are 32+ characters (64+ for JWT_SECRET)
- [ ] `EMAIL_MOCK=false` and SMTP credentials configured
- [ ] `NEXT_PUBLIC_API_URL` matches production domain
- [ ] `FRONTEND_URL` matches production domain
- [ ] `RUST_LOG=info` (not debug or trace)
- [ ] `RUST_BACKTRACE=0` (disabled)
- [ ] `.env.production` file permissions are 600
- [ ] No secrets committed to version control
- [ ] Secrets documented in secure password manager

## References

- [Twelve-Factor App: Configuration](https://12factor.net/config)
- [OWASP: Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [Docker Secrets](https://docs.docker.com/engine/swarm/secrets/)
- [PostgreSQL Connection Strings](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING)
- [Redis Connection Strings](https://redis.io/docs/manual/cli/)
