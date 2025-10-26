//! Authentication service layer.
//!
//! This module provides business logic for user authentication, including
//! JWT token generation/verification, password hashing, and secure token rotation.
//! It follows Domain-Driven Design principles with clear separation between
//! the service layer (business logic) and handler layer (HTTP concerns).
//!
//! # Architecture
//!
//! The authentication service is organized into submodules:
//!
//! - **error**: Domain-specific error types and HTTP mapping
//! - **jwt**: JSON Web Token creation and verification
//! - **password**: Secure password hashing and verification with Argon2
//! - **token_rotation**: Refresh token rotation and revocation
//!
//! # Security Features
//!
//! - **Argon2 password hashing**: Industry-standard password protection
//! - **JWT tokens**: Stateless authentication with access/refresh tokens
//! - **Token rotation**: Automatic refresh token rotation prevents theft
//! - **Token revocation**: Individual token revocation capability
//! - **Hash storage**: Refresh tokens stored as SHA-256 hashes
//!
//! # Token Flow
//!
//! ```text
//! 1. Register/Login → Generate access token (30m) + refresh token (7d)
//! 2. Access protected resources with access token
//! 3. Access token expires → Use refresh token to get new pair
//! 4. Refresh token used → Old token revoked, new token issued (rotation)
//! 5. Logout → Refresh token revoked
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::services::auth::{
//!     hash_password, verify_password, create_access_token,
//!     create_refresh_token, JwtConfig
//! };
//! use uuid::Uuid;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Password hashing
//! let hash = hash_password("user_password")?;
//! assert!(verify_password("user_password", &hash)?);
//!
//! // JWT token generation
//! let config = JwtConfig::from_env();
//! let user_id = Uuid::new_v4();
//! let access_token = create_access_token(user_id, "alice".to_string(), &config)?;
//! let (refresh_token, jti) = create_refresh_token(user_id, &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! All service functions return [`Result<T>`] using domain-specific [`AuthError`] types.
//! Errors are automatically mapped to appropriate HTTP status codes via `IntoResponse`.

pub mod error;
pub mod jwt;
pub mod password;
pub mod token_rotation;

pub use error::{AuthError, Result};
pub use jwt::{
    create_access_token, create_refresh_token, verify_access_token, verify_refresh_token,
    AccessTokenClaims, JwtConfig, RefreshTokenClaims,
};
pub use password::{hash_password, verify_password};
pub use token_rotation::{
    cleanup_expired_tokens, revoke_all_user_tokens, revoke_refresh_token, rotate_refresh_token,
    store_refresh_token, validate_refresh_token,
};
