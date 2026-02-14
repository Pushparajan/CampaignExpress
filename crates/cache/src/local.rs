//! In-process LRU cache backed by DashMap for lock-free concurrent access.
//! Serves as L1 cache in front of Redis to reduce network round trips.

use campaign_core::types::UserProfile;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct CacheEntry {
    profile: Arc<UserProfile>,
    inserted_at: Instant,
}

/// Lock-free local cache for frequently accessed user profiles.
/// Returns Arc<UserProfile> to avoid cloning on every cache hit.
pub struct LocalCache {
    store: Arc<DashMap<String, CacheEntry>>,
    ttl: Duration,
    max_entries: usize,
}

impl LocalCache {
    pub fn new(ttl_secs: u64, max_entries: usize) -> Self {
        Self {
            store: Arc::new(DashMap::with_capacity(max_entries)),
            ttl: Duration::from_secs(ttl_secs),
            max_entries,
        }
    }

    /// Get a profile from the local cache, returns None if expired or missing.
    /// Returns Arc to avoid cloning on every cache hit (critical for 50M req/hr).
    pub fn get(&self, user_id: &str) -> Option<Arc<UserProfile>> {
        let entry = self.store.get(user_id)?;
        if entry.inserted_at.elapsed() > self.ttl {
            drop(entry);
            self.store.remove(user_id);
            return None;
        }
        Some(Arc::clone(&entry.profile))
    }

    /// Insert or update a profile in the local cache.
    pub fn put(&self, user_id: String, profile: UserProfile) {
        self.put_arc(user_id, Arc::new(profile));
    }

    /// Insert or update with a pre-wrapped Arc (avoids double-Arc on L2 backfill).
    pub fn put_arc(&self, user_id: String, profile: Arc<UserProfile>) {
        // If at capacity and key doesn't exist, evict one expired entry first
        if self.store.len() >= self.max_entries && !self.store.contains_key(&user_id) {
            self.evict_one_expired();
            // If still full after eviction attempt, skip insert
            if self.store.len() >= self.max_entries {
                return;
            }
        }
        self.store.insert(
            user_id,
            CacheEntry {
                profile,
                inserted_at: Instant::now(),
            },
        );
    }

    /// Evict a single expired entry (fast path for put under pressure).
    fn evict_one_expired(&self) {
        let mut to_remove = None;
        for entry in self.store.iter() {
            if entry.value().inserted_at.elapsed() > self.ttl {
                to_remove = Some(entry.key().clone());
                break;
            }
        }
        if let Some(key) = to_remove {
            self.store.remove(&key);
        }
    }

    /// Remove expired entries. Call this periodically from a background task.
    pub fn evict_expired(&self) -> usize {
        let before = self.store.len();
        self.store
            .retain(|_, entry| entry.inserted_at.elapsed() <= self.ttl);
        before - self.store.len()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
