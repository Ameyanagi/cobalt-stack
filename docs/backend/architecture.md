# Backend Architecture

## Table of Contents
- [Overview](#overview)
- [Domain-Driven Design Principles](#domain-driven-design-principles)
- [Layered Architecture](#layered-architecture)
- [Dependency Injection](#dependency-injection)
- [Code Organization](#code-organization)
- [Layer Interactions](#layer-interactions)
- [Best Practices](#best-practices)

## Overview

The Cobalt Stack backend follows **Domain-Driven Design (DDD)** principles with a clean layered architecture. The codebase separates concerns into distinct layers, each with specific responsibilities and clear boundaries.

**Technology Stack:**
- **Framework:** Axum (high-performance async web framework)
- **Database:** PostgreSQL with SeaORM
- **Cache/Session:** Valkey (Redis-compatible)
- **Authentication:** JWT with token rotation
- **API Documentation:** OpenAPI (utoipa)

## Domain-Driven Design Principles

The backend architecture emphasizes:

1. **Separation of Concerns**: Each layer has a single, well-defined responsibility
2. **Dependency Inversion**: High-level modules don't depend on low-level modules; both depend on abstractions
3. **Domain-Centric Design**: Business logic is isolated from infrastructure concerns
4. **Explicit Error Handling**: Domain-specific errors with clear mappings to HTTP responses

## Layered Architecture

```text
┌─────────────────────────────────────────┐
│          HTTP Layer (Axum)              │
│    (Request/Response, Routing, CORS)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│        Middleware Layer                 │
│  (Auth, Admin, Rate Limiting, Logging)  │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Handler Layer                   │
│    (HTTP Concerns, Validation, DTOs)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Service Layer                   │
│  (Business Logic, Domain Operations)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│          Model Layer                    │
│   (Database Entities, Domain Models)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       Database (PostgreSQL)             │
│          + Valkey Cache                 │
└─────────────────────────────────────────┘
```

### Layer Responsibilities

#### 1. Handler Layer (`src/handlers/`)

**Responsibilities:**
- HTTP request/response handling
- Input validation and sanitization
- Request/response transformation (DTOs)
- HTTP status code mapping
- OpenAPI documentation

**Should NOT contain:**
- Business logic
- Database queries
- Complex algorithms

**Example:**
```rust
// src/handlers/auth.rs
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // 1. Validate input (handler responsibility)
    req.validate()?;

    // 2. Call service layer for business logic
    let password_hash = hash_password(&req.password)?;

    // 3. Database operations via service layer
    let user = create_user(&state.db, req.username, req.email, password_hash).await?;

    // 4. Return HTTP response with appropriate status
    Ok((StatusCode::OK, Json(AuthResponse { ... })))
}
```

#### 2. Service Layer (`src/services/`)

**Responsibilities:**
- Business logic and domain rules
- Orchestration of complex operations
- Transaction coordination
- Domain-specific error handling

**Should NOT contain:**
- HTTP concerns (StatusCode, headers)
- Request/response parsing
- Direct access to HTTP layer

**Example:**
```rust
// src/services/auth/jwt.rs
pub fn create_access_token(
    user_id: Uuid,
    username: String,
    config: &JwtConfig,
) -> Result<String> {
    let expiry = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(config.access_token_expiry_minutes))
        .ok_or(AuthError::InternalError)?;

    let claims = AccessTokenClaims {
        sub: user_id,
        username,
        exp: expiry.timestamp(),
        iat: Utc::now().timestamp(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(config.secret.as_bytes()))
        .map_err(|_| AuthError::JwtEncodingError)
}
```

#### 3. Model Layer (`src/models/`)

**Responsibilities:**
- Database entity definitions
- Data structure and relationships
- Column constraints and types
- Entity lifecycle hooks

**Should NOT contain:**
- Business logic
- HTTP concerns
- Complex operations

**Example:**
```rust
// src/models/users.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub username: String,

    #[sea_orm(unique)]
    pub email: String,

    pub password_hash: Option<String>,
    pub email_verified: bool,
    pub role: UserRole,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
```

#### 4. Middleware Layer (`src/middleware/`)

**Responsibilities:**
- Cross-cutting concerns
- Authentication and authorization
- Request logging
- Rate limiting
- CORS handling

**Example:**
```rust
// src/middleware/auth.rs
pub async fn auth_middleware(
    State(jwt_config): State<JwtConfig>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract and verify JWT token
    let token = extract_token_from_header(req.headers())?;
    let claims = verify_access_token(&token, &jwt_config)?;

    // Inject user claims into request
    let auth_user = AuthUser {
        user_id: claims.sub,
        username: claims.username,
    };
    req.extensions_mut().insert(auth_user);

    // Continue to next middleware/handler
    Ok(next.run(req).await)
}
```

## Dependency Injection

The backend uses **state-based dependency injection** via Axum's `State` extractor.

### Application State Pattern

```rust
// Shared state with database connection and configuration
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub jwt_config: JwtConfig,
}

// Router with injected state
let app = Router::new()
    .route("/api/auth/login", post(handlers::auth::login))
    .with_state(state);

// Handler receives state via extraction
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Use state.db and state.jwt_config
}
```

### Benefits of This Pattern

1. **Testability**: Easy to inject mock dependencies
2. **Flexibility**: Configuration changes don't require code changes
3. **Type Safety**: Compile-time verification of dependencies
4. **Performance**: Arc-wrapped state is cheap to clone

## Code Organization

```text
backend/
├── src/
│   ├── handlers/          # HTTP request/response handling
│   │   ├── mod.rs
│   │   ├── auth.rs        # Authentication endpoints
│   │   ├── admin.rs       # Admin management endpoints
│   │   └── health.rs      # Health check endpoint
│   │
│   ├── services/          # Business logic
│   │   ├── mod.rs
│   │   ├── auth/          # Authentication services
│   │   │   ├── mod.rs
│   │   │   ├── jwt.rs     # JWT token operations
│   │   │   ├── password.rs # Password hashing
│   │   │   ├── token_rotation.rs # Token rotation logic
│   │   │   └── error.rs   # Domain errors
│   │   ├── email/         # Email services
│   │   └── valkey/        # Cache/session services
│   │
│   ├── models/            # Database entities
│   │   ├── mod.rs
│   │   ├── users.rs       # User entity
│   │   ├── refresh_tokens.rs
│   │   └── prelude.rs     # Re-exports
│   │
│   ├── middleware/        # Request middleware
│   │   ├── mod.rs
│   │   ├── auth.rs        # Authentication middleware
│   │   └── admin.rs       # Authorization middleware
│   │
│   ├── config/            # Configuration management
│   │   └── mod.rs
│   │
│   ├── utils/             # Shared utilities
│   │   ├── mod.rs
│   │   └── token.rs
│   │
│   ├── openapi/           # API documentation
│   │   └── mod.rs
│   │
│   ├── lib.rs             # Library root
│   └── main.rs            # Application entry point
│
├── migration/             # Database migrations
│   └── src/
│       └── m*.rs          # Migration files
│
└── Cargo.toml             # Dependencies
```

## Layer Interactions

### Request Flow Example: User Login

```text
1. HTTP Request
   ↓
2. Middleware (CORS, Logging)
   ↓
3. Handler Layer (handlers::auth::login)
   - Validate LoginRequest DTO
   - Extract credentials
   ↓
4. Service Layer (services::auth)
   - Verify password hash
   - Generate JWT tokens
   ↓
5. Model Layer (models::users)
   - Query user by username
   - Fetch user entity
   ↓
6. Database (PostgreSQL)
   - Execute query
   - Return user record
   ↓
7. Response Path (reverse)
   - Service returns tokens
   - Handler creates AuthResponse
   - Middleware adds cookies
   - HTTP response sent
```

### Code Example: Complete Flow

```rust
// 1. Handler receives request
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    req.validate()?;

    // 2. Query model layer
    let user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // 3. Call service layer for business logic
    let password_hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
    verify_password(&req.password, &password_hash)?;

    // 4. Generate tokens via service
    let access_token = create_access_token(user.id, user.username, &state.jwt_config)?;
    let (refresh_token, jti) = create_refresh_token(user.id, &state.jwt_config)?;

    // 5. Store token in database
    store_refresh_token(state.db.as_ref(), user.id, &refresh_token, jti, 7).await?;

    // 6. Return HTTP response
    Ok((StatusCode::OK, Json(AuthResponse { access_token, ... })))
}
```

## Best Practices

### 1. Error Handling

**Use domain-specific errors:**
```rust
// services/auth/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid token")]
    InvalidToken,
}

// Map to HTTP status codes
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        (status, Json(ErrorResponse { error: message.to_string() })).into_response()
    }
}
```

### 2. DTOs for Data Transfer

**Keep domain models separate from HTTP DTOs:**
```rust
// Handler-specific DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

// Domain model (from database)
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    // ... more fields
}

// Response DTO (subset of fields)
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
}
```

### 3. Separation of Concerns

**Handlers should be thin:**
```rust
// ❌ Bad: Business logic in handler
pub async fn login(req: LoginRequest) -> Result<Response> {
    let hash = Argon2::new().hash_password(req.password)?; // Business logic!
    let token = encode(&claims, secret)?;                  // Business logic!
    // ...
}

// ✅ Good: Delegate to service layer
pub async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>)
    -> Result<impl IntoResponse, AuthError>
{
    req.validate()?;
    let user = find_user_by_username(&state.db, &req.username).await?;
    verify_password(&req.password, &user.password_hash)?;
    let tokens = generate_tokens(user.id, user.username, &state.jwt_config)?;
    Ok((StatusCode::OK, Json(tokens)))
}
```

### 4. Consistent Patterns

**Follow established patterns across modules:**
- All handlers return `Result<impl IntoResponse, DomainError>`
- All services return `Result<T, DomainError>`
- All validation happens in handler layer
- All business logic lives in service layer

### 5. Documentation

**Use doc comments extensively:**
```rust
/// POST /api/auth/login - Authenticate user with credentials
///
/// Verifies username and password, then generates access and refresh tokens.
/// Refresh token is set as HttpOnly cookie.
///
/// # Security
/// - Rate limited to 5 attempts per 15 minutes
/// - Password verified using Argon2
/// - Tokens signed with HS256
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    )
)]
pub async fn login(/* ... */) { /* ... */ }
```

## Related Documentation

- [API Handlers](./api-handlers.md) - Detailed handler layer documentation
- [Services](./services.md) - Service layer patterns and examples
- [Models](./models.md) - Database entities and relationships
- [Database](./database.md) - SeaORM and migrations
- [Testing](./testing.md) - Testing strategies and patterns
