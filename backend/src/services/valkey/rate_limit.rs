use anyhow::Result;
use redis::{Commands, Connection};

/// Rate limit configuration
pub struct RateLimitConfig {
    pub max_attempts: u32,
    pub window_seconds: i64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,      // 5 attempts
            window_seconds: 900,  // 15 minutes
        }
    }
}

/// Check rate limit for an IP address
///
/// Returns Ok(true) if rate limit exceeded, Ok(false) if allowed
///
/// # Arguments
/// * `conn` - Valkey connection
/// * `ip` - IP address to check
/// * `config` - Rate limit configuration
pub fn check_rate_limit(
    conn: &mut Connection,
    ip: &str,
    config: &RateLimitConfig,
) -> Result<bool> {
    let key = format!("ratelimit:login:{}", ip);

    // Get current count
    let count: Option<u32> = conn.get(&key)?;

    match count {
        Some(current) if current >= config.max_attempts => {
            // Rate limit exceeded
            Ok(true)
        }
        Some(_current) => {
            // Increment counter
            conn.incr::<_, _, ()>(&key, 1)?;
            Ok(false)
        }
        None => {
            // First attempt - set counter and TTL
            conn.set_ex::<_, _, ()>(&key, 1, config.window_seconds as u64)?;
            Ok(false)
        }
    }
}

/// Reset rate limit for an IP address (for testing or manual unblock)
///
/// # Arguments
/// * `conn` - Valkey connection
/// * `ip` - IP address to reset
pub fn reset_rate_limit(conn: &mut Connection, ip: &str) -> Result<()> {
    let key = format!("ratelimit:login:{}", ip);
    conn.del::<_, ()>(&key)?;
    Ok(())
}

/// Get current attempt count for an IP address
///
/// # Arguments
/// * `conn` - Valkey connection
/// * `ip` - IP address to check
pub fn get_attempt_count(conn: &mut Connection, ip: &str) -> Result<u32> {
    let key = format!("ratelimit:login:{}", ip);
    let count: Option<u32> = conn.get(&key)?;
    Ok(count.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.window_seconds, 900); // 15 minutes
    }

    #[test]
    fn test_rate_limit_key_format() {
        let ip = "192.168.1.1";
        let expected_key = "ratelimit:login:192.168.1.1";
        let key = format!("ratelimit:login:{}", ip);
        assert_eq!(key, expected_key);
    }

    #[test]
    fn test_rate_limit_key_uniqueness() {
        let ip1 = "192.168.1.1";
        let ip2 = "192.168.1.2";
        let key1 = format!("ratelimit:login:{}", ip1);
        let key2 = format!("ratelimit:login:{}", ip2);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_custom_rate_limit_config() {
        let config = RateLimitConfig {
            max_attempts: 10,
            window_seconds: 300,
        };
        assert_eq!(config.max_attempts, 10);
        assert_eq!(config.window_seconds, 300);
    }

    // Integration tests will be in tests/valkey_integration.rs
    // They require actual Valkey connection and will test:
    // - check_rate_limit() correctly increments counter
    // - check_rate_limit() returns true when limit exceeded
    // - check_rate_limit() sets TTL on first attempt
    // - reset_rate_limit() clears the counter
    // - get_attempt_count() returns correct count
    // - TTL expiry resets the counter automatically
}
