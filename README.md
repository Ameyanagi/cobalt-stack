# Cobalt Stack

Modern full-stack application template with Rust backend (Axum + SeaORM) and Next.js 16 frontend (Bun + React Query + shadcn/ui).

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ ([install](https://rustup.rs/))
- **Bun** 1.2+ ([install](https://bun.sh/))
- **Docker** & **Docker Compose** ([install](https://docs.docker.com/get-docker/))
- **Make** (usually pre-installed on macOS/Linux)

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
- Copy `frontend/.env.local.example` to `frontend/.env.local`
- Install Rust dependencies with `cargo build`
- Install frontend dependencies with `bun install`

### Development

#### Run Full Stack (Docker Compose)

```bash
# Start all services (PostgreSQL, Redis, Backend, Frontend)
make dev

# Access the services
# Frontend: http://localhost:3001
# Backend API: http://localhost:3002/health
# Swagger UI: http://localhost:3002/swagger-ui
```

#### Run Backend Only (Hot Reload)

```bash
# Installs cargo-watch if not present
make dev-backend

# Backend runs on http://localhost:3002
# Auto-reloads on file changes
```

#### Run Frontend Only (Dev Server)

```bash
# Start frontend dev server with hot reload
make dev-frontend

# Frontend runs on http://localhost:3001
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
# Build optimized backend release binary
make build
# Binary location: backend/target/release/cobalt-stack-backend

# Build frontend for production
make build-frontend
# Output: frontend/.next (standalone mode)
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

### Creating Initial Admin User

After running migrations, create an initial admin user for testing and development:

```bash
# Create admin user
make seed-admin

# Default credentials (CHANGE IN PRODUCTION):
# Email: admin@example.com
# Password: admin123
```

**⚠️ IMPORTANT:** The seed script:
- Creates a user with admin role and verified email
- Will skip if an admin user already exists with `admin@example.com`
- Should only be used in development/testing environments
- **Always change the default password after first login**

## 📧 Email Verification

The application includes a complete email verification system for user registration and account security.

### How It Works

1. **User Registration**:
   - User registers with email and password
   - Account is created with `email_verified = false`
   - Verification email is sent automatically (or mocked in development)
   - User is redirected to dashboard but shown an "unverified email" banner

2. **Verification Token**:
   - 32-byte cryptographically secure random token generated
   - Token is hashed with SHA-256 before storage (fast, secure for single-use tokens)
   - Expires after 24 hours (configurable via `EMAIL_VERIFICATION_EXPIRY_SECONDS`)
   - Single-use: marked with `verified_at` timestamp when used

3. **Verification Flow**:
   - User clicks link in email: `/verify-email?token=<token>`
   - Frontend sends token to `/api/auth/verify-email` endpoint
   - Backend validates token (not expired, matches hash, not already used)
   - User's `email_verified` field is set to `true`
   - Success message shown, banner disappears

4. **Resend Verification**:
   - Unverified users see a banner with "Resend verification email" button
   - Sends POST to `/api/auth/send-verification` (requires auth)
   - Previous unexpired tokens remain valid, new token also works

### Configuration

Configure email verification in your environment:

```bash
# Email Verification Settings
EMAIL_VERIFICATION_EXPIRY_SECONDS=86400  # 24 hours (default)
EMAIL_MOCK=true  # Use mock email (logs to console) - set false for production

# SMTP Configuration (only needed if EMAIL_MOCK=false)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@example.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@example.com
```

### Development vs Production

**Development (EMAIL_MOCK=true)**:
- Verification emails are logged to console
- Check backend logs for verification links
- No SMTP server required
- Perfect for testing and local development

**Production (EMAIL_MOCK=false)**:
- Real emails sent via SMTP
- Requires valid SMTP credentials
- Configure your email service (Gmail, SendGrid, AWS SES, etc.)
- Monitor email delivery rates

### API Endpoints

- **POST /api/auth/send-verification**: Send/resend verification email (requires authentication)
- **POST /api/auth/verify-email**: Verify email with token (public endpoint)

See the OpenAPI schema at `/swagger-ui` for detailed request/response formats.

## 👤 Role-Based Access Control & Admin System

The application includes a complete role-based authorization system with admin capabilities.

### User Roles

- **user**: Standard user role (default for all registrations)
- **admin**: Administrator role with access to admin dashboard and management endpoints

Roles are stored as a PostgreSQL ENUM type for type safety and performance.

### Admin Features

**Admin Dashboard** (`/admin`):
- Platform statistics (total users, verified users, admin count)
- Quick actions and navigation

**User Management** (`/admin/users`):
- Paginated user list with filtering:
  - Search by username or email
  - Filter by role (all/admin/user)
  - Filter by verification status (all/verified/unverified)
- View user details (email, role, verification status, account status)
- User actions:
  - **Disable user**: Soft-delete with `disabled_at` timestamp (reversible)
  - **Enable user**: Restore previously disabled account

### Security

**Protected Routes**:
- Frontend checks user role before rendering admin pages
- Non-admin users are automatically redirected to dashboard
- All admin API endpoints require both authentication AND admin role

**Admin Middleware**:
- Layered authentication: JWT validation → role verification
- Returns 403 Forbidden for non-admin users
- Applied to all `/api/admin/*` routes

### Admin API Endpoints

All endpoints require Bearer token authentication with admin role:

- **GET /api/admin/stats**: Platform statistics
- **GET /api/admin/users**: List users with pagination and filters
- **GET /api/admin/users/:id**: Get user details
- **PATCH /api/admin/users/:id/disable**: Disable user account
- **PATCH /api/admin/users/:id/enable**: Enable user account

See the OpenAPI schema at `/swagger-ui` for detailed request/response formats.

## 📝 OpenAPI & Type Generation

```bash
# Generate OpenAPI schema from backend (utoipa)
make generate-openapi
# Output: openapi/schema.json

# Generate TypeScript types from OpenAPI schema
make generate-types
# Output: frontend/src/types/api.ts
```

The frontend API client (`src/lib/api-client.ts`) is fully type-safe, automatically typed from the OpenAPI schema.

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
├── frontend/
│   ├── src/
│   │   ├── app/           # Next.js 16 App Router pages
│   │   ├── components/    # shadcn/ui components
│   │   ├── lib/           # API client, utilities
│   │   └── types/         # Generated TypeScript types (from OpenAPI)
│   ├── package.json       # Frontend dependencies
│   ├── next.config.ts     # Next.js configuration (standalone output)
│   └── Dockerfile         # Multi-stage build with Bun
├── openapi/
│   └── schema.json        # Generated OpenAPI schema
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

- **Framework**: Next.js 16 (App Router with src directory)
- **Runtime**: Bun 1.2+ (fast JavaScript runtime and package manager)
- **UI Components**: shadcn/ui (Radix UI primitives) + TailwindCSS 4.1
- **State Management**: React Query (@tanstack/react-query 5.x)
- **Type Safety**: TypeScript strict mode + OpenAPI-generated types
- **API Client**: Type-safe client auto-generated from OpenAPI schema

**Frontend Features:**
- **Type-Safe API Integration**: Full TypeScript types from backend OpenAPI schema
- **Error Boundary**: Graceful error handling for production
- **Environment Validation**: Type-safe env var access with runtime validation
- **Health Check UI**: Real-time backend health monitoring
- **Docker Ready**: Multi-stage build with standalone output (~50MB)

## 🐳 Docker Environments

### Development (`docker-compose.yml`)

- **PostgreSQL**: Port 5432 exposed, persistent volume
- **Redis**: Port 6379 exposed, no password
- **Backend**: Port 3002 exposed, hot reload, debug logging, CORS enabled
- **Frontend**: Port 3001 exposed, connected to backend service network

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

### Docker Compose Environment (Recommended)

For Docker deployment, copy `.env.example` to `.env` and customize as needed:

```bash
# Port Configuration
FRONTEND_PORT=2727       # Frontend accessible at http://localhost:2727
BACKEND_PORT=2750        # Backend API at http://localhost:2750
POSTGRES_PORT=2800       # PostgreSQL at localhost:2800
REDIS_PORT=2900          # Redis at localhost:2900

# API URL (must match BACKEND_PORT)
NEXT_PUBLIC_API_URL=http://localhost:2750

# Database Configuration
POSTGRES_DB=cobalt_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres

# Backend Logging
RUST_LOG=debug  # Options: trace, debug, info, warn, error
```

All ports and URLs are now configurable through the `.env` file. When you change `BACKEND_PORT`, make sure to update `NEXT_PUBLIC_API_URL` to match.

### Local Development Environment

**Backend** - Copy `backend/.env.example` to `backend/.env`:

```bash
# Server
PORT=3000
RUST_LOG=debug

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/cobalt_dev

# Redis
REDIS_URL=redis://localhost:6379

# Security
JWT_SECRET=change-me-in-production

# CORS
CORS_ORIGINS=http://localhost:3001,http://localhost:3000
```

**Frontend** - Copy `frontend/.env.local.example` to `frontend/.env.local`:

```bash
# Backend API URL
NEXT_PUBLIC_API_URL=http://localhost:3002

# Optional: NextAuth configuration
# NEXTAUTH_URL=http://localhost:3001
# NEXTAUTH_SECRET=your-secret-key-change-me-in-production
```

## 🚨 Troubleshooting

### Common Issues

**1. Port Already in Use**
```bash
# Check what's using ports
lsof -i :3001  # Frontend
lsof -i :3002  # Backend
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
