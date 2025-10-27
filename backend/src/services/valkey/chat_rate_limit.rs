//! Chat-specific rate limiting service.
//!
//! This module provides two-tier rate limiting for chat messages:
//! - Per-minute rate limit (burst protection)
//! - Daily quota (cost control)
//!
//! # Architecture
//!
//! - **Per-Minute Key**: `ratelimit:chat:user:{user_id}:minute`
//! - **Daily Quota Key**: `quota:chat:user:{user_id}:daily`
//! - **Auto-Expiry**: Counters reset automatically after window expires
//!
//! # Configuration
//!
//! Rate limits are loaded from environment variables via `ChatConfig`:
//! - `CHAT_RATE_LIMIT_PER_MINUTE` - Messages per minute (default: 20)
//! - `CHAT_DAILY_MESSAGE_QUOTA` - Messages per day (default: 100)

use anyhow::Result;
use redis::{Commands, Connection};
use uuid::Uuid;

/// Rate limit check result with detailed information
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the rate limit was exceeded
    pub exceeded: bool,
    /// Type of limit that was exceeded (if any)
    pub limit_type: Option<LimitType>,
    /// Current count in the window
    pub current: u64,
    /// Maximum allowed in the window
    pub limit: u64,
    /// Seconds until the limit resets
    pub retry_after: Option<i64>,
}

/// Type of rate limit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitType {
    /// Per-minute rate limit
    PerMinute,
    /// Daily quota limit
    Daily,
}

impl LimitType {
    /// Get human-readable description
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PerMinute => "per_minute",
            Self::Daily => "daily",
        }
    }
}

/// Configuration for chat rate limiting
#[derive(Debug, Clone)]
pub struct ChatRateLimitConfig {
    /// Messages allowed per minute
    pub rate_limit_per_minute: u64,
    /// Messages allowed per day
    pub daily_message_quota: u64,
}

impl Default for ChatRateLimitConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 20,
            daily_message_quota: 100,
        }
    }
}

/// Check both rate limit tiers for a user
///
/// Checks per-minute rate limit first, then daily quota.
/// Increments counters only if both checks pass.
///
/// # Arguments
///
/// * `conn` - Active Redis connection
/// * `user_id` - User UUID to rate limit
/// * `config` - Rate limit configuration
///
/// # Returns
///
/// `RateLimitResult` with details about the check
pub fn check_chat_rate_limit(
    conn: &mut Connection,
    user_id: Uuid,
    config: &ChatRateLimitConfig,
) -> Result<RateLimitResult> {
    // Check per-minute rate limit first (fast fail)
    let minute_result = check_per_minute_limit(conn, user_id, config.rate_limit_per_minute)?;
    if minute_result.exceeded {
        return Ok(minute_result);
    }

    // Check daily quota
    let daily_result = check_daily_quota(conn, user_id, config.daily_message_quota)?;
    if daily_result.exceeded {
        return Ok(daily_result);
    }

    // Both checks passed - increment counters
    increment_chat_counters(conn, user_id)?;

    Ok(RateLimitResult {
        exceeded: false,
        limit_type: None,
        current: minute_result.current + 1,
        limit: config.rate_limit_per_minute,
        retry_after: None,
    })
}

/// Check per-minute rate limit without incrementing
fn check_per_minute_limit(
    conn: &mut Connection,
    user_id: Uuid,
    limit: u64,
) -> Result<RateLimitResult> {
    let key = format!("ratelimit:chat:user:{}:minute", user_id);
    let count: Option<u64> = conn.get(&key)?;
    let current = count.unwrap_or(0);

    if current >= limit {
        // Get TTL for retry_after
        let ttl: i64 = conn.ttl(&key)?;
        Ok(RateLimitResult {
            exceeded: true,
            limit_type: Some(LimitType::PerMinute),
            current,
            limit,
            retry_after: Some(ttl.max(0)),
        })
    } else {
        Ok(RateLimitResult {
            exceeded: false,
            limit_type: None,
            current,
            limit,
            retry_after: None,
        })
    }
}

/// Check daily quota without incrementing
fn check_daily_quota(
    conn: &mut Connection,
    user_id: Uuid,
    limit: u64,
) -> Result<RateLimitResult> {
    let key = format!("quota:chat:user:{}:daily", user_id);
    let count: Option<u64> = conn.get(&key)?;
    let current = count.unwrap_or(0);

    if current >= limit {
        // Get TTL for retry_after
        let ttl: i64 = conn.ttl(&key)?;
        Ok(RateLimitResult {
            exceeded: true,
            limit_type: Some(LimitType::Daily),
            current,
            limit,
            retry_after: Some(ttl.max(0)),
        })
    } else {
        Ok(RateLimitResult {
            exceeded: false,
            limit_type: None,
            current,
            limit,
            retry_after: None,
        })
    }
}

/// Increment both rate limit counters
fn increment_chat_counters(conn: &mut Connection, user_id: Uuid) -> Result<()> {
    let minute_key = format!("ratelimit:chat:user:{}:minute", user_id);
    let daily_key = format!("quota:chat:user:{}:daily", user_id);

    // Increment per-minute counter
    let minute_count: Option<u64> = conn.get(&minute_key)?;
    if minute_count.is_none() {
        // First message in this minute - set with TTL
        conn.set_ex::<_, _, ()>(&minute_key, 1, 60)?;
    } else {
        conn.incr::<_, _, ()>(&minute_key, 1)?;
    }

    // Increment daily counter
    let daily_count: Option<u64> = conn.get(&daily_key)?;
    if daily_count.is_none() {
        // First message today - set with TTL (24 hours)
        conn.set_ex::<_, _, ()>(&daily_key, 1, 86400)?;
    } else {
        conn.incr::<_, _, ()>(&daily_key, 1)?;
    }

    Ok(())
}

/// Get current usage stats for a user without incrementing
///
/// # Returns
///
/// Tuple of (per_minute_count, daily_count)
pub fn get_chat_usage(conn: &mut Connection, user_id: Uuid) -> Result<(u64, u64)> {
    let minute_key = format!("ratelimit:chat:user:{}:minute", user_id);
    let daily_key = format!("quota:chat:user:{}:daily", user_id);

    let minute_count: Option<u64> = conn.get(&minute_key)?;
    let daily_count: Option<u64> = conn.get(&daily_key)?;

    Ok((minute_count.unwrap_or(0), daily_count.unwrap_or(0)))
}

/// Reset rate limits for a user (admin function)
pub fn reset_chat_rate_limit(conn: &mut Connection, user_id: Uuid) -> Result<()> {
    let minute_key = format!("ratelimit:chat:user:{}:minute", user_id);
    let daily_key = format!("quota:chat:user:{}:daily", user_id);

    conn.del::<_, ()>(&minute_key)?;
    conn.del::<_, ()>(&daily_key)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_rate_limit_config_defaults() {
        let config = ChatRateLimitConfig::default();
        assert_eq!(config.rate_limit_per_minute, 20);
        assert_eq!(config.daily_message_quota, 100);
    }

    #[test]
    fn test_limit_type_as_str() {
        assert_eq!(LimitType::PerMinute.as_str(), "per_minute");
        assert_eq!(LimitType::Daily.as_str(), "daily");
    }

    #[test]
    fn test_rate_limit_key_format() {
        let user_id = Uuid::new_v4();
        let minute_key = format!("ratelimit:chat:user:{}:minute", user_id);
        let daily_key = format!("quota:chat:user:{}:daily", user_id);

        assert!(minute_key.starts_with("ratelimit:chat:user:"));
        assert!(minute_key.ends_with(":minute"));
        assert!(daily_key.starts_with("quota:chat:user:"));
        assert!(daily_key.ends_with(":daily"));
    }

    #[test]
    fn test_rate_limit_result_exceeded() {
        let result = RateLimitResult {
            exceeded: true,
            limit_type: Some(LimitType::PerMinute),
            current: 20,
            limit: 20,
            retry_after: Some(45),
        };

        assert!(result.exceeded);
        assert_eq!(result.limit_type, Some(LimitType::PerMinute));
        assert_eq!(result.current, 20);
        assert_eq!(result.retry_after, Some(45));
    }
}
