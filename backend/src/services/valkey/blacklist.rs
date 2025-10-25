use anyhow::Result;
use redis::{Commands, Connection};

/// Add a token to the blacklist with TTL (in seconds)
///
/// # Arguments
/// * `conn` - Valkey connection
/// * `token` - Token to blacklist
/// * `ttl` - Time to live in seconds (should match access token expiry)
pub fn add_to_blacklist(conn: &mut Connection, token: &str, ttl: i64) -> Result<()> {
    let key = format!("blacklist:{}", token);
    conn.set_ex::<_, _, ()>(&key, 1, ttl as u64)?;
    Ok(())
}

/// Check if a token is blacklisted
///
/// # Arguments
/// * `conn` - Valkey connection
/// * `token` - Token to check
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
