use crate::services::auth::{AuthError, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================================
// DTOs (Data Transfer Objects)
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "alice")]
    pub username: String,

    #[schema(example = "alice@example.com")]
    pub email: String,

    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "alice")]
    pub username: String,

    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

// ============================================================================
// Validation
// ============================================================================

impl RegisterRequest {
    pub fn validate(&self) -> Result<()> {
        // Username validation
        if self.username.is_empty() {
            return Err(AuthError::InvalidInput("Username cannot be empty".to_string()).into());
        }
        if self.username.len() < 3 || self.username.len() > 50 {
            return Err(AuthError::InvalidInput(
                "Username must be between 3 and 50 characters".to_string(),
            )
            .into());
        }

        // Email validation (basic)
        if self.email.is_empty() {
            return Err(AuthError::InvalidInput("Email cannot be empty".to_string()).into());
        }
        if !self.email.contains('@') {
            return Err(AuthError::InvalidInput("Invalid email format".to_string()).into());
        }

        // Password validation
        if self.password.len() < 8 {
            return Err(AuthError::InvalidInput(
                "Password must be at least 8 characters".to_string(),
            )
            .into());
        }
        if self.password.len() > 128 {
            return Err(
                AuthError::InvalidInput("Password must not exceed 128 characters".to_string())
                    .into(),
            );
        }

        Ok(())
    }
}

impl LoginRequest {
    pub fn validate(&self) -> Result<()> {
        if self.username.is_empty() {
            return Err(AuthError::InvalidInput("Username cannot be empty".to_string()).into());
        }
        if self.password.is_empty() {
            return Err(AuthError::InvalidInput("Password cannot be empty".to_string()).into());
        }
        Ok(())
    }
}

// ============================================================================
// Handlers
// ============================================================================

use crate::models::{prelude::*, users};
use crate::services::auth::{
    create_access_token, create_refresh_token, hash_password, verify_password, JwtConfig,
    store_refresh_token,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub jwt_config: JwtConfig,
}

/// POST /api/auth/register - Register a new user
///
/// Creates a new user account with username/email/password.
/// Returns access token on success.
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
    // Validate input
    req.validate().map_err(|e| {
        // The validate() function already returns AuthError wrapped in anyhow::Error
        // Extract the AuthError from the anyhow chain
        e.downcast::<AuthError>().unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // Check if username already exists
    let existing_user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?;

    if existing_user.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // Check if email already exists
    let existing_email = Users::find()
        .filter(users::Column::Email.eq(&req.email))
        .one(state.db.as_ref())
        .await?;

    if existing_email.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&req.password).map_err(|_| AuthError::PasswordHashError)?;

    // Create user
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

    // Generate tokens
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;
    let (refresh_token, refresh_jti) = create_refresh_token(user.id, &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;

    // Store refresh token in database
    store_refresh_token(
        state.db.as_ref(),
        user.id,
        &refresh_token,
        refresh_jti,
        state.jwt_config.refresh_token_expiry_days,
    )
    .await
    .map_err(|_| AuthError::DatabaseError("Failed to store refresh token".to_string()))?;

    // Return response
    let response = AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// POST /api/auth/login - Login with username/password
///
/// Authenticates user and returns access token.
/// Rate limited to 5 attempts per 15 minutes per IP.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // Validate input
    req.validate().map_err(|e| {
        // The validate() function already returns AuthError wrapped in anyhow::Error
        // Extract the AuthError from the anyhow chain
        e.downcast::<AuthError>().unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // Find user by username
    let user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    let password_hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
    let is_valid = verify_password(&req.password, &password_hash)
        .map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Generate tokens
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;
    let (refresh_token, refresh_jti) = create_refresh_token(user.id, &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;

    // Store refresh token in database
    store_refresh_token(
        state.db.as_ref(),
        user.id,
        &refresh_token,
        refresh_jti,
        state.jwt_config.refresh_token_expiry_days,
    )
    .await
    .map_err(|_| AuthError::DatabaseError("Failed to store refresh token".to_string()))?;

    // Return response
    let response = AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// POST /api/auth/refresh - Refresh access token using refresh token
///
/// Rotates refresh token and returns new access token.
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
    State(_state): State<AppState>,
    // TODO: Extract refresh token from HttpOnly cookie
) -> std::result::Result<impl IntoResponse, AuthError> {
    // This will be implemented when we add cookie handling
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// POST /api/auth/logout - Logout and invalidate tokens
///
/// Revokes refresh token and blacklists access token.
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn logout(
    State(_state): State<AppState>,
    // TODO: Extract tokens from request
) -> std::result::Result<impl IntoResponse, AuthError> {
    // This will be implemented when we add middleware
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// GET /api/auth/me - Get current user information
///
/// Protected route - requires valid access token.
#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "User information", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "Authentication",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    State(_state): State<AppState>,
    // TODO: Extract user from middleware
) -> std::result::Result<impl IntoResponse, AuthError> {
    // This will be implemented when we add auth middleware
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_validation_valid() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_register_request_validation_empty_username() {
        let req = RegisterRequest {
            username: "".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_register_request_validation_username_too_short() {
        let req = RegisterRequest {
            username: "ab".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 3 and 50"));
    }

    #[test]
    fn test_register_request_validation_username_too_long() {
        let req = RegisterRequest {
            username: "a".repeat(51),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 3 and 50"));
    }

    #[test]
    fn test_register_request_validation_invalid_email() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "not-an-email".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid email"));
    }

    #[test]
    fn test_register_request_validation_password_too_short() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "short".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 8 characters"));
    }

    #[test]
    fn test_register_request_validation_password_too_long() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "a".repeat(129),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not exceed 128"));
    }

    #[test]
    fn test_login_request_validation_valid() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_login_request_validation_empty_username() {
        let req = LoginRequest {
            username: "".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_login_request_validation_empty_password() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "".to_string(),
        };
        assert!(req.validate().is_err());
    }
}
