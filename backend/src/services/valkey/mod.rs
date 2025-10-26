//! Valkey/Redis caching and state management services.
//!
//! This module provides Redis-compatible caching services using Valkey,
//! including token blacklisting and rate limiting. Valkey is a high-performance
//! Redis fork maintained by the Linux Foundation.
//!
//! # Modules
//!
//! - **blacklist**: JWT access token revocation via blacklist
//! - **rate_limit**: Login attempt rate limiting by IP address
//!
//! # Connection Management
//!
//! [`ValkeyManager`] provides connection pooling and management for Valkey operations.
//! In production, consider using a connection pool like `r2d2` or `bb8-redis`.
//!
//! # Configuration
//!
//! Set the `VALKEY_URL` environment variable:
//! ```bash
//! VALKEY_URL=redis://localhost:6379
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack_backend::services::valkey::ValkeyManager;
//!
//! # fn example() -> anyhow::Result<()> {
//! let manager = ValkeyManager::new("redis://127.0.0.1:6379")?;
//! let mut conn = manager.get_connection()?;
//!
//! // Use connection for blacklist or rate limit operations
//! # Ok(())
//! # }
//! ```
//!
//! # Why Valkey?
//!
//! - **Performance**: Drop-in Redis replacement with same protocol
//! - **Open Source**: Linux Foundation stewardship, no licensing concerns
//! - **Compatibility**: Uses redis-rs crate, fully compatible with Redis
//! - **Future-proof**: Active development and community support

pub mod blacklist;
pub mod rate_limit;

pub use blacklist::{add_to_blacklist, is_blacklisted};
pub use rate_limit::{check_rate_limit, reset_rate_limit};

use redis::Client;
use std::sync::Arc;

/// Connection manager for Valkey/Redis operations.
///
/// Provides connection creation and management for Valkey services.
/// For production use, consider using a connection pool library like
/// `r2d2-redis` or `bb8-redis` for better performance and resource management.
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::services::valkey::ValkeyManager;
///
/// # fn example() -> anyhow::Result<()> {
/// // Create manager from environment variable
/// let url = std::env::var("VALKEY_URL").unwrap_or("redis://127.0.0.1:6379".to_string());
/// let manager = ValkeyManager::new(&url)?;
///
/// // Get connection for operations
/// let mut conn = manager.get_connection()?;
/// # Ok(())
/// # }
/// ```
pub struct ValkeyManager {
    client: Arc<Client>,
}

impl ValkeyManager {
    /// Create a new Valkey connection manager.
    ///
    /// # Arguments
    ///
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Returns
    ///
    /// - `Ok(ValkeyManager)` - Manager successfully created
    /// - `Err(_)` - Invalid URL format or connection parameters
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cobalt_stack_backend::services::valkey::ValkeyManager;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// // Local development
    /// let manager = ValkeyManager::new("redis://127.0.0.1:6379")?;
    ///
    /// // With password
    /// let manager = ValkeyManager::new("redis://:password@localhost:6379")?;
    ///
    /// // Unix socket
    /// let manager = ValkeyManager::new("redis+unix:///var/run/redis.sock")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let client = Client::open(url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Get a connection to Valkey/Redis.
    ///
    /// Creates a new connection from the client. For production applications,
    /// consider using a connection pool to avoid connection overhead.
    ///
    /// # Returns
    ///
    /// - `Ok(Connection)` - Active Redis connection
    /// - `Err(_)` - Connection failed (network, authentication, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cobalt_stack_backend::services::valkey::ValkeyManager;
    /// use cobalt_stack_backend::services::valkey::blacklist;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let manager = ValkeyManager::new("redis://127.0.0.1:6379")?;
    /// let mut conn = manager.get_connection()?;
    ///
    /// // Use connection for operations
    /// blacklist::add_to_blacklist(&mut conn, "token123", 1800)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_connection(&self) -> anyhow::Result<redis::Connection> {
        Ok(self.client.get_connection()?)
    }
}
