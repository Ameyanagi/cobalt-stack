# Service Layer

## Table of Contents
- [Overview](#overview)
- [Service Architecture](#service-architecture)
- [Business Logic Organization](#business-logic-organization)
- [Authentication Services](#authentication-services)
- [Email Services](#email-services)
- [Cache Services](#cache-services)
- [Service Patterns](#service-patterns)
- [Testing Services](#testing-services)

## Overview

The **Service Layer** contains business logic and domain operations. It sits between handlers (HTTP layer) and models (database layer), providing:
- Pure business logic without HTTP concerns
- Reusable operations across multiple handlers
- Domain-specific error handling
- Transaction coordination
- Integration with external services

**Key Principle**: Services should be testable in isolation without HTTP dependencies.

## Service Architecture

### Service Organization

```text
services/
├── auth/                    # Authentication services
│   ├── mod.rs              # Public API and re-exports
│   ├── error.rs            # Domain errors
│   ├── jwt.rs              # JWT token operations
│   ├── password.rs         # Password hashing/verification
│   └── token_rotation.rs   # Refresh token rotation
│
├── email/                   # Email services
│   ├── mod.rs
│   └── verification.rs     # Email verification
│
└── valkey/                  # Cache/session services
    ├── mod.rs
    ├── blacklist.rs        # Token blacklisting
    └── rate_limit.rs       # Rate limiting
```

### Service Layer Responsibilities

✅ **Business Logic**
- Domain rules and validation
- Complex calculations
- Workflow orchestration

✅ **Data Operations**
- Database queries and mutations
- Transaction management
- Data transformation

✅ **Integration**
- External service calls
- Cache operations
- Third-party APIs

❌ **HTTP Concerns**
- Request/response parsing
- Status code selection
- Header management

## Business Logic Organization

### Organizing by Domain

Group related business logic into domain-specific modules:

```rust
// services/auth/mod.rs - Authentication domain
pub mod error;
pub mod jwt;
pub mod password;
pub mod token_rotation;

// Re-export commonly used items
pub use error::{AuthError, Result};
pub use jwt::{create_access_token, verify_access_token, JwtConfig};
pub use password::{hash_password, verify_password};
pub use token_rotation::{rotate_refresh_token, validate_refresh_token};
```

### Pure Functions

Services should be pure functions when possible:

```rust
// ✅ Pure function - same inputs always produce same outputs
pub fn create_access_token(
    user_id: Uuid,
    username: String,
    config: &JwtConfig,
) -> Result<String> {
    let claims = AccessTokenClaims {
        sub: user_id,
        username,
        exp: (Utc::now() + Duration::minutes(config.access_token_expiry_minutes)).timestamp(),
        iat: Utc::now().timestamp(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(config.secret.as_bytes()))
        .map_err(|_| AuthError::JwtEncodingError)
}
```

### Async Operations

Use async functions for I/O operations (database, network):

```rust
// Async function for database operations
pub async fn store_refresh_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    token: &str,
    jti: Uuid,
    expiry_days: i64,
) -> Result<()> {
    let token_hash = hash_token(token);
    let expires_at = Utc::now() + Duration::days(expiry_days);

    let refresh_token = refresh_tokens::ActiveModel {
        id: Set(jti),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at.into()),
        revoked_at: Set(None),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    };

    refresh_token.insert(db).await?;
    Ok(())
}
```

## Authentication Services

### JWT Token Management

**Location**: `services/auth/jwt.rs`

#### Token Creation

```rust
/// Create a short-lived access token (30 minutes default)
pub fn create_access_token(
    user_id: Uuid,
    username: String,
    config: &JwtConfig,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::minutes(config.access_token_expiry_minutes);

    let claims = AccessTokenClaims {
        sub: user_id,
        username,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| AuthError::JwtEncodingError)
}

/// Create a long-lived refresh token (7 days default)
/// Returns token string and unique token ID (jti)
pub fn create_refresh_token(
    user_id: Uuid,
    config: &JwtConfig,
) -> Result<(String, Uuid)> {
    let now = Utc::now();
    let exp = now + Duration::days(config.refresh_token_expiry_days);
    let jti = Uuid::new_v4();

    let claims = RefreshTokenClaims {
        sub: user_id,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| AuthError::JwtEncodingError)?;

    Ok((token, jti))
}
```

#### Token Verification

```rust
/// Verify and decode an access token
pub fn verify_access_token(
    token: &str,
    config: &JwtConfig,
) -> Result<AccessTokenClaims> {
    let token_data = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    })?;

    Ok(token_data.claims)
}

/// Verify and decode a refresh token
pub fn verify_refresh_token(
    token: &str,
    config: &JwtConfig,
) -> Result<RefreshTokenClaims> {
    let token_data = decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    })?;

    Ok(token_data.claims)
}
```

### Password Management

**Location**: `services/auth/password.rs`

#### Password Hashing

```rust
/// Hash password using Argon2id with OWASP-recommended parameters
///
/// Security:
/// - Algorithm: Argon2id (hybrid mode)
/// - Memory cost: 19 MiB
/// - Time cost: 2 iterations
/// - Parallelism: 1 thread
/// - Unique salt per hash
pub fn hash_password(password: &str) -> Result<String> {
    // Validate password strength
    validate_password_strength(password)?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError::PasswordHashError)?;

    Ok(password_hash.to_string())
}

fn validate_password_strength(password: &str) -> Result<()> {
    let len = password.len();

    if len < 8 {
        return Err(AuthError::WeakPassword.into());
    }

    if len > 128 {
        return Err(AuthError::WeakPassword.into());
    }

    Ok(())
}
```

#### Password Verification

```rust
/// Verify password against Argon2id hash using constant-time comparison
///
/// Returns:
/// - Ok(true) if password matches
/// - Ok(false) if password doesn't match
/// - Err if hash is invalid
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AuthError::InvalidCredentials)?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}
```

### Token Rotation

**Location**: `services/auth/token_rotation.rs`

#### Storing Refresh Tokens

```rust
use sha2::{Digest, Sha256};

/// Hash token using SHA-256 before storing in database
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Store refresh token in database with hash
pub async fn store_refresh_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    token: &str,
    jti: Uuid,
    expiry_days: i64,
) -> Result<()> {
    let token_hash = hash_token(token);
    let expires_at = Utc::now() + Duration::days(expiry_days);

    let refresh_token = refresh_tokens::ActiveModel {
        id: Set(jti),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at.into()),
        revoked_at: Set(None),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    };

    refresh_token.insert(db).await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(())
}
```

#### Validating Refresh Tokens

```rust
/// Validate refresh token against database
///
/// Checks:
/// 1. Token hash matches database record
/// 2. Token is not revoked
/// 3. Token has not expired
pub async fn validate_refresh_token(
    db: &DatabaseConnection,
    token: &str,
    jti: Uuid,
) -> Result<Uuid> {
    let token_hash = hash_token(token);

    let stored_token = RefreshTokens::find_by_id(jti)
        .one(db)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidToken)?;

    // Check if token is revoked
    if stored_token.revoked_at.is_some() {
        return Err(AuthError::InvalidToken);
    }

    // Check if token is expired
    let now = Utc::now();
    if stored_token.expires_at < now.into() {
        return Err(AuthError::TokenExpired);
    }

    // Check if hash matches
    if stored_token.token_hash != token_hash {
        return Err(AuthError::InvalidToken);
    }

    Ok(stored_token.user_id)
}
```

#### Token Rotation

```rust
/// Rotate refresh token (revoke old, create new)
///
/// This prevents token theft by invalidating the old token
/// as soon as it's used to get a new one.
pub async fn rotate_refresh_token(
    db: &DatabaseConnection,
    old_jti: Uuid,
    new_token: &str,
    new_jti: Uuid,
    user_id: Uuid,
    expiry_days: i64,
) -> Result<()> {
    // Use transaction for atomicity
    let txn = db.begin().await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Revoke old token
    let old_token = RefreshTokens::find_by_id(old_jti)
        .one(&txn)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidToken)?;

    let mut old_token: refresh_tokens::ActiveModel = old_token.into();
    old_token.revoked_at = Set(Some(Utc::now().into()));
    old_token.update(&txn).await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Store new token
    let token_hash = hash_token(new_token);
    let expires_at = Utc::now() + Duration::days(expiry_days);

    let new_refresh_token = refresh_tokens::ActiveModel {
        id: Set(new_jti),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at.into()),
        revoked_at: Set(None),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    };

    new_refresh_token.insert(&txn).await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Commit transaction
    txn.commit().await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(())
}
```

#### Token Revocation

```rust
/// Revoke a refresh token (for logout)
pub async fn revoke_refresh_token(
    db: &DatabaseConnection,
    jti: Uuid,
) -> Result<()> {
    let token = RefreshTokens::find_by_id(jti)
        .one(db)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidToken)?;

    let mut token: refresh_tokens::ActiveModel = token.into();
    token.revoked_at = Set(Some(Utc::now().into()));

    token.update(db).await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Revoke all user's refresh tokens (for security events)
pub async fn revoke_all_user_tokens(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<()> {
    let now = Utc::now();

    RefreshTokens::update_many()
        .col_expr(refresh_tokens::Column::RevokedAt, Expr::value(now))
        .filter(refresh_tokens::Column::UserId.eq(user_id))
        .filter(refresh_tokens::Column::RevokedAt.is_null())
        .exec(db)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(())
}
```

## Email Services

**Location**: `services/email/verification.rs`

### Email Verification

```rust
/// Create email verification token
pub async fn create_verification_token(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<String> {
    // Generate secure random token
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = hex::encode(token_bytes);

    let expires_at = Utc::now() + Duration::hours(24);

    let verification = email_verifications::ActiveModel {
        user_id: Set(user_id),
        token: Set(token.clone()),
        expires_at: Set(expires_at.into()),
        verified_at: Set(None),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    };

    verification.insert(db).await?;
    Ok(token)
}

/// Verify email token and mark user as verified
pub async fn verify_email_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<Uuid> {
    // Find verification record
    let verification = EmailVerifications::find()
        .filter(email_verifications::Column::Token.eq(token))
        .one(db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    // Check if already verified
    if verification.verified_at.is_some() {
        return Err(AuthError::InvalidInput("Email already verified".to_string()));
    }

    // Check expiration
    if verification.expires_at < Utc::now().into() {
        return Err(AuthError::TokenExpired);
    }

    let txn = db.begin().await?;

    // Mark verification as used
    let mut verification: email_verifications::ActiveModel = verification.into();
    verification.verified_at = Set(Some(Utc::now().into()));
    verification.update(&txn).await?;

    // Update user email_verified flag
    let user = Users::find_by_id(verification.user_id.clone().unwrap())
        .one(&txn)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let mut user: users::ActiveModel = user.into();
    user.email_verified = Set(true);
    user.update(&txn).await?;

    txn.commit().await?;

    Ok(verification.user_id.unwrap())
}
```

### Email Sender Trait

```rust
/// Trait for email sending implementations
pub trait EmailSender {
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()>;
}

/// Mock email sender for development
pub struct MockEmailSender;

impl EmailSender for MockEmailSender {
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()> {
        tracing::info!("Mock email sent to {}: verification token = {}", to, token);
        Ok(())
    }
}

/// Production email sender (to be implemented)
pub struct SmtpEmailSender {
    // SMTP configuration
}

impl EmailSender for SmtpEmailSender {
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()> {
        // TODO: Implement SMTP email sending
        todo!("Implement SMTP email sender")
    }
}
```

## Cache Services

**Location**: `services/valkey/`

### Token Blacklist

```rust
// services/valkey/blacklist.rs

/// Add access token to blacklist (for immediate logout)
pub async fn blacklist_token(
    redis: &mut redis::Connection,
    jti: &str,
    expiry_seconds: i64,
) -> Result<()> {
    let key = format!("blacklist:{}", jti);
    redis::cmd("SETEX")
        .arg(&key)
        .arg(expiry_seconds)
        .arg("1")
        .query(redis)
        .map_err(|e| AuthError::InternalError)?;

    Ok(())
}

/// Check if access token is blacklisted
pub async fn is_token_blacklisted(
    redis: &mut redis::Connection,
    jti: &str,
) -> Result<bool> {
    let key = format!("blacklist:{}", jti);
    let exists: bool = redis::cmd("EXISTS")
        .arg(&key)
        .query(redis)
        .map_err(|_| AuthError::InternalError)?;

    Ok(exists)
}
```

### Rate Limiting

```rust
// services/valkey/rate_limit.rs

/// Check and increment rate limit for IP address
pub async fn check_rate_limit(
    redis: &mut redis::Connection,
    ip: &str,
    max_attempts: u32,
    window_seconds: u32,
) -> Result<bool> {
    let key = format!("ratelimit:{}:{}", ip, Utc::now().timestamp() / window_seconds as i64);

    let count: u32 = redis::cmd("INCR")
        .arg(&key)
        .query(redis)
        .unwrap_or(1);

    // Set expiry on first increment
    if count == 1 {
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(window_seconds)
            .query(redis)?;
    }

    Ok(count <= max_attempts)
}
```

## Service Patterns

### Error Handling Pattern

```rust
// Define domain errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Convert to anyhow::Result
pub type Result<T> = anyhow::Result<T>;

// Use in service functions
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    // ... implementation
    Ok(true)
}
```

### Configuration Pattern

```rust
/// Configuration from environment
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev_secret".to_string()),
            access_token_expiry_minutes: std::env::var("JWT_ACCESS_EXPIRY_MINUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            refresh_token_expiry_days: std::env::var("JWT_REFRESH_EXPIRY_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
        }
    }
}
```

### Transaction Pattern

```rust
pub async fn atomic_operation(db: &DatabaseConnection) -> Result<()> {
    // Begin transaction
    let txn = db.begin().await?;

    // Perform operations
    let result1 = operation1(&txn).await?;
    let result2 = operation2(&txn, result1).await?;

    // Commit transaction
    txn.commit().await?;

    Ok(())
}
```

## Testing Services

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(hash.starts_with("$argon2"));
        assert!(hash.len() > 50);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn test_token_rotation() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        // ... setup database schema ...

        let user_id = Uuid::new_v4();
        let config = JwtConfig::test_config();

        // Create and store initial token
        let (token1, jti1) = create_refresh_token(user_id, &config).unwrap();
        store_refresh_token(&db, user_id, &token1, jti1, 7).await.unwrap();

        // Rotate token
        let (token2, jti2) = create_refresh_token(user_id, &config).unwrap();
        rotate_refresh_token(&db, jti1, &token2, jti2, user_id, 7).await.unwrap();

        // Verify old token is revoked
        assert!(validate_refresh_token(&db, &token1, jti1).await.is_err());

        // Verify new token works
        assert!(validate_refresh_token(&db, &token2, jti2).await.is_ok());
    }
}
```

## Best Practices

1. **Keep services pure**: Avoid side effects when possible
2. **Use domain errors**: Define specific error types for each domain
3. **Write comprehensive tests**: Unit tests for pure functions, integration tests for database operations
4. **Document security properties**: Explain cryptographic choices and security guarantees
5. **Use transactions**: Ensure atomic operations for related database changes
6. **Log appropriately**: Info for important operations, debug for details, error for failures

## Related Documentation

- [Architecture](./architecture.md) - Overall backend architecture
- [API Handlers](./api-handlers.md) - Handler layer that uses services
- [Models](./models.md) - Database entities used by services
- [Testing](./testing.md) - Comprehensive testing guide
