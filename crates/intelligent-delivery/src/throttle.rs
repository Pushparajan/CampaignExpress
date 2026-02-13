//! Message throttling — controls send rate to avoid overwhelming downstream systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleConfig {
    pub max_per_second: u64,
    pub max_per_minute: u64,
    pub burst_allowance: u64,
    pub channel_limits: std::collections::HashMap<String, u64>,
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            max_per_second: 10_000,
            max_per_minute: 500_000,
            burst_allowance: 5_000,
            channel_limits: std::collections::HashMap::new(),
        }
    }
}

pub struct MessageThrottler {
    config: ThrottleConfig,
    second_counter: AtomicU64,
    minute_counter: AtomicU64,
    last_second_reset: std::sync::Mutex<DateTime<Utc>>,
    last_minute_reset: std::sync::Mutex<DateTime<Utc>>,
}

impl MessageThrottler {
    pub fn new(config: ThrottleConfig) -> Self {
        let now = Utc::now();
        Self {
            config,
            second_counter: AtomicU64::new(0),
            minute_counter: AtomicU64::new(0),
            last_second_reset: std::sync::Mutex::new(now),
            last_minute_reset: std::sync::Mutex::new(now),
        }
    }

    pub fn try_acquire(&self) -> bool {
        self.maybe_reset_counters();
        let per_sec = self.second_counter.fetch_add(1, Ordering::Relaxed);
        let per_min = self.minute_counter.fetch_add(1, Ordering::Relaxed);

        if per_sec >= self.config.max_per_second + self.config.burst_allowance {
            self.second_counter.fetch_sub(1, Ordering::Relaxed);
            self.minute_counter.fetch_sub(1, Ordering::Relaxed);
            return false;
        }
        if per_min >= self.config.max_per_minute {
            self.second_counter.fetch_sub(1, Ordering::Relaxed);
            self.minute_counter.fetch_sub(1, Ordering::Relaxed);
            return false;
        }
        true
    }

    pub fn current_rate_per_second(&self) -> u64 {
        self.second_counter.load(Ordering::Relaxed)
    }

    fn maybe_reset_counters(&self) {
        let now = Utc::now();
        if let Ok(mut last) = self.last_second_reset.lock() {
            if (now - *last).num_seconds() >= 1 {
                self.second_counter.store(0, Ordering::Relaxed);
                *last = now;
            }
        }
        if let Ok(mut last) = self.last_minute_reset.lock() {
            if (now - *last).num_seconds() >= 60 {
                self.minute_counter.store(0, Ordering::Relaxed);
                *last = now;
            }
        }
    }
}
