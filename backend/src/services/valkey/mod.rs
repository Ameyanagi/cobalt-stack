pub mod blacklist;
pub mod rate_limit;

pub use blacklist::{add_to_blacklist, is_blacklisted};
pub use rate_limit::{check_rate_limit, reset_rate_limit};

use redis::Client;
use std::sync::Arc;

/// Valkey connection manager
pub struct ValkeyManager {
    client: Arc<Client>,
}

impl ValkeyManager {
    /// Create a new Valkey manager
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let client = Client::open(url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> anyhow::Result<redis::Connection> {
        Ok(self.client.get_connection()?)
    }
}
