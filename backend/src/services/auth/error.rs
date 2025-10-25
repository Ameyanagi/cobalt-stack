use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Domain-specific authentication errors using thiserror
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("User not found")]
    UserNotFound,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token blacklisted")]
    TokenBlacklisted,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Weak password")]
    WeakPassword,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Password hashing error")]
    PasswordHashError,

    #[error("JWT encoding error")]
    JwtEncodingError,

    #[error("JWT decoding error")]
    JwtDecodingError,

    #[error("Internal server error")]
    InternalError,
}

/// Implement Axum's IntoResponse for automatic HTTP status mapping
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::TokenBlacklisted => (StatusCode::UNAUTHORIZED, "Token has been revoked"),
            AuthError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts")
            }
            AuthError::EmailNotVerified => (StatusCode::FORBIDDEN, "Email not verified"),
            AuthError::WeakPassword => (
                StatusCode::BAD_REQUEST,
                "Password does not meet security requirements",
            ),
            AuthError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AuthError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database operation failed",
            ),
            AuthError::RedisError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cache operation failed",
            ),
            AuthError::PasswordHashError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password processing failed",
            ),
            AuthError::JwtEncodingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token generation failed",
            ),
            AuthError::JwtDecodingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token verification failed",
            ),
            AuthError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

/// Convert database errors to AuthError
impl From<sea_orm::DbErr> for AuthError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!("Database error: {:?}", err);
        AuthError::DatabaseError(err.to_string())
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
