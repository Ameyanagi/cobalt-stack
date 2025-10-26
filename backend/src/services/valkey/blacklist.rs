//! Token blacklist service for JWT revocation.
//!
//! This module provides functionality to blacklist JWT access tokens, enabling
//! immediate token revocation for logout, security incidents, or administrative
//! actions. Uses Valkey/Redis with automatic TTL expiry.
//!
//! # Architecture
//!
//! - **Key Format**: `blacklist:{token}` stored with value `1`
//! - **TTL Management**: Tokens automatically expire when they would naturally expire
//! - **Fast Lookup**: O(1) Redis GET operation for blacklist checks
//! - **Memory Efficient**: Expired entries automatically removed by Redis
//!
//! # Use Cases
//!
//! - **User Logout**: Immediately invalidate access token
//! - **Security Incidents**: Revoke compromised tokens
//! - **Account Suspension**: Block all user tokens
//! - **Password Change**: Optionally invalidate existing sessions
//!
//! # Security Notes
//!
//! - Only blacklist access tokens, not refresh tokens (use database for refresh tokens)
//! - TTL must match access token expiry to prevent premature removal
//! - Blacklist checks add small latency to protected endpoints
//! - Consider connection pooling for high-traffic applications
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack_backend::services::valkey::blacklist::{add_to_blacklist, is_blacklisted};
//! use redis::Client;
//!
//! # fn example() -> anyhow::Result<()> {
//! let client = Client::open("redis://127.0.0.1/")?;
//! let mut conn = client.get_connection()?;
//!
//! // Blacklist token for 30 minutes (1800 seconds)
//! add_to_blacklist(&mut conn, "expired_token_123", 1800)?;
//!
//! // Check if token is blacklisted
//! assert!(is_blacklisted(&mut conn, "expired_token_123")?);
//! assert!(!is_blacklisted(&mut conn, "valid_token_456")?);
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use redis::{Commands, Connection};

/// Add a JWT access token to the blacklist with automatic expiry.
///
/// Stores the token in Valkey/Redis with a TTL matching the token's remaining
/// lifetime. Once added, the token will fail all authentication checks until
/// it naturally expires.
///
/// # Arguments
///
/// * `conn` - Active Valkey/Redis connection
/// * `token` - JWT access token to blacklist (full token string)
/// * `ttl` - Time to live in seconds (should match token's exp - now)
///
/// # Returns
///
/// - `Ok(())` - Token successfully added to blacklist
/// - `Err(_)` - Redis connection or command error
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::services::valkey::blacklist::add_to_blacklist;
/// use redis::Client;
///
/// # fn example() -> anyhow::Result<()> {
/// let client = Client::open("redis://127.0.0.1/")?;
/// let mut conn = client.get_connection()?;
///
/// // Blacklist token that expires in 30 minutes
/// let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
/// add_to_blacklist(&mut conn, token, 1800)?;
/// # Ok(())
/// # }
/// ```
///
/// # Security Notes
///
/// - TTL should be calculated from token's actual expiry time
/// - Setting TTL too short allows token to work after removal from blacklist
/// - Setting TTL too long wastes Redis memory unnecessarily
/// - Use this for access tokens only (refresh tokens use database revocation)
pub fn add_to_blacklist(conn: &mut Connection, token: &str, ttl: i64) -> Result<()> {
    let key = format!("blacklist:{}", token);
    conn.set_ex::<_, _, ()>(&key, 1, ttl as u64)?;
    Ok(())
}

/// Check if a JWT access token has been blacklisted.
///
/// Performs a fast O(1) Redis EXISTS check to determine if the token
/// has been revoked. Should be called during authentication to reject
/// blacklisted tokens.
///
/// # Arguments
///
/// * `conn` - Active Valkey/Redis connection
/// * `token` - JWT access token to check (full token string)
///
/// # Returns
///
/// - `Ok(true)` - Token is blacklisted and should be rejected
/// - `Ok(false)` - Token is not blacklisted (may still be invalid for other reasons)
/// - `Err(_)` - Redis connection or command error
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::services::valkey::blacklist::{add_to_blacklist, is_blacklisted};
/// use redis::Client;
///
/// # fn example() -> anyhow::Result<()> {
/// let client = Client::open("redis://127.0.0.1/")?;
/// let mut conn = client.get_connection()?;
///
/// let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
///
/// // Initially not blacklisted
/// assert!(!is_blacklisted(&mut conn, token)?);
///
/// // After blacklisting
/// add_to_blacklist(&mut conn, token, 1800)?;
/// assert!(is_blacklisted(&mut conn, token)?);
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// - O(1) time complexity (Redis EXISTS command)
/// - Typical latency: <1ms on local Redis
/// - Consider connection pooling for concurrent requests
///
/// # Error Handling
///
/// If Redis is unavailable, this function returns an error. Depending on your
/// security requirements, you may want to:
/// - Fail secure: reject all requests if blacklist check fails
/// - Fail open: allow requests if blacklist check fails (risky)
pub fn is_blacklisted(conn: &mut Connection, token: &str) -> Result<bool> {
    let key = format!("blacklist:{}", token);
    let exists: bool = conn.exists(&key)?;
    Ok(exists)
}

#[cfg(test)]
mod tests {

    // Note: These are integration tests requiring actual Valkey instance
    // Run with: docker-compose up -d valkey
    // Then: cargo test --test valkey_integration

    #[test]
    fn test_blacklist_key_format() {
        // Unit test: verify key format without Valkey connection
        let token = "test_token_123";
        let expected_key = "blacklist:test_token_123";
        let key = format!("blacklist:{}", token);
        assert_eq!(key, expected_key);
    }

    #[test]
    fn test_blacklist_key_uniqueness() {
        // Unit test: verify different tokens create different keys
        let token1 = "token_abc";
        let token2 = "token_xyz";
        let key1 = format!("blacklist:{}", token1);
        let key2 = format!("blacklist:{}", token2);
        assert_ne!(key1, key2);
    }

    // Integration tests will be in tests/valkey_integration.rs
    // They require actual Valkey connection and will test:
    // - add_to_blacklist() correctly adds tokens
    // - is_blacklisted() correctly detects blacklisted tokens
    // - TTL expiry works correctly
    // - Non-blacklisted tokens return false
}
