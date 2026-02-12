//! Send-time optimization — predicts the best time to send each user a message.

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEngagementProfile {
    pub user_id: Uuid,
    pub timezone: String,
    pub hourly_open_rates: [f32; 24],
    pub day_of_week_rates: [f32; 7],
    pub last_open: Option<DateTime<Utc>>,
    pub total_messages: u64,
    pub total_opens: u64,
    pub average_time_to_open_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTimeRecommendation {
    pub user_id: Uuid,
    pub recommended_time: DateTime<Utc>,
    pub confidence: f32,
    pub predicted_open_rate: f32,
    pub method: OptimizationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationMethod {
    PersonalOptimal,
    CohortBased,
    GlobalBest,
    Fallback,
}

pub struct SendTimeOptimizer {
    profiles: dashmap::DashMap<Uuid, UserEngagementProfile>,
    global_hourly_rates: [f32; 24],
}

impl SendTimeOptimizer {
    pub fn new() -> Self {
        let global_hourly_rates = [
            0.02, 0.01, 0.01, 0.01, 0.01, 0.02, 0.04, 0.06, 0.08, 0.09, 0.08, 0.07, 0.06, 0.05,
            0.05, 0.05, 0.05, 0.06, 0.06, 0.05, 0.04, 0.03, 0.03, 0.02,
        ];
        Self {
            profiles: dashmap::DashMap::new(),
            global_hourly_rates,
        }
    }

    pub fn update_profile(&self, profile: UserEngagementProfile) {
        self.profiles.insert(profile.user_id, profile);
    }

    pub fn recommend(&self, user_id: &Uuid) -> SendTimeRecommendation {
        if let Some(profile) = self.profiles.get(user_id) {
            if profile.total_messages >= 50 {
                let best_hour = profile
                    .hourly_open_rates
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i)
                    .unwrap_or(10);

                let now = Utc::now();
                let target = now
                    .date_naive()
                    .and_time(NaiveTime::from_hms_opt(best_hour as u32, 0, 0).unwrap_or_default());
                let recommended = DateTime::<Utc>::from_naive_utc_and_offset(target, Utc);

                return SendTimeRecommendation {
                    user_id: *user_id,
                    recommended_time: recommended,
                    confidence: 0.85,
                    predicted_open_rate: profile.hourly_open_rates[best_hour],
                    method: OptimizationMethod::PersonalOptimal,
                };
            }
        }

        let best_hour = self
            .global_hourly_rates
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(9);

        let now = Utc::now();
        let target = now
            .date_naive()
            .and_time(NaiveTime::from_hms_opt(best_hour as u32, 0, 0).unwrap_or_default());
        let recommended = DateTime::<Utc>::from_naive_utc_and_offset(target, Utc);

        SendTimeRecommendation {
            user_id: *user_id,
            recommended_time: recommended,
            confidence: 0.5,
            predicted_open_rate: self.global_hourly_rates[best_hour],
            method: OptimizationMethod::GlobalBest,
        }
    }
}

impl Default for SendTimeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
