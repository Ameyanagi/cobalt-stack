use super::{AuthError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims for access tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    pub sub: Uuid,        // user_id
    pub exp: i64,         // expiration time
    pub iat: i64,         // issued at
    pub username: String, // username for convenience
}

/// JWT Claims for refresh tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    pub sub: Uuid, // user_id
    pub exp: i64,  // expiration time
    pub iat: i64,  // issued at
    pub jti: Uuid, // token id for rotation tracking
}

/// JWT Configuration
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
}

impl JwtConfig {
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

        let token = create_access_token(user_id, username.clone(), &config).unwrap();

        // JWT should have 3 parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
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
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

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
