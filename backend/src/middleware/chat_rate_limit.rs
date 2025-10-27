//! Chat rate limiting middleware
//!
//! Enforces per-minute and daily rate limits on chat message endpoints.

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;

use crate::{
    middleware::auth::AuthUser,
    services::valkey::{chat_rate_limit, ValkeyManager},
};

/// Rate limit state shared across middleware
#[derive(Clone)]
pub struct ChatRateLimitState {
    /// Valkey connection manager
    pub valkey: ValkeyManager,
    /// Rate limit configuration
    pub config: chat_rate_limit::ChatRateLimitConfig,
}

/// Chat rate limiting middleware
///
/// Checks both per-minute rate limit and daily quota before allowing request.
/// Returns 429 with detailed error if limit exceeded.
///
/// # Headers Added
///
/// - `X-RateLimit-Limit-Minute`: Per-minute limit
/// - `X-RateLimit-Remaining-Minute`: Remaining in current minute
/// - `X-RateLimit-Reset-Minute`: Unix timestamp when minute resets
/// - `X-RateLimit-Limit-Daily`: Daily quota limit
/// - `X-RateLimit-Remaining-Daily`: Remaining today
/// - `X-RateLimit-Reset-Daily`: Unix timestamp when quota resets
/// - `Retry-After`: Seconds until retry (if rate limited)
pub async fn chat_rate_limit_middleware(
    State(state): State<ChatRateLimitState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract authenticated user
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Unauthorized",
                    "message": "Authentication required"
                })),
            )
        })?;

    // Get Redis connection
    let mut conn = state.valkey.get_connection().map_err(|e| {
        tracing::error!("Failed to connect to Redis: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal server error",
                "message": "Rate limit check failed"
            })),
        )
    })?;

    // Check rate limits
    let result = chat_rate_limit::check_chat_rate_limit(&mut conn, auth_user.user_id, &state.config)
        .map_err(|e| {
            tracing::error!("Rate limit check failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error",
                    "message": "Rate limit check failed"
                })),
            )
        })?;

    // If rate limited, return 429
    if result.exceeded {
        let limit_type = result.limit_type.unwrap();
        let retry_after = result.retry_after.unwrap_or(60);

        let mut headers = HeaderMap::new();
        headers.insert(header::RETRY_AFTER, retry_after.to_string().parse().unwrap());

        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Rate limit exceeded",
                "limit_type": limit_type.as_str(),
                "limit": result.limit,
                "current": result.current,
                "retry_after": retry_after,
                "message": format!(
                    "You have exceeded the {} rate limit. Please try again in {} seconds.",
                    limit_type.as_str(),
                    retry_after
                )
            })),
        ));
    }

    // Get current usage for response headers
    let (minute_count, daily_count) = chat_rate_limit::get_chat_usage(&mut conn, auth_user.user_id)
        .map_err(|e| {
            tracing::error!("Failed to get usage stats: {}", e);
            // Continue without headers on error
            (0, 0)
        })
        .unwrap_or((0, 0));

    // Store usage info in request extensions for response headers
    req.extensions_mut().insert(RateLimitInfo {
        minute_limit: state.config.rate_limit_per_minute,
        minute_remaining: state.config.rate_limit_per_minute.saturating_sub(minute_count),
        daily_limit: state.config.daily_message_quota,
        daily_remaining: state.config.daily_message_quota.saturating_sub(daily_count),
    });

    // Continue to handler
    Ok(next.run(req).await)
}

/// Rate limit information to add to response headers
#[derive(Clone, Debug)]
pub struct RateLimitInfo {
    pub minute_limit: u64,
    pub minute_remaining: u64,
    pub daily_limit: u64,
    pub daily_remaining: u64,
}

/// Add rate limit headers to response
///
/// Should be called from handlers after processing request
pub fn add_rate_limit_headers(headers: &mut HeaderMap, info: &RateLimitInfo) {
    let now = chrono::Utc::now().timestamp();
    let minute_reset = now + 60;
    let daily_reset = now + 86400;

    headers.insert(
        "X-RateLimit-Limit-Minute",
        info.minute_limit.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining-Minute",
        info.minute_remaining.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset-Minute",
        minute_reset.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Limit-Daily",
        info.daily_limit.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining-Daily",
        info.daily_remaining.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset-Daily",
        daily_reset.to_string().parse().unwrap(),
    );
}
