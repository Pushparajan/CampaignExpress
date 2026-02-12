//! Token-bucket / sliding-window rate limiter backed by DashMap.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Rate-limit configuration for a tier or tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

/// Per-key sliding-window counters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: DateTime<Utc>,
    pub minute_count: u32,
    pub minute_window_start: DateTime<Utc>,
}

/// Result returned by `check_rate_limit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub limit: u32,
}

/// In-memory rate limiter with per-tenant overrides.
pub struct RateLimiter {
    entries: DashMap<String, RateLimitEntry>,
    default_config: RateLimitConfig,
    tenant_configs: DashMap<Uuid, RateLimitConfig>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given default config.
    pub fn new(default_config: RateLimitConfig) -> Self {
        Self {
            entries: DashMap::new(),
            default_config,
            tenant_configs: DashMap::new(),
        }
    }

    /// Override the rate-limit config for a specific tenant.
    pub fn set_tenant_config(&self, tenant_id: Uuid, config: RateLimitConfig) {
        self.tenant_configs.insert(tenant_id, config);
    }

    /// Check (and consume) a request against the rate limit for `key`.
    pub fn check_rate_limit(&self, key: &str, tenant_id: Option<Uuid>) -> RateLimitResult {
        let config = tenant_id
            .and_then(|id| self.tenant_configs.get(&id).map(|c| c.clone()))
            .unwrap_or_else(|| self.default_config.clone());

        let now = Utc::now();

        let mut entry = self
            .entries
            .entry(key.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                window_start: now,
                minute_count: 0,
                minute_window_start: now,
            });

        // Reset per-second window if expired.
        if now.signed_duration_since(entry.window_start) >= Duration::seconds(1) {
            entry.count = 0;
            entry.window_start = now;
        }

        // Reset per-minute window if expired.
        if now.signed_duration_since(entry.minute_window_start) >= Duration::minutes(1) {
            entry.minute_count = 0;
            entry.minute_window_start = now;
        }

        let second_limit = config.burst_size.max(config.requests_per_second);
        let second_ok = entry.count < second_limit;
        let minute_ok = entry.minute_count < config.requests_per_minute;

        if second_ok && minute_ok {
            entry.count += 1;
            entry.minute_count += 1;

            let remaining = (second_limit - entry.count).min(config.requests_per_minute - entry.minute_count);
            RateLimitResult {
                allowed: true,
                remaining,
                reset_at: entry.window_start + Duration::seconds(1),
                limit: second_limit,
            }
        } else {
            let reset_at = if !second_ok {
                entry.window_start + Duration::seconds(1)
            } else {
                entry.minute_window_start + Duration::minutes(1)
            };
            RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at,
                limit: second_limit,
            }
        }
    }

    /// Read current usage for a key (if any).
    pub fn get_usage(&self, key: &str) -> Option<RateLimitEntry> {
        self.entries.get(key).map(|e| e.value().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_basic() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_second: 5,
            requests_per_minute: 100,
            burst_size: 5,
        });

        // First 5 requests should be allowed.
        for i in 0..5 {
            let result = limiter.check_rate_limit("user:1", None);
            assert!(result.allowed, "request {i} should be allowed");
        }

        // 6th request should be denied (per-second limit).
        let result = limiter.check_rate_limit("user:1", None);
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
    }

    #[test]
    fn test_burst_handling() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_second: 2,
            requests_per_minute: 1000,
            burst_size: 10,
        });

        // Burst size (10) should be the effective per-second cap.
        for i in 0..10 {
            let result = limiter.check_rate_limit("burst_key", None);
            assert!(result.allowed, "burst request {i} should be allowed");
        }

        // 11th should be denied.
        let result = limiter.check_rate_limit("burst_key", None);
        assert!(!result.allowed);

        // Usage should be recorded.
        let usage = limiter.get_usage("burst_key").unwrap();
        assert_eq!(usage.count, 10);
    }
}
