//! In-process LRU cache backed by DashMap for lock-free concurrent access.
//! Serves as L1 cache in front of Redis to reduce network round trips.

use campaign_core::types::UserProfile;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct CacheEntry {
    profile: UserProfile,
    inserted_at: Instant,
}

/// Lock-free local cache for frequently accessed user profiles.
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
    pub fn get(&self, user_id: &str) -> Option<UserProfile> {
        let entry = self.store.get(user_id)?;
        if entry.inserted_at.elapsed() > self.ttl {
            drop(entry);
            self.store.remove(user_id);
            return None;
        }
        Some(entry.profile.clone())
    }

    /// Insert or update a profile in the local cache.
    pub fn put(&self, user_id: String, profile: UserProfile) {
        // Simple eviction: if over capacity, skip insert (background cleanup handles this)
        if self.store.len() >= self.max_entries && !self.store.contains_key(&user_id) {
            return;
        }
        self.store.insert(
            user_id,
            CacheEntry {
                profile,
                inserted_at: Instant::now(),
            },
        );
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
