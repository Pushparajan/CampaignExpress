//! Quiet hours — prevents messaging during user-configured do-not-disturb times.

use chrono::{NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    pub user_id: Uuid,
    pub enabled: bool,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub timezone: String,
    pub override_for_transactional: bool,
}

pub struct QuietHoursEngine {
    configs: dashmap::DashMap<Uuid, QuietHoursConfig>,
}

impl QuietHoursEngine {
    pub fn new() -> Self {
        Self {
            configs: dashmap::DashMap::new(),
        }
    }

    pub fn set_config(&self, config: QuietHoursConfig) {
        self.configs.insert(config.user_id, config);
    }

    pub fn is_quiet(&self, user_id: &Uuid, is_transactional: bool) -> bool {
        if let Some(config) = self.configs.get(user_id) {
            if !config.enabled {
                return false;
            }
            if is_transactional && config.override_for_transactional {
                return false;
            }
            let now = Utc::now();
            let current_time =
                NaiveTime::from_hms_opt(now.hour(), now.minute(), 0).unwrap_or_default();

            if config.start_time <= config.end_time {
                current_time >= config.start_time && current_time < config.end_time
            } else {
                current_time >= config.start_time || current_time < config.end_time
            }
        } else {
            false
        }
    }
}

impl Default for QuietHoursEngine {
    fn default() -> Self {
        Self::new()
    }
}
