# API Handlers

## Table of Contents
- [Overview](#overview)
- [Handler Responsibilities](#handler-responsibilities)
- [Request/Response Patterns](#requestresponse-patterns)
- [Error Handling](#error-handling)
- [Middleware Integration](#middleware-integration)
- [Input Validation](#input-validation)
- [Real Examples](#real-examples)

## Overview

The **Handler Layer** is the boundary between HTTP and your application's business logic. Handlers are responsible for:
- Receiving HTTP requests and extracting data
- Validating input
- Calling service layer for business operations
- Transforming results into HTTP responses

Handlers **should NOT** contain business logic—that belongs in the service layer.

## Handler Responsibilities

### What Handlers SHOULD Do

✅ **HTTP Concerns**
- Parse request bodies (JSON, form data)
- Extract path parameters and query strings
- Validate input data
- Set HTTP status codes
- Add response headers
- Handle cookies

✅ **Orchestration**
- Call service layer functions
- Coordinate multiple service calls
- Transform domain models to DTOs

✅ **Error Mapping**
- Convert domain errors to HTTP responses
- Set appropriate status codes

### What Handlers SHOULD NOT Do

❌ **Business Logic**
- Password hashing
- Token generation
- Complex validation rules
- Database queries directly

❌ **Infrastructure Concerns**
- Database connection management
- Cache operations
- Email sending

## Request/Response Patterns

### Basic Handler Structure

```rust
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use crate::services::auth::AuthError;

pub async fn handler_name(
    State(state): State<AppState>,           // Application state
    Json(req): Json<RequestDTO>,             // Request body
) -> std::result::Result<impl IntoResponse, AuthError> {
    // 1. Validate input
    req.validate()?;

    // 2. Call service layer
    let result = service_function(&state.db, req.data).await?;

    // 3. Transform to response DTO
    let response = ResponseDTO::from(result);

    // 4. Return HTTP response
    Ok((StatusCode::OK, Json(response)))
}
```

### State Extraction

Handlers receive shared application state via Axum's `State` extractor:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub jwt_config: JwtConfig,
}

// In handler
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Access database: state.db
    // Access config: state.jwt_config
}
```

### JSON Request Bodies

Use `Json` extractor to automatically parse request bodies:

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "alice")]
    pub username: String,

    #[schema(example = "alice@example.com")]
    pub email: String,

    #[schema(example = "SecurePass123!")]
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,  // Automatic JSON parsing
) -> Result<impl IntoResponse, AuthError> {
    // req is deserialized RegisterRequest
}
```

### Path Parameters

Extract path parameters using `Path` extractor:

```rust
use axum::extract::Path;
use uuid::Uuid;

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let user = Users::find_by_id(user_id)
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::UserNotFound)?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}
```

### Cookies

Handle cookies with `axum_extra::extract::cookie`:

```rust
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum::http::header;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // ... authentication logic ...

    // Create HttpOnly cookie for refresh token
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(7))
        .build();

    // Return response with cookie
    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}

// Reading cookies
pub async fn refresh_token(
    State(state): State<AppState>,
    jar: axum_extra::extract::CookieJar,
) -> Result<impl IntoResponse, AuthError> {
    let refresh_token = jar
        .get("refresh_token")
        .ok_or(AuthError::InvalidToken)?
        .value()
        .to_string();

    // ... token refresh logic ...
}
```

## Error Handling

### Domain Error Types

Define domain-specific errors in the service layer:

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

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error")]
    InternalError,
}
```

### HTTP Response Mapping

Implement `IntoResponse` to map domain errors to HTTP responses:

```rust
use axum::{response::IntoResponse, http::StatusCode, Json};

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AuthError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let body = Json(ErrorResponse {
            error: message.to_string(),
        });

        (status, body).into_response()
    }
}
```

### Handler Error Propagation

Use `?` operator to propagate errors up to Axum's error handling:

```rust
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // Validation error automatically converted to HTTP response
    req.validate()?;

    // Database error automatically converted
    let user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?;

    // Custom error
    if user.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // ... rest of handler
}
```

## Middleware Integration

### Protected Routes

Use middleware to protect routes requiring authentication:

```rust
use axum::middleware as axum_middleware;

// Public routes (no auth required)
let auth_public_routes = Router::new()
    .route("/api/auth/register", post(handlers::auth::register))
    .route("/api/auth/login", post(handlers::auth::login))
    .with_state(state.clone());

// Protected routes (auth required)
let auth_protected_routes = Router::new()
    .route("/api/auth/me", get(handlers::auth::get_current_user))
    .route("/api/auth/logout", post(handlers::auth::logout))
    .layer(axum_middleware::from_fn_with_state(
        jwt_config.clone(),
        middleware::auth::auth_middleware,  // Inject auth middleware
    ))
    .with_state(state.clone());
```

### Extracting User from Middleware

Access authenticated user information injected by middleware:

```rust
use crate::middleware::auth::AuthUser;

pub async fn get_current_user(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<impl IntoResponse, AuthError> {
    // Extract AuthUser from request extensions (set by middleware)
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AuthError::InvalidToken)?;

    // Fetch full user information
    let user = Users::find_by_id(auth_user.user_id)
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::UserNotFound)?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}
```

### Admin Authorization

Stack multiple middleware for role-based access:

```rust
let admin_routes = Router::new()
    .route("/api/admin/users", get(handlers::admin::list_users))
    .layer(axum_middleware::from_fn_with_state(
        state.db.clone(),
        middleware::admin::admin_middleware,  // Check admin role
    ))
    .layer(axum_middleware::from_fn_with_state(
        jwt_config,
        middleware::auth::auth_middleware,    // Verify authentication first
    ))
    .with_state(admin_state);
```

## Input Validation

### Request DTO Validation

Implement validation logic on request DTOs:

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<()> {
        // Username validation
        if self.username.is_empty() {
            return Err(AuthError::InvalidInput("Username cannot be empty".to_string()));
        }
        if self.username.len() < 3 || self.username.len() > 50 {
            return Err(AuthError::InvalidInput(
                "Username must be between 3 and 50 characters".to_string(),
            ));
        }

        // Email validation
        if !self.email.contains('@') {
            return Err(AuthError::InvalidInput("Invalid email format".to_string()));
        }

        // Password validation
        if self.password.len() < 8 {
            return Err(AuthError::InvalidInput(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        Ok(())
    }
}
```

### Handler Validation Flow

```rust
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate immediately in handler
    req.validate().map_err(|e| {
        e.downcast::<AuthError>()
            .unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // Proceed with business logic...
}
```

## Real Examples

### Example 1: User Registration

```rust
/// POST /api/auth/register - Register a new user
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // 1. Validate input
    req.validate().map_err(|e| {
        e.downcast::<AuthError>().unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // 2. Check if username already exists
    let existing_user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?;

    if existing_user.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // 3. Check if email already exists
    let existing_email = Users::find()
        .filter(users::Column::Email.eq(&req.email))
        .one(state.db.as_ref())
        .await?;

    if existing_email.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // 4. Hash password (service layer)
    let password_hash = hash_password(&req.password)
        .map_err(|_| AuthError::PasswordHashError)?;

    // 5. Create user
    let user = users::ActiveModel {
        username: Set(req.username.clone()),
        email: Set(req.email.clone()),
        password_hash: Set(Some(password_hash)),
        email_verified: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let user = user.insert(state.db.as_ref()).await?;

    // 6. Send verification email
    let token = create_verification_token(state.db.as_ref(), user.id).await?;
    let email_sender = MockEmailSender;
    email_sender.send_verification_email(&user.email, &token)?;

    // 7. Generate JWT tokens (service layer)
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)?;
    let (refresh_token, refresh_jti) = create_refresh_token(user.id, &state.jwt_config)?;

    // 8. Store refresh token
    store_refresh_token(
        state.db.as_ref(),
        user.id,
        &refresh_token,
        refresh_jti,
        state.jwt_config.refresh_token_expiry_days,
    ).await?;

    // 9. Create HttpOnly cookie
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(state.jwt_config.refresh_token_expiry_days))
        .build();

    // 10. Return response with cookie
    let response = AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}
```

### Example 2: User Login

```rust
/// POST /api/auth/login - Login with username/password
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // 1. Validate input
    req.validate().map_err(|e| {
        e.downcast::<AuthError>().unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // 2. Find user by username
    let user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // 3. Verify password (service layer)
    let password_hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
    let is_valid = verify_password(&req.password, &password_hash)
        .map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // 4. Generate tokens (service layer)
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)?;
    let (refresh_token, refresh_jti) = create_refresh_token(user.id, &state.jwt_config)?;

    // 5. Store refresh token
    store_refresh_token(
        state.db.as_ref(),
        user.id,
        &refresh_token,
        refresh_jti,
        state.jwt_config.refresh_token_expiry_days,
    ).await?;

    // 6. Create HttpOnly cookie
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(state.jwt_config.refresh_token_expiry_days))
        .build();

    // 7. Return response
    let response = AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}
```

### Example 3: Protected Endpoint

```rust
/// GET /api/auth/me - Get current user information
#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "User information", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "Authentication",
    security(("bearer_auth" = []))
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    use crate::middleware::auth::AuthUser;

    // Extract AuthUser from request extensions (injected by middleware)
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AuthError::InvalidToken)?;

    // Fetch full user information from database
    let user = Users::find_by_id(auth_user.user_id)
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // Return user response (exclude sensitive fields)
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        email_verified: user.email_verified,
        role: user.role,
    };

    Ok((StatusCode::OK, Json(response)))
}
```

### Example 4: Token Refresh

```rust
/// POST /api/auth/refresh - Refresh access token using refresh token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    jar: axum_extra::extract::CookieJar,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // 1. Extract refresh token from cookie
    let old_refresh_token = jar
        .get("refresh_token")
        .ok_or(AuthError::InvalidToken)?
        .value()
        .to_string();

    // 2. Verify JWT signature and expiry (service layer)
    let claims = verify_refresh_token(&old_refresh_token, &state.jwt_config)?;

    // 3. Validate token in database (service layer)
    let user_id = validate_refresh_token(state.db.as_ref(), &old_refresh_token, claims.jti).await?;

    // 4. Get username
    let username = {
        let user = Users::find_by_id(user_id)
            .one(state.db.as_ref())
            .await?
            .ok_or(AuthError::UserNotFound)?;
        user.username
    };

    // 5. Generate new tokens (service layer)
    let new_access_token = create_access_token(user_id, username, &state.jwt_config)?;
    let (new_refresh_token, new_refresh_jti) = create_refresh_token(user_id, &state.jwt_config)?;

    // 6. Rotate refresh token (revoke old, store new)
    rotate_refresh_token(
        state.db.as_ref(),
        claims.jti,
        &new_refresh_token,
        new_refresh_jti,
        user_id,
        state.jwt_config.refresh_token_expiry_days,
    ).await?;

    // 7. Create new HttpOnly cookie
    let cookie = Cookie::build(("refresh_token", new_refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(state.jwt_config.refresh_token_expiry_days))
        .build();

    // 8. Return new access token
    let response = AuthResponse {
        access_token: new_access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}
```

## Related Documentation

- [Architecture](./architecture.md) - Overall backend architecture
- [Services](./services.md) - Service layer patterns
- [Models](./models.md) - Database entities
- [Testing](./testing.md) - Testing handlers
