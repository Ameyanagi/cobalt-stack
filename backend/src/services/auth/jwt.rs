//! JSON Web Token (JWT) creation and verification.
//!
//! This module provides JWT token generation and verification for authentication.
//! It implements a dual-token system with short-lived access tokens and long-lived
//! refresh tokens for secure, stateless authentication.
//!
//! # Token Types
//!
//! - **Access Token**: Short-lived (30 minutes), includes user ID and username
//! - **Refresh Token**: Long-lived (7 days), includes user ID and token ID (jti)
//!
//! # Security
//!
//! - HMAC-SHA256 signature algorithm (HS256)
//! - Configurable secret key from environment
//! - Token expiration validation
//! - Token rotation via jti tracking
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::services::auth::jwt::{
//!     JwtConfig, create_access_token, create_refresh_token,
//!     verify_access_token, verify_refresh_token
//! };
//! use uuid::Uuid;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration
//! let config = JwtConfig::from_env();
//!
//! // Create tokens
//! let user_id = Uuid::new_v4();
//! let access_token = create_access_token(user_id, "alice".to_string(), &config)?;
//! let (refresh_token, jti) = create_refresh_token(user_id, &config)?;
//!
//! // Verify tokens
//! let access_claims = verify_access_token(&access_token, &config)?;
//! let refresh_claims = verify_refresh_token(&refresh_token, &config)?;
//!
//! assert_eq!(access_claims.sub, user_id);
//! assert_eq!(refresh_claims.jti, jti);
//! # Ok(())
//! # }
//! ```

use super::{AuthError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims for access tokens.
///
/// Access tokens are short-lived (default 30 minutes) and include user identity
/// for authentication. They are sent in Authorization headers for API requests.
///
/// # Fields
///
/// - `sub`: User ID (UUID) - standard JWT subject claim
/// - `exp`: Expiration timestamp (Unix epoch) - standard JWT expiration claim
/// - `iat`: Issued at timestamp (Unix epoch) - standard JWT issued-at claim
/// - `username`: Username string for convenience (custom claim)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    /// User ID (subject of the token).
    pub sub: Uuid,

    /// Expiration time as Unix timestamp.
    /// Token is invalid after this time.
    pub exp: i64,

    /// Issued at time as Unix timestamp.
    /// When the token was created.
    pub iat: i64,

    /// Username for convenience in handlers.
    /// Avoids additional database lookups.
    pub username: String,
}

/// JWT claims for refresh tokens.
///
/// Refresh tokens are long-lived (default 7 days) and used to obtain new access tokens.
/// They include a unique token ID (jti) for rotation tracking and revocation.
///
/// # Fields
///
/// - `sub`: User ID (UUID) - standard JWT subject claim
/// - `exp`: Expiration timestamp (Unix epoch) - standard JWT expiration claim
/// - `iat`: Issued at timestamp (Unix epoch) - standard JWT issued-at claim
/// - `jti`: Token ID (UUID) for rotation/revocation - standard JWT ID claim
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    /// User ID (subject of the token).
    pub sub: Uuid,

    /// Expiration time as Unix timestamp.
    /// Token is invalid after this time.
    pub exp: i64,

    /// Issued at time as Unix timestamp.
    /// When the token was created.
    pub iat: i64,

    /// Token ID for rotation tracking.
    /// Matches `refresh_tokens.id` in database.
    pub jti: Uuid,
}

/// JWT configuration loaded from environment variables.
///
/// Controls token expiration times and signing secret.
/// Can be loaded from environment or constructed manually for testing.
///
/// # Environment Variables
///
/// - `JWT_SECRET`: HMAC secret key (required in production)
/// - `JWT_ACCESS_EXPIRY_MINUTES`: Access token lifetime (default: 30)
/// - `JWT_REFRESH_EXPIRY_DAYS`: Refresh token lifetime (default: 7)
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack::services::auth::jwt::JwtConfig;
///
/// // Load from environment
/// let config = JwtConfig::from_env();
///
/// // Or construct manually for testing
/// let config = JwtConfig {
///     secret: "test_secret".to_string(),
///     access_token_expiry_minutes: 15,
///     refresh_token_expiry_days: 7,
/// };
/// ```
#[derive(Clone)]
pub struct JwtConfig {
    /// HMAC secret key for signing tokens.
    /// Must be kept secure and never exposed to clients.
    pub secret: String,

    /// Access token lifetime in minutes.
    /// Shorter lifetimes increase security but require more refreshes.
    pub access_token_expiry_minutes: i64,

    /// Refresh token lifetime in days.
    /// Longer lifetimes improve UX but increase risk if compromised.
    pub refresh_token_expiry_days: i64,
}

impl JwtConfig {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using default (INSECURE FOR PRODUCTION!)");
                "dev_secret_key_change_in_production".to_string()
            }),
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

/// Create an access token
pub fn create_access_token(user_id: Uuid, username: String, config: &JwtConfig) -> Result<String> {
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
    .map_err(|e| {
        tracing::error!("JWT encoding failed: {:?}", e);
        anyhow::Error::new(AuthError::JwtEncodingError)
    })
}

/// Create a refresh token
pub fn create_refresh_token(user_id: Uuid, config: &JwtConfig) -> Result<(String, Uuid)> {
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
    .map_err(|e| {
        tracing::error!("JWT encoding failed: {:?}", e);
        anyhow::Error::new(AuthError::JwtEncodingError)
    })?;

    Ok((token, jti))
}

/// Verify and decode an access token
pub fn verify_access_token(token: &str, config: &JwtConfig) -> Result<AccessTokenClaims> {
    let token_data = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::debug!("JWT decoding failed: {:?}", e);
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    })?;

    Ok(token_data.claims)
}

/// Verify and decode a refresh token
pub fn verify_refresh_token(token: &str, config: &JwtConfig) -> Result<RefreshTokenClaims> {
    let token_data = decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::debug!("JWT decoding failed: {:?}", e);
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    })?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key".to_string(),
            access_token_expiry_minutes: 30,
            refresh_token_expiry_days: 7,
        }
    }

    #[test]
    fn test_create_access_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let token = create_access_token(user_id, username, &config).unwrap();

        // JWT should have 3 parts separated by dots
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_verify_access_token_valid() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let token = create_access_token(user_id, username.clone(), &config).unwrap();
        let claims = verify_access_token(&token, &config).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
        assert!(claims.exp > Utc::now().timestamp());
    }

    #[test]
    fn test_verify_access_token_invalid() {
        let config = test_config();
        let result = verify_access_token("invalid.token.here", &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid token"));
    }

    #[test]
    fn test_verify_access_token_wrong_secret() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let token = create_access_token(user_id, username, &config).unwrap();

        // Try to verify with wrong secret
        let wrong_config = JwtConfig {
            secret: "wrong_secret".to_string(),
            ..config
        };

        let result = verify_access_token(&token, &wrong_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_refresh_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let (token, jti) = create_refresh_token(user_id, &config).unwrap();

        // JWT should have 3 parts
        assert_eq!(token.split('.').count(), 3);

        // jti should be valid UUID
        assert_ne!(jti, Uuid::nil());
    }

    #[test]
    fn test_verify_refresh_token_valid() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let (token, jti) = create_refresh_token(user_id, &config).unwrap();
        let claims = verify_refresh_token(&token, &config).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.jti, jti);
        assert!(claims.exp > Utc::now().timestamp());
    }

    #[test]
    fn test_verify_refresh_token_invalid() {
        let config = test_config();
        let result = verify_refresh_token("invalid.token.here", &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_tokens_have_different_jti() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let (_, jti1) = create_refresh_token(user_id, &config).unwrap();
        let (_, jti2) = create_refresh_token(user_id, &config).unwrap();

        // Each refresh token should have unique jti
        assert_ne!(jti1, jti2);
    }

    #[test]
    fn test_jwt_config_from_env() {
        // This test verifies the from_env method doesn't panic
        let _config = JwtConfig::from_env();
    }

    #[test]
    fn test_access_token_expiry() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            access_token_expiry_minutes: 1,
            refresh_token_expiry_days: 7,
        };

        let user_id = Uuid::new_v4();
        let token = create_access_token(user_id, "test".to_string(), &config).unwrap();
        let claims = verify_access_token(&token, &config).unwrap();

        let expected_exp = Utc::now().timestamp() + 60;
        // Allow 5 second tolerance
        assert!((claims.exp - expected_exp).abs() < 5);
    }

    #[test]
    fn test_refresh_token_expiry() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            access_token_expiry_minutes: 30,
            refresh_token_expiry_days: 1,
        };

        let user_id = Uuid::new_v4();
        let (token, _) = create_refresh_token(user_id, &config).unwrap();
        let claims = verify_refresh_token(&token, &config).unwrap();

        let expected_exp = Utc::now().timestamp() + 86400; // 1 day
                                                           // Allow 5 second tolerance
        assert!((claims.exp - expected_exp).abs() < 5);
    }
}
