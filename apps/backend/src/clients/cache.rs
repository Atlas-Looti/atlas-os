//! Redis cache layer — TTL-based caching for API responses.

use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};


/// Redis-backed cache with configurable TTLs.
#[derive(Clone)]
pub struct Cache {
    conn: ConnectionManager,
}

/// Standard TTLs for different data types.
pub struct CacheTtl;

impl CacheTtl {
    /// Token metadata (name, symbol, decimals) — rarely changes.
    pub const TOKEN_METADATA: Duration = Duration::from_secs(86400); // 24h
    /// Portfolio (balances + prices + metadata) — moderate.
    pub const PORTFOLIO: Duration = Duration::from_secs(30);
}

impl Cache {
    /// Connect to Redis and create a cache instance.
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// Get a cached value, deserializing from JSON.
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(key).await.ok()?;
        result.and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Set a value with TTL, serializing to JSON.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(value)?;
        let ttl_secs = ttl.as_secs().max(1);
        let _: () = conn.set_ex(key, &json, ttl_secs).await?;
        tracing::debug!("cache SET {key} (ttl={ttl_secs}s)");
        Ok(())
    }



    /// Build a cache key with namespace.
    pub fn key(namespace: &str, parts: &[&str]) -> String {
        format!("atlas:{}:{}", namespace, parts.join(":"))
    }
}
