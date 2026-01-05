# Backend Architecture

Comprehensive guide to the Rust backend architecture, including code organization, design patterns, and implementation details.

## Technology Stack

### Core Framework

- **Axum 0.7**: Modern web framework built on Tokio
- **Tokio**: Asynchronous runtime for Rust
- **Tower**: Middleware and service abstractions
- **Tower-HTTP**: HTTP-specific middleware (CORS, tracing)

### Database & Caching

- **SeaORM 1.1**: Async ORM for PostgreSQL
- **Redis 0.27**: Async Redis client with connection pooling

### Authentication & Security

- **jsonwebtoken 9**: JWT creation and verification
- **argon2 0.5**: Password hashing (Argon2id algorithm)
- **sha2 0.10**: Token hashing (SHA-256)
- **rand 0.8**: Cryptographically secure random generation

### Utilities

- **serde 1**: Serialization/deserialization
- **tracing 0.1**: Structured logging
- **utoipa 5**: OpenAPI documentation generation
- **anyhow/thiserror**: Error handling

## Project Structure

```
backend/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library root (public API)
│   ├── config/                 # Configuration management
│   │   └── mod.rs              # Config structs (future)
│   ├── handlers/               # HTTP request handlers
│   │   ├── mod.rs              # Handler module exports
│   │   ├── auth.rs             # Auth endpoints (register, login, etc.)
│   │   ├── admin.rs            # Admin endpoints (user management)
│   │   └── health.rs           # Health check endpoint
│   ├── middleware/             # HTTP middleware
│   │   ├── mod.rs              # Middleware exports
│   │   ├── auth.rs             # JWT authentication middleware
│   │   └── admin.rs            # Admin authorization middleware
│   ├── services/               # Business logic services
│   │   ├── mod.rs              # Service exports
│   │   ├── auth/               # Authentication services
│   │   │   ├── mod.rs          # Auth service exports
│   │   │   ├── jwt.rs          # JWT creation/verification
│   │   │   ├── password.rs     # Password hashing/verification
│   │   │   ├── token_rotation.rs # Refresh token rotation
│   │   │   └── error.rs        # Auth-specific errors
│   │   ├── email/              # Email services
│   │   │   ├── mod.rs          # Email exports
│   │   │   └── verification.rs # Email verification logic
│   │   └── valkey/             # Redis/Valkey services
│   │       ├── mod.rs          # Valkey exports
│   │       ├── blacklist.rs    # Token blacklist
│   │       └── rate_limit.rs   # Rate limiting (future)
│   ├── models/                 # SeaORM database models
│   │   ├── mod.rs              # Model exports
│   │   ├── prelude.rs          # Common imports
│   │   ├── users.rs            # User entity
│   │   ├── refresh_tokens.rs  # Refresh token entity
│   │   ├── email_verifications.rs # Email verification entity
│   │   ├── o_auth_accounts.rs # OAuth account entity
│   │   └── sea_orm_active_enums.rs # Enums (Role, Status)
│   ├── utils/                  # Shared utilities
│   │   ├── mod.rs              # Utility exports
│   │   └── token.rs            # Token generation helpers
│   ├── openapi/                # OpenAPI documentation
│   │   └── mod.rs              # Schema generation
│   └── bin/                    # Binary executables
│       └── seed_admin.rs       # Admin seeding script
├── migration/                  # Database migrations
│   └── src/
│       └── m*.rs               # Migration files
├── Cargo.toml                  # Dependencies and metadata
├── Dockerfile                  # Production Docker image
└── .env.example                # Environment variable template
```

## Layered Architecture

### Layer 1: HTTP Handlers

**Responsibility**: HTTP request/response handling

**Location**: `src/handlers/`

**Key Principles**:
- Parse request (query params, JSON body, headers)
- Validate input (basic format validation)
- Call service layer for business logic
- Format response (JSON, status codes, cookies)
- Handle HTTP-specific errors

**Example Handler**:
```rust
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AuthError> {
    // 1. Validate input format (already done by serde)

    // 2. Call service layer
    let user = services::auth::register_user(
        &state.db,
        &payload.email,
        &payload.password,
    ).await?;

    // 3. Format response
    Ok(Json(RegisterResponse {
        id: user.id,
        email: user.email,
    }))
}
```

**Handlers by Module**:

- **auth.rs**: User authentication
  - `POST /api/auth/register` - Create new user account
  - `POST /api/auth/login` - Authenticate user, issue tokens
  - `POST /api/auth/refresh` - Refresh access token
  - `POST /api/auth/logout` - Revoke refresh token
  - `GET /api/auth/me` - Get current user info
  - `POST /api/auth/verify-email` - Verify email with code
  - `POST /api/auth/send-verification` - Resend verification email

- **admin.rs**: Administrative operations
  - `GET /api/admin/users` - List all users (paginated)
  - `GET /api/admin/users/:id` - Get user details
  - `PATCH /api/admin/users/:id/disable` - Disable user account
  - `PATCH /api/admin/users/:id/enable` - Enable user account
  - `GET /api/admin/stats` - System statistics

- **health.rs**: Health monitoring
  - `GET /health` - Health check endpoint

### Layer 2: Middleware

**Responsibility**: Cross-cutting concerns

**Location**: `src/middleware/`

**Execution Flow**:
```
Request → CORS → Tracing → Auth → Admin → Handler → Response
```

**Middleware Components**:

#### 1. CORS Middleware (Tower-HTTP)

**Configuration**:
- Allow origins: Development (`:2727`), Production (configured)
- Allow methods: GET, POST, PUT, DELETE, OPTIONS
- Allow headers: Authorization, Content-Type, Accept, Cookie
- Allow credentials: true (for HttpOnly cookies)

```rust
let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::predicate(|origin, _| {
        origin.to_str()
            .map(|s| s.ends_with(":2727"))
            .unwrap_or(false)
    }))
    .allow_methods([GET, POST, PUT, DELETE, OPTIONS])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT, COOKIE])
    .allow_credentials(true);
```

#### 2. Tracing Middleware (Tower-HTTP)

**Purpose**: Request logging and performance monitoring

**Logged Information**:
- HTTP method and path
- Status code
- Response time
- Request ID (for correlation)

#### 3. Auth Middleware

**Location**: `src/middleware/auth.rs`

**Purpose**: JWT token verification

**Flow**:
1. Extract `Authorization: Bearer <token>` header
2. Verify JWT signature with secret
3. Validate expiration time
4. Extract user ID and role from claims
5. Inject authenticated user into request extensions
6. Allow request to continue

**Error Cases**:
- Missing Authorization header → 401 Unauthorized
- Invalid token format → 401 Unauthorized
- Expired token → 401 Unauthorized
- Invalid signature → 401 Unauthorized

**Implementation**:
```rust
pub async fn auth_middleware(
    State(jwt_config): State<JwtConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract token from Authorization header
    let token = extract_token(&request)?;

    // Verify and decode JWT
    let claims = jwt_config.verify_token(&token)?;

    // Inject user info into request
    request.extensions_mut().insert(claims.user_id);
    request.extensions_mut().insert(claims.role);

    // Continue to handler
    Ok(next.run(request).await)
}
```

#### 4. Admin Middleware

**Location**: `src/middleware/admin.rs`

**Purpose**: Role-based authorization

**Flow**:
1. Retrieve user role from request extensions (set by auth middleware)
2. Check if role is `admin`
3. Allow or reject request

**Error Cases**:
- Missing user role (auth middleware not executed) → 401 Unauthorized
- Role is not admin → 403 Forbidden

**Chain Requirements**:
- Must be applied **after** auth middleware
- Auth middleware injects role, admin middleware reads it

### Layer 3: Services

**Responsibility**: Business logic and domain operations

**Location**: `src/services/`

**Key Principles**:
- Pure business logic (no HTTP concerns)
- Reusable across handlers
- Transaction management
- Error handling with domain-specific errors

#### Auth Services

**Location**: `src/services/auth/`

**Key Services**:

##### 1. JWT Service (`jwt.rs`)

**Responsibilities**:
- Create access tokens (15-minute expiry)
- Verify token signatures
- Extract claims from tokens

**Key Functions**:
```rust
impl JwtConfig {
    pub fn create_access_token(&self, user_id: Uuid, role: Role) -> Result<String>;
    pub fn verify_token(&self, token: &str) -> Result<Claims>;
}
```

**Token Claims**:
```rust
struct Claims {
    sub: Uuid,        // User ID (subject)
    role: Role,       // User role (user/admin)
    exp: i64,         // Expiration timestamp
    iat: i64,         // Issued at timestamp
}
```

##### 2. Password Service (`password.rs`)

**Responsibilities**:
- Hash passwords with Argon2id
- Verify password hashes

**Key Functions**:
```rust
pub fn hash_password(password: &str) -> Result<String>;
pub fn verify_password(password: &str, hash: &str) -> Result<bool>;
```

**Security Parameters**:
- Algorithm: Argon2id (hybrid, resistant to GPU/ASIC attacks)
- Memory cost: 64 MB
- Time cost: 4 iterations
- Parallelism: 1 thread
- Hash time: ~100ms (tuned for security/UX balance)

##### 3. Token Rotation Service (`token_rotation.rs`)

**Responsibilities**:
- Generate refresh tokens (cryptographically random)
- Hash refresh tokens (SHA-256 before storage)
- Rotate tokens (invalidate old, issue new)

**Key Functions**:
```rust
pub fn generate_refresh_token() -> String;
pub fn hash_token(token: &str) -> String;
pub async fn rotate_refresh_token(db: &DatabaseConnection, old_token: &str) -> Result<String>;
```

**Rotation Flow**:
1. Verify old refresh token exists in database
2. Mark old token as used/revoked
3. Generate new random token
4. Hash new token with SHA-256
5. Store new token hash in database
6. Return new token to client

#### Email Services

**Location**: `src/services/email/`

##### Verification Service (`verification.rs`)

**Responsibilities**:
- Generate 6-digit verification codes
- Send verification emails (mock in dev)
- Validate verification codes

**Key Functions**:
```rust
pub async fn send_verification_email(db: &DatabaseConnection, user_id: Uuid) -> Result<()>;
pub async fn verify_email(db: &DatabaseConnection, user_id: Uuid, code: &str) -> Result<()>;
```

**Mock Mode** (Development):
- Emails printed to console instead of sent
- Enables testing without email provider
- Controlled via `EMAIL_MOCK=true` env var

#### Valkey Services

**Location**: `src/services/valkey/`

##### Blacklist Service (`blacklist.rs`)

**Responsibilities**:
- Add revoked tokens to blacklist
- Check if token is blacklisted
- Automatic expiration (TTL)

**Key Functions**:
```rust
pub async fn blacklist_token(redis: &RedisConnection, jti: &str, expires_at: DateTime) -> Result<()>;
pub async fn is_token_blacklisted(redis: &RedisConnection, jti: &str) -> Result<bool>;
```

**Implementation**:
- Key format: `blacklist:{jti}`
- Value: Empty string (existence check only)
- TTL: Set to token expiry time (auto-cleanup)

### Layer 4: Data Models

**Responsibility**: Database entities and queries

**Location**: `src/models/`

**ORM**: SeaORM (async, type-safe)

#### User Model

**Table**: `users`

**Columns**:
- `id` (UUID, Primary Key)
- `email` (VARCHAR, UNIQUE, NOT NULL)
- `password_hash` (VARCHAR, NOT NULL)
- `role` (ENUM: user/admin)
- `status` (ENUM: active/disabled/pending_verification)
- `email_verified_at` (TIMESTAMP, NULL)
- `created_at` (TIMESTAMP, NOT NULL)
- `updated_at` (TIMESTAMP, NOT NULL)

**Relationships**:
- Has many `refresh_tokens`
- Has many `email_verifications`
- Has many `o_auth_accounts` (future)

#### Refresh Token Model

**Table**: `refresh_tokens`

**Columns**:
- `id` (UUID, Primary Key)
- `user_id` (UUID, Foreign Key → users.id)
- `token_hash` (VARCHAR, UNIQUE, NOT NULL)
- `expires_at` (TIMESTAMP, NOT NULL)
- `revoked_at` (TIMESTAMP, NULL)
- `used_at` (TIMESTAMP, NULL)
- `created_at` (TIMESTAMP, NOT NULL)

**Indexes**:
- `token_hash` (for fast lookup)
- `user_id` (for user token management)

#### Email Verification Model

**Table**: `email_verifications`

**Columns**:
- `id` (UUID, Primary Key)
- `user_id` (UUID, Foreign Key → users.id)
- `code` (VARCHAR(6), NOT NULL)
- `expires_at` (TIMESTAMP, NOT NULL)
- `created_at` (TIMESTAMP, NOT NULL)

**TTL**: 24 hours (configurable via env var)

## Request Flow Examples

### Authentication Flow

#### 1. User Registration

```
POST /api/auth/register
Body: { "email": "user@example.com", "password": "SecurePass123!" }

Flow:
1. Handler validates JSON format
2. Service checks if email already exists
3. Service hashes password with Argon2id
4. Service creates user in database (status: pending_verification)
5. Service generates 6-digit verification code
6. Service sends verification email (or prints to console)
7. Handler returns user ID and email

Response: 201 Created
{ "id": "uuid", "email": "user@example.com" }
```

#### 2. User Login

```
POST /api/auth/login
Body: { "email": "user@example.com", "password": "SecurePass123!" }

Flow:
1. Handler validates JSON format
2. Service queries user by email
3. Service verifies password hash
4. Service generates access token (JWT, 15min)
5. Service generates refresh token (random hash, 7 days)
6. Service stores refresh token hash in database
7. Handler sets refresh token in HttpOnly cookie
8. Handler returns access token in response body

Response: 200 OK
Set-Cookie: refresh_token=<token>; HttpOnly; Secure; SameSite=Lax; Max-Age=604800
{ "access_token": "eyJhbGc...", "user": { "id": "uuid", "email": "..." } }
```

#### 3. Refresh Access Token

```
POST /api/auth/refresh
Cookie: refresh_token=<token>

Flow:
1. Handler extracts refresh token from cookie
2. Service hashes token with SHA-256
3. Service queries database for token hash
4. Service validates token not expired/revoked
5. Service rotates token (invalidates old, generates new)
6. Service creates new access token
7. Handler sets new refresh token in cookie
8. Handler returns new access token

Response: 200 OK
Set-Cookie: refresh_token=<new_token>; HttpOnly; Secure; SameSite=Lax; Max-Age=604800
{ "access_token": "eyJhbGc..." }
```

#### 4. Protected Request

```
GET /api/auth/me
Authorization: Bearer eyJhbGc...

Flow:
1. CORS middleware validates origin
2. Tracing middleware logs request
3. Auth middleware extracts JWT from header
4. Auth middleware verifies JWT signature
5. Auth middleware extracts user_id from claims
6. Auth middleware injects user_id into request
7. Handler retrieves user_id from extensions
8. Service queries user by ID
9. Handler returns user data

Response: 200 OK
{ "id": "uuid", "email": "user@example.com", "role": "user" }
```

### Admin Flow

#### 1. List Users (Admin Only)

```
GET /api/admin/users?page=1&limit=20
Authorization: Bearer eyJhbGc...

Flow:
1. CORS middleware validates origin
2. Tracing middleware logs request
3. Auth middleware verifies JWT
4. Auth middleware extracts role from claims
5. Auth middleware injects role into request
6. Admin middleware checks role is "admin"
7. Handler extracts pagination params
8. Service queries users with pagination
9. Handler returns paginated user list

Response: 200 OK
{
  "users": [...],
  "total": 100,
  "page": 1,
  "limit": 20
}
```

## Error Handling

### Error Types

**Auth Errors** (`services/auth/error.rs`):
```rust
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    UserDisabled,
    EmailAlreadyExists,
    TokenExpired,
    TokenRevoked,
    InvalidToken,
}
```

**HTTP Status Mapping**:
- `InvalidCredentials` → 401 Unauthorized
- `UserNotFound` → 404 Not Found
- `UserDisabled` → 403 Forbidden
- `EmailAlreadyExists` → 409 Conflict
- `TokenExpired` → 401 Unauthorized
- `TokenRevoked` → 401 Unauthorized
- `InvalidToken` → 401 Unauthorized

### Error Response Format

```json
{
  "error": "InvalidCredentials",
  "message": "Email or password is incorrect",
  "request_id": "uuid"
}
```

## Configuration

### Environment Variables

**Required**:
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret for JWT signing (min 32 chars)

**Optional**:
- `REDIS_URL`: Redis connection string (default: `redis://localhost:6379`)
- `PORT`: Server port (default: 3000)
- `RUST_LOG`: Log level (default: `info`)
- `FRONTEND_URL`: Frontend URL for CORS (default: `http://localhost:2727`)
- `EMAIL_MOCK`: Mock email sending (default: `true`)
- `EMAIL_VERIFICATION_EXPIRY_SECONDS`: Verification code TTL (default: 86400)

### JWT Configuration

**Loaded from Environment**:
```rust
pub struct JwtConfig {
    secret: String,              // JWT_SECRET
    access_token_expiry: i64,    // 15 minutes (hardcoded)
    refresh_token_expiry: i64,   // 7 days (hardcoded)
}
```

## Testing Strategy

### Unit Tests

**Location**: Inline with source files (`#[cfg(test)]` modules)

**Coverage**:
- Password hashing and verification
- Token generation and validation
- JWT creation and verification
- Error handling

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "SecurePass123!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("WrongPass", &hash).unwrap());
    }
}
```

### Integration Tests

**Location**: `backend/tests/` (future)

**Coverage**:
- API endpoint testing
- Database transactions
- Authentication flows
- Admin operations

## Performance Optimization

### Async Runtime

**Tokio Configuration**:
- Multi-threaded runtime
- Work-stealing scheduler
- Thread pool size = CPU cores

### Database Connection Pool

**SeaORM Configuration**:
- Min connections: 5
- Max connections: 20
- Connection timeout: 30s
- Idle timeout: 10 minutes

### Caching Strategy

**Redis Usage**:
- Token blacklist (avoid DB query for revoked tokens)
- Rate limiting (future)
- Session data (future)

### Query Optimization

**Best Practices**:
- Use indexes on frequently queried columns
- Avoid N+1 queries (use eager loading)
- Limit result sets with pagination
- Use database transactions for consistency

## Security Best Practices

### Input Validation

- JSON schema validation (via Serde)
- Email format validation (regex)
- Password strength requirements (future)
- SQL injection prevention (parameterized queries)

### Output Sanitization

- No HTML rendering (JSON API only)
- Error messages sanitized (no sensitive info)
- Stack traces disabled in production

### Token Security

- Short-lived access tokens (15 min)
- HttpOnly cookies (prevent XSS theft)
- Secure flag (HTTPS only in production)
- SameSite flag (prevent CSRF)
- Token rotation (prevent replay attacks)

## Deployment Considerations

### Build Optimization

**Release Profile**:
```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"          # Link-time optimization
codegen-units = 1    # Optimize for speed, not compile time
strip = true         # Strip symbols for smaller binary
```

### Docker Image

**Multi-stage Build**:
1. Builder: Compile Rust binary (nightly toolchain)
2. Runtime: Debian Slim with binary (minimal image)

**Image Size**: ~150MB (binary + runtime dependencies)

### Health Check

**Kubernetes/Docker**:
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 40
  periodSeconds: 30
```

## Future Enhancements

### Planned Features

- OAuth 2.0 integration (Google, GitHub)
- Two-factor authentication (TOTP)
- Rate limiting (per-user, per-IP, per-endpoint)
- Audit logging (user actions, admin actions)
- API versioning (v1, v2)
- WebSocket support (real-time updates)

### Performance Improvements

- Query caching (Redis)
- Connection pooling optimization
- Database read replicas
- CDN for static assets
- Gzip/Brotli compression

### Security Enhancements

- Content Security Policy headers
- Rate limiting on auth endpoints
- Account lockout after failed attempts
- IP geolocation and blocklist
- Security headers (HSTS, X-Frame-Options)

## References

- [Axum Documentation](https://docs.rs/axum)
- [Tokio Documentation](https://docs.rs/tokio)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM)
- [JWT Best Practices](https://datatracker.ietf.org/doc/html/rfc8725)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
