//! Redis cluster cache client for user profiles.
//! Two-tier caching: LocalCache (L1) -> Redis (L2).

use crate::local::LocalCache;
use campaign_core::config::RedisConfig;
use campaign_core::types::UserProfile;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Redis-backed distributed cache with local L1 layer.
pub struct RedisCache {
    conn: Mutex<MultiplexedConnection>,
    local: Arc<LocalCache>,
    ttl_secs: u64,
}

impl RedisCache {
    /// Connect to Redis (single node or cluster).
    pub async fn new(config: &RedisConfig) -> anyhow::Result<Self> {
        let url = config
            .urls
            .first()
            .cloned()
            .unwrap_or_else(|| "redis://localhost:6379".to_string());

        info!(url = %url, "Connecting to Redis");

        let client = redis::Client::open(url.as_str())?;

        // Create a persistent multiplexed connection (no per-request connect overhead)
        let mut conn = client.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        info!(response = %pong, "Redis connection established");

        let local = Arc::new(LocalCache::new(
            config.ttl_secs / 2, // L1 TTL is half of L2
            1_000_000,           // 1M entries in local cache
        ));

        Ok(Self {
            conn: Mutex::new(conn),
            local,
            ttl_secs: config.ttl_secs,
        })
    }

    /// Get a user profile. Checks L1 local cache first, then Redis.
    pub async fn get_profile(&self, user_id: &str) -> anyhow::Result<Option<UserProfile>> {
        // L1 check — returns Arc, zero-copy
        if let Some(profile) = self.local.get(user_id) {
            metrics::counter!("cache.l1.hit").increment(1);
            // Arc::unwrap_or_clone: cheap if sole owner, clone if shared
            return Ok(Some(Arc::unwrap_or_clone(profile)));
        }
        metrics::counter!("cache.l1.miss").increment(1);

        // L2 Redis check — reuse persistent connection
        let key = format!("profile:{user_id}");
        let data: Option<String> = {
            let mut conn = self.conn.lock().await;
            conn.get(&key).await?
        };

        match data {
            Some(json) => {
                let profile: UserProfile = serde_json::from_str(&json)?;
                let arc_profile = Arc::new(profile);
                // Populate L1 with Arc (no extra clone)
                self.local
                    .put_arc(user_id.to_string(), Arc::clone(&arc_profile));
                metrics::counter!("cache.l2.hit").increment(1);
                Ok(Some(Arc::unwrap_or_clone(arc_profile)))
            }
            None => {
                metrics::counter!("cache.l2.miss").increment(1);
                debug!(user_id = user_id, "Cache miss for user profile");
                Ok(None)
            }
        }
    }

    /// Store a user profile in both L1 and L2 caches.
    pub async fn put_profile(&self, user_id: &str, profile: &UserProfile) -> anyhow::Result<()> {
        let json = serde_json::to_string(profile)?;
        let key = format!("profile:{user_id}");

        {
            let mut conn = self.conn.lock().await;
            conn.set_ex::<_, _, ()>(&key, &json, self.ttl_secs).await?;
        }

        // Update L1
        self.local.put(user_id.to_string(), profile.clone());

        Ok(())
    }

    /// Get a default profile for unknown users.
    pub fn default_profile(user_id: &str) -> UserProfile {
        UserProfile {
            user_id: user_id.to_string(),
            ..Default::default()
        }
    }

    /// Run periodic maintenance (L1 eviction).
    pub async fn maintenance(&self) {
        let evicted = self.local.evict_expired();
        if evicted > 0 {
            debug!(evicted = evicted, "Local cache eviction complete");
        }
    }

    pub fn local_cache_size(&self) -> usize {
        self.local.len()
    }
}
