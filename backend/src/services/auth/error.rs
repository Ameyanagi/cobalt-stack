use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Authentication and authorization error types.
///
/// Domain-specific errors for authentication operations including login,
/// registration, token management, and email verification. These errors
/// are automatically mapped to HTTP status codes via [`IntoResponse`].
///
/// # Error Categories
///
/// - **Authentication**: `InvalidCredentials`, `TokenExpired`, `InvalidToken`
/// - **Authorization**: `EmailNotVerified`, `TokenBlacklisted`
/// - **User Management**: `UserAlreadyExists`, `UserNotFound`
/// - **Input Validation**: `InvalidInput`, `WeakPassword`
/// - **Infrastructure**: `DatabaseError`, `RedisError`, `InternalError`
/// - **Rate Limiting**: `RateLimitExceeded`
///
/// # HTTP Status Mapping
///
/// | Error | HTTP Status |
/// |-------|-------------|
/// | `InvalidCredentials` | 401 Unauthorized |
/// | `UserAlreadyExists` | 409 Conflict |
/// | `UserNotFound` | 404 Not Found |
/// | `EmailNotVerified` | 403 Forbidden |
/// | `RateLimitExceeded` | 429 Too Many Requests |
/// | `InvalidInput` | 400 Bad Request |
/// | `DatabaseError` | 500 Internal Server Error |
///
/// # Examples
///
/// ```
/// use cobalt_stack::services::auth::AuthError;
///
/// // Create specific error types
/// let err = AuthError::InvalidCredentials;
/// assert_eq!(err.to_string(), "Invalid credentials");
///
/// let err = AuthError::InvalidInput("Username too short".to_string());
/// assert_eq!(err.to_string(), "Invalid input: Username too short");
/// ```
///
/// # Axum Integration
///
/// Errors are automatically converted to HTTP responses:
///
/// ```no_run
/// use cobalt_stack::services::auth::AuthError;
/// use axum::response::IntoResponse;
///
/// async fn handler() -> Result<String, AuthError> {
///     // Returns 401 Unauthorized with JSON error message
///     Err(AuthError::InvalidCredentials)
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// User provided incorrect email or password.
    ///
    /// Returned when credentials don't match database records.
    /// Maps to HTTP 401 Unauthorized.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User account with this username or email already exists.
    ///
    /// Returned during registration when username/email is taken.
    /// Maps to HTTP 409 Conflict.
    #[error("User already exists")]
    UserAlreadyExists,

    /// User account not found in database.
    ///
    /// Returned when looking up user by ID or username.
    /// Maps to HTTP 404 Not Found.
    #[error("User not found")]
    UserNotFound,

    /// JWT token has expired and is no longer valid.
    ///
    /// Returned when token's `exp` claim is in the past.
    /// Maps to HTTP 401 Unauthorized.
    #[error("Token expired")]
    TokenExpired,

    /// JWT token is malformed, has invalid signature, or missing claims.
    ///
    /// Returned when token verification fails for any reason except expiry.
    /// Maps to HTTP 401 Unauthorized.
    #[error("Invalid token")]
    InvalidToken,

    /// JWT token has been revoked and cannot be used.
    ///
    /// Returned when checking blacklist/revocation list.
    /// Maps to HTTP 401 Unauthorized.
    #[error("Token blacklisted")]
    TokenBlacklisted,

    /// Too many authentication attempts from this IP/user.
    ///
    /// Returned when rate limit is exceeded (e.g., 5 login attempts in 15 minutes).
    /// Maps to HTTP 429 Too Many Requests.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// User's email address has not been verified.
    ///
    /// Returned when accessing protected resources requiring email verification.
    /// Maps to HTTP 403 Forbidden.
    #[error("Email not verified")]
    EmailNotVerified,

    /// Password does not meet complexity requirements.
    ///
    /// Returned when password is too short, weak, or common.
    /// Maps to HTTP 400 Bad Request.
    #[error("Weak password")]
    WeakPassword,

    /// Input validation failed with specific reason.
    ///
    /// Wraps validation error messages (e.g., "Username must be 3-50 characters").
    /// Maps to HTTP 400 Bad Request.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Database operation failed.
    ///
    /// Wraps `SeaORM` database errors. Details are logged but not exposed to client.
    /// Maps to HTTP 500 Internal Server Error.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Redis/Valkey cache operation failed.
    ///
    /// Wraps Redis client errors. Details are logged but not exposed to client.
    /// Maps to HTTP 500 Internal Server Error.
    #[error("Redis error: {0}")]
    RedisError(String),

    /// Password hashing operation failed.
    ///
    /// Returned when Argon2 hashing fails (rare, usually system issue).
    /// Maps to HTTP 500 Internal Server Error.
    #[error("Password hashing error")]
    PasswordHashError,

    /// JWT token encoding failed.
    ///
    /// Returned when creating JWT fails (rare, usually configuration issue).
    /// Maps to HTTP 500 Internal Server Error.
    #[error("JWT encoding error")]
    JwtEncodingError,

    /// JWT token decoding failed.
    ///
    /// Returned when parsing JWT fails (rare, usually token corruption).
    /// Maps to HTTP 500 Internal Server Error.
    #[error("JWT decoding error")]
    JwtDecodingError,

    /// Unspecified internal server error.
    ///
    /// Generic error for unexpected conditions.
    /// Maps to HTTP 500 Internal Server Error.
    #[error("Internal server error")]
    InternalError,
}

/// Implement Axum's `IntoResponse` for automatic HTTP status mapping
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            Self::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            Self::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            Self::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            Self::TokenBlacklisted => (StatusCode::UNAUTHORIZED, "Token has been revoked"),
            Self::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts"),
            Self::EmailNotVerified => (StatusCode::FORBIDDEN, "Email not verified"),
            Self::WeakPassword => (
                StatusCode::BAD_REQUEST,
                "Password does not meet security requirements",
            ),
            Self::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            Self::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database operation failed",
            ),
            Self::RedisError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cache operation failed"),
            Self::PasswordHashError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password processing failed",
            ),
            Self::JwtEncodingError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed")
            }
            Self::JwtDecodingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token verification failed",
            ),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

/// Convert database errors to `AuthError`
impl From<sea_orm::DbErr> for AuthError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!("Database error: {:?}", err);
        Self::DatabaseError(err.to_string())
    }
}

/// Application-level Result type using anyhow for flexible error propagation
pub type Result<T> = anyhow::Result<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_messages() {
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "Invalid credentials"
        );
        assert_eq!(
            AuthError::UserAlreadyExists.to_string(),
            "User already exists"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
        assert_eq!(
            AuthError::RateLimitExceeded.to_string(),
            "Rate limit exceeded"
        );
    }

    #[test]
    fn test_auth_error_status_codes() {
        use axum::http::StatusCode;

        let response = AuthError::InvalidCredentials.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = AuthError::UserAlreadyExists.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let response = AuthError::RateLimitExceeded.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        let response = AuthError::DatabaseError("test".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_database_error_conversion() {
        let db_err = sea_orm::DbErr::Custom("test error".to_string());
        let auth_err: AuthError = db_err.into();

        match auth_err {
            AuthError::DatabaseError(msg) => assert!(msg.contains("test error")),
            _ => panic!("Expected DatabaseError variant"),
        }
    }
}
