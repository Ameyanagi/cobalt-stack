//! Rate limiting service for preventing abuse and brute force attacks.
//!
//! This module provides sliding window rate limiting for login attempts
//! and other security-sensitive operations. Uses Valkey/Redis for distributed
//! rate limiting across multiple application instances.
//!
//! # Architecture
//!
//! - **Key Format**: `ratelimit:login:{ip}` with attempt counter
//! - **Window**: Sliding time window (default 15 minutes)
//! - **Threshold**: Maximum attempts before blocking (default 5)
//! - **Auto-Expiry**: Counters reset automatically after window expires
//!
//! # Default Configuration
//!
//! - **Max Attempts**: 5 login attempts
//! - **Window**: 900 seconds (15 minutes)
//! - **Behavior**: Block on 6th attempt, reset after 15 minutes
//!
//! # Use Cases
//!
//! - **Login Attempts**: Prevent brute force password attacks
//! - **Password Reset**: Limit reset request frequency
//! - **Email Verification**: Prevent verification spam
//! - **API Endpoints**: General rate limiting for public APIs
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack_backend::services::valkey::rate_limit::{
//!     check_rate_limit, reset_rate_limit, RateLimitConfig
//! };
//! use redis::Client;
//!
//! # fn example() -> anyhow::Result<()> {
//! let client = Client::open("redis://127.0.0.1/")?;
//! let mut conn = client.get_connection()?;
//! let config = RateLimitConfig::default();
//!
//! let ip = "192.168.1.100";
//!
//! // Check rate limit before processing login
//! if check_rate_limit(&mut conn, ip, &config)? {
//!     // Rate limit exceeded - reject request
//!     return Err(anyhow::anyhow!("Too many login attempts"));
//! }
//!
//! // Process login attempt...
//!
//! // On successful login, optionally reset counter
//! reset_rate_limit(&mut conn, ip)?;
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use redis::{Commands, Connection};

/// Configuration for rate limiting behavior.
///
/// Defines the maximum number of allowed attempts and the time window
/// for rate limiting. Default values provide reasonable protection against
/// brute force attacks while allowing legitimate user retries.
///
/// # Examples
///
/// ```
/// use cobalt_stack_backend::services::valkey::rate_limit::RateLimitConfig;
///
/// // Use default configuration (5 attempts per 15 minutes)
/// let config = RateLimitConfig::default();
/// assert_eq!(config.max_attempts, 5);
/// assert_eq!(config.window_seconds, 900);
///
/// // Custom configuration (10 attempts per 5 minutes)
/// let strict_config = RateLimitConfig {
///     max_attempts: 10,
///     window_seconds: 300,
/// };
/// ```
pub struct RateLimitConfig {
    /// Maximum allowed attempts before rate limiting triggers.
    pub max_attempts: u32,
    /// Time window in seconds for counting attempts.
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

/// Check and increment rate limit counter for an IP address.
///
/// This function checks if the IP has exceeded the rate limit and increments
/// the counter for each call. On first attempt, it sets a TTL matching the
/// configured window. The counter automatically resets after the window expires.
///
/// # Arguments
///
/// * `conn` - Active Valkey/Redis connection
/// * `ip` - IP address to rate limit (typically from request)
/// * `config` - Rate limit configuration (attempts and window)
///
/// # Returns
///
/// - `Ok(true)` - Rate limit exceeded, reject request (HTTP 429)
/// - `Ok(false)` - Within rate limit, allow request and increment counter
/// - `Err(_)` - Redis connection or command error
///
/// # Behavior
///
/// 1. **First attempt**: Creates key with count=1, sets TTL
/// 2. **Subsequent attempts**: Increments counter
/// 3. **At limit**: Returns true, blocks request
/// 4. **After window**: Key expires, counter resets to 0
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::services::valkey::rate_limit::{check_rate_limit, RateLimitConfig};
/// use redis::Client;
///
/// # fn example() -> anyhow::Result<()> {
/// let client = Client::open("redis://127.0.0.1/")?;
/// let mut conn = client.get_connection()?;
/// let config = RateLimitConfig::default();
///
/// let ip = "203.0.113.42";
///
/// // Check before login attempt
/// if check_rate_limit(&mut conn, ip, &config)? {
///     // Return 429 Too Many Requests
///     println!("Rate limit exceeded for IP: {}", ip);
/// } else {
///     // Process login
///     println!("Login attempt allowed");
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Security Notes
///
/// - Always call this BEFORE processing the request
/// - Use `X-Forwarded-For` header carefully (can be spoofed)
/// - Consider using real client IP from trusted proxy headers
/// - Combine with other security measures (CAPTCHA after N failures)
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

/// Reset rate limit counter for an IP address.
///
/// Removes the rate limit counter for the specified IP, effectively unblocking
/// them immediately. Use this for successful authentications, admin overrides,
/// or testing scenarios.
///
/// # Arguments
///
/// * `conn` - Active Valkey/Redis connection
/// * `ip` - IP address to reset
///
/// # Returns
///
/// - `Ok(())` - Counter successfully removed
/// - `Err(_)` - Redis connection or command error
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::services::valkey::rate_limit::reset_rate_limit;
/// use redis::Client;
///
/// # fn example() -> anyhow::Result<()> {
/// let client = Client::open("redis://127.0.0.1/")?;
/// let mut conn = client.get_connection()?;
///
/// let ip = "203.0.113.42";
///
/// // Reset counter after successful login
/// reset_rate_limit(&mut conn, ip)?;
/// println!("Rate limit reset for IP: {}", ip);
/// # Ok(())
/// # }
/// ```
///
/// # Use Cases
///
/// - **Successful Login**: Reset counter after valid credentials
/// - **Admin Override**: Manually unblock a user/IP
/// - **False Positive**: Clear counter for legitimate users
/// - **Testing**: Reset between test cases
pub fn reset_rate_limit(conn: &mut Connection, ip: &str) -> Result<()> {
    let key = format!("ratelimit:login:{}", ip);
    conn.del::<_, ()>(&key)?;
    Ok(())
}

/// Get current attempt count for an IP address without incrementing.
///
/// Retrieves the current number of login attempts for the given IP without
/// modifying the counter. Useful for monitoring, logging, or UI display.
///
/// # Arguments
///
/// * `conn` - Active Valkey/Redis connection
/// * `ip` - IP address to check
///
/// # Returns
///
/// - `Ok(count)` - Current attempt count (0 if no attempts or expired)
/// - `Err(_)` - Redis connection or command error
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::services::valkey::rate_limit::get_attempt_count;
/// use redis::Client;
///
/// # fn example() -> anyhow::Result<()> {
/// let client = Client::open("redis://127.0.0.1/")?;
/// let mut conn = client.get_connection()?;
///
/// let ip = "203.0.113.42";
/// let count = get_attempt_count(&mut conn, ip)?;
///
/// println!("IP {} has {} failed attempts", ip, count);
/// # Ok(())
/// # }
/// ```
///
/// # Use Cases
///
/// - **Monitoring**: Track failed login patterns
/// - **Logging**: Include attempt count in security logs
/// - **UI Display**: Show "X attempts remaining" message
/// - **Analytics**: Collect rate limit statistics
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
