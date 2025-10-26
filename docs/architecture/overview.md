# Architecture Overview

This document provides a comprehensive overview of the Cobalt Stack architecture, including system design, component interactions, and architectural patterns.

## System Architecture

Cobalt Stack follows a modern **three-tier architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend Layer                        │
│  Next.js 15 + TypeScript + Tailwind + shadcn/ui (Bun)      │
│              Port: 2727 (dev) / 3001 (container)            │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTP/REST + JWT
                         │
┌────────────────────────▼────────────────────────────────────┐
│                       Backend Layer                          │
│     Rust + Axum + SeaORM + Tokio (Async Runtime)           │
│              Port: 2750 (dev) / 3000 (container)            │
└───────────┬──────────────────────────┬──────────────────────┘
            │                          │
┌───────────▼────────────┐  ┌─────────▼──────────────────────┐
│   PostgreSQL 15        │  │   Redis/Valkey 7               │
│   (Primary Database)   │  │   (Cache & Session Store)      │
│   Port: 2800 (dev)     │  │   Port: 2900 (dev)             │
└────────────────────────┘  └────────────────────────────────┘
```

## Architectural Layers

### 1. Frontend Layer (Next.js)

**Purpose**: User interface and client-side logic

**Technology Stack**:
- **Framework**: Next.js 15 with App Router
- **Runtime**: Bun (fast JavaScript runtime)
- **Language**: TypeScript (type safety)
- **Styling**: Tailwind CSS (utility-first)
- **Components**: shadcn/ui (accessible components)
- **State Management**: React hooks + Context API
- **HTTP Client**: Fetch API with type-safe OpenAPI-generated types

**Key Features**:
- Server-side rendering (SSR) and static generation
- Client-side navigation with prefetching
- Type-safe API client from OpenAPI schema
- Form validation with react-hook-form + zod
- JWT token management (HttpOnly cookies)

### 2. Backend Layer (Rust)

**Purpose**: Business logic, authentication, API endpoints

**Technology Stack**:
- **Framework**: Axum (web framework)
- **Runtime**: Tokio (async runtime)
- **ORM**: SeaORM (database abstraction)
- **Serialization**: Serde (JSON)
- **Documentation**: utoipa (OpenAPI generation)
- **Logging**: tracing + tracing-subscriber

**Architecture Pattern**: **Layered Architecture**

```
┌─────────────────────────────────────────────────────────┐
│                    HTTP Layer                           │
│  Handlers (auth, admin, health)                         │
│  Request validation, response formatting                │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                 Middleware Layer                         │
│  Auth middleware, Admin middleware, CORS, Tracing       │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                 Service Layer                            │
│  Auth service, Email service, Valkey service            │
│  Business logic, token management, caching              │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                  Data Layer                              │
│  SeaORM models, Database queries, Redis operations      │
└─────────────────────────────────────────────────────────┘
```

### 3. Data Layer

#### PostgreSQL (Primary Database)

**Purpose**: Persistent data storage

**Schema**:
- `users` - User accounts (email, hashed password, roles, status)
- `refresh_tokens` - JWT refresh tokens (token_hash, expiry, rotation tracking)
- `email_verifications` - Email verification tokens (code, expiry)
- `o_auth_accounts` - OAuth provider linkage (future feature)

**Key Features**:
- ACID compliance for data integrity
- Foreign key constraints for referential integrity
- Indexes on frequently queried fields
- Migration management with SeaORM CLI

#### Redis/Valkey (Cache & Session Store)

**Purpose**: High-performance caching and session management

**Use Cases**:
- Token blacklist (revoked JWTs before expiry)
- Rate limiting (IP-based request throttling)
- Session data (optional - currently stateless JWT)
- Cache layer for expensive queries (future)

**Key Features**:
- In-memory storage (sub-millisecond latency)
- TTL support (automatic expiration)
- Atomic operations (thread-safe counters)
- Persistence to disk (optional)

## Communication Patterns

### REST API

**Protocol**: HTTP/1.1 with JSON payloads

**Authentication Flow**:
1. Client sends credentials to `/api/auth/login`
2. Backend validates credentials
3. Backend issues access token (JWT, 15min) + refresh token (HttpOnly cookie, 7 days)
4. Client includes access token in `Authorization: Bearer <token>` header
5. On access token expiry, client calls `/api/auth/refresh` with cookie
6. Backend rotates refresh token and issues new access token

**CORS Policy**:
- Development: Allow origins ending with `:2727`
- Production: Specific allowed origins (configure via environment)
- Credentials: Allowed (for HttpOnly cookies)
- Methods: GET, POST, PUT, DELETE, OPTIONS

### Database Connections

**Connection Pooling**:
- SeaORM manages connection pool to PostgreSQL
- Redis client uses connection manager
- Pool size configurable via environment variables

**Query Patterns**:
- Async/await for non-blocking I/O
- Prepared statements (SQL injection prevention)
- Transaction support for multi-step operations

## Security Architecture

### Authentication

**Token Strategy**: Dual-token (Access + Refresh)

- **Access Token**: JWT, short-lived (15 min), stateless
- **Refresh Token**: Random hash, long-lived (7 days), stored in database

**Password Security**:
- Argon2id hashing (memory-hard, GPU-resistant)
- Salt automatically generated per-password
- Cost parameters tuned for ~100ms hash time

### Authorization

**Role-Based Access Control (RBAC)**:
- Roles: `user`, `admin`
- Middleware: `auth_middleware` (verifies JWT), `admin_middleware` (checks role)
- Protected routes require both authentication and authorization

### Attack Prevention

- **SQL Injection**: Parameterized queries via SeaORM
- **XSS**: JSON responses (no HTML rendering in backend)
- **CSRF**: SameSite cookies + CORS restrictions
- **Brute Force**: Rate limiting on auth endpoints (future)
- **Token Theft**: Token rotation, blacklisting, short expiry

## Performance Considerations

### Backend Optimizations

- **Async Runtime**: Tokio for concurrent request handling
- **Connection Pooling**: Reuse database connections
- **Caching**: Redis for token blacklist (avoids DB queries)
- **Lazy Loading**: Services initialized on-demand

### Database Optimizations

- **Indexes**: On `users.email`, `refresh_tokens.token_hash`
- **Connection Limits**: Prevent connection exhaustion
- **Query Optimization**: Use EXPLAIN ANALYZE in development

### Frontend Optimizations

- **Code Splitting**: Next.js automatic splitting
- **Prefetching**: Link prefetch on hover
- **Static Assets**: CDN-ready (future)
- **Image Optimization**: Next.js built-in

## Scalability Patterns

### Horizontal Scaling

**Stateless Backend**:
- JWT-based authentication (no session state)
- Multiple backend instances behind load balancer
- Shared PostgreSQL and Redis instances

**Load Balancing**:
- Round-robin or least-connections
- Health check endpoint: `/health`
- Session affinity not required (stateless)

### Database Scaling

**Read Replicas**:
- Primary for writes, replicas for reads
- SeaORM supports read/write splitting
- Eventual consistency for non-critical reads

**Connection Pooling**:
- Limit connections per instance
- Pool size = (CPU cores * 2) + disk spindles

### Caching Strategy

**Cache Layers**:
1. **Application Cache**: In-memory LRU cache (future)
2. **Redis Cache**: Shared cache across instances
3. **Database Query Cache**: PostgreSQL query cache

**Cache Invalidation**:
- TTL-based (time-to-live)
- Event-based (on data mutation)
- Manual purge (admin endpoint)

## Monitoring & Observability

### Logging

**Framework**: tracing + tracing-subscriber

**Log Levels**:
- `trace`: Detailed debug information
- `debug`: Development debugging
- `info`: General informational messages
- `warn`: Warning conditions
- `error`: Error conditions requiring attention

**Structured Logging**:
- JSON format for machine parsing
- Context propagation (request IDs)
- Performance metrics (request duration)

### Health Checks

**Endpoint**: `GET /health`

**Checks**:
- Database connectivity
- Redis connectivity
- System resources (future: memory, CPU)

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2025-10-27T12:00:00Z",
  "services": {
    "database": "ok",
    "redis": "ok"
  }
}
```

### Metrics (Future)

- Request rate (requests/sec)
- Response time (p50, p95, p99)
- Error rate (errors/sec)
- Database query time
- Cache hit rate

## Deployment Architecture

### Development

```
Host Machine (localhost)
├── Frontend (Bun dev server): localhost:2727
├── Backend (cargo run): localhost:2750
├── PostgreSQL (Docker): localhost:2800
└── Redis (Docker): localhost:2900
```

### Production (Docker Compose)

```
Docker Network (cobalt-prod-network)
├── Frontend Container (Next.js): port 2727 → 3001
├── Backend Container (Rust binary): port 2750 → 3000
├── PostgreSQL Container: internal only (no exposed port)
└── Redis Container: internal only (no exposed port)
```

**Volumes**:
- `postgres_prod_data`: Persistent database storage
- `redis_prod_data`: Persistent cache storage (optional)

**Resource Limits**:
- Backend: 2 CPU cores, 2GB RAM (limit), 0.5 CPU, 512MB RAM (reservation)

## Design Patterns

### Backend Patterns

- **Repository Pattern**: SeaORM models as data access layer
- **Service Pattern**: Business logic isolated from HTTP handlers
- **Middleware Pattern**: Cross-cutting concerns (auth, logging, CORS)
- **State Pattern**: Shared application state (DB, config)

### Frontend Patterns

- **Component Composition**: Reusable UI components
- **Custom Hooks**: Shared logic extraction
- **Context API**: Global state management
- **Form Patterns**: react-hook-form + zod validation

## Technology Decisions

### Why Rust for Backend?

- **Performance**: Compiled, zero-cost abstractions, no GC pauses
- **Safety**: Compile-time memory safety, no null pointers
- **Concurrency**: Async/await, lightweight tasks (Tokio)
- **Ecosystem**: Mature web frameworks (Axum), ORMs (SeaORM)

### Why Next.js for Frontend?

- **Developer Experience**: Fast refresh, TypeScript support
- **Performance**: SSR, static generation, code splitting
- **SEO-Friendly**: Server-side rendering
- **Ecosystem**: React community, component libraries

### Why PostgreSQL?

- **Reliability**: ACID compliance, mature ecosystem
- **Features**: JSON support, full-text search, extensions
- **Performance**: Query planner, indexing, partitioning
- **Open Source**: No vendor lock-in

### Why Redis/Valkey?

- **Speed**: In-memory storage (sub-ms latency)
- **Simplicity**: Key-value store, simple API
- **Versatility**: Cache, session store, rate limiting
- **Compatibility**: Valkey (Redis fork) maintains protocol compatibility

## Future Enhancements

### Planned Features

- OAuth 2.0 integration (Google, GitHub)
- Two-factor authentication (TOTP)
- Rate limiting (per-user, per-endpoint)
- Audit logging (user actions)
- Email service integration (SendGrid, AWS SES)

### Infrastructure Improvements

- Kubernetes deployment manifests
- CI/CD pipeline (GitHub Actions)
- Monitoring stack (Prometheus + Grafana)
- Error tracking (Sentry)
- Load testing suite (k6, Locust)

## References

- [Axum Documentation](https://docs.rs/axum)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM)
- [Next.js Documentation](https://nextjs.org/docs)
- [PostgreSQL Documentation](https://www.postgresql.org/docs)
- [Redis Documentation](https://redis.io/docs)
