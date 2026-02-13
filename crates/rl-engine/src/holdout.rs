//! Holdout control groups — automated incrementality testing with statistical rigor.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldoutGroup {
    Treatment,
    Control,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldoutConfig {
    pub campaign_id: Uuid,
    pub enabled: bool,
    pub holdout_percentage: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalityReport {
    pub campaign_id: Uuid,
    pub treatment_conversions: u64,
    pub treatment_total: u64,
    pub treatment_rate: f64,
    pub control_conversions: u64,
    pub control_total: u64,
    pub control_rate: f64,
    pub absolute_lift: f64,
    pub relative_lift: f64,
    pub p_value: f64,
    pub is_significant: bool,
    pub confidence_interval_lower: f64,
    pub confidence_interval_upper: f64,
    pub incremental_conversions: u64,
    pub incremental_revenue: f64,
    pub computed_at: DateTime<Utc>,
}

pub struct HoldoutManager {
    configs: dashmap::DashMap<Uuid, HoldoutConfig>,
    treatment_data: dashmap::DashMap<Uuid, (u64, u64)>,
    control_data: dashmap::DashMap<Uuid, (u64, u64)>,
}

impl HoldoutManager {
    pub fn new() -> Self {
        Self {
            configs: dashmap::DashMap::new(),
            treatment_data: dashmap::DashMap::new(),
            control_data: dashmap::DashMap::new(),
        }
    }

    pub fn configure(&self, config: HoldoutConfig) {
        self.configs.insert(config.campaign_id, config);
    }

    pub fn assign_group(&self, campaign_id: &Uuid, user_id: &str) -> HoldoutGroup {
        let config = self.configs.get(campaign_id);
        let holdout_pct = config.map(|c| c.holdout_percentage).unwrap_or(0.1);

        let hash = Self::hash_user(user_id);
        let bucket = (hash % 100) as f64 / 100.0;

        if bucket < holdout_pct {
            HoldoutGroup::Control
        } else {
            HoldoutGroup::Treatment
        }
    }

    pub fn record_outcome(&self, campaign_id: &Uuid, group: HoldoutGroup, converted: bool) {
        let data = match group {
            HoldoutGroup::Treatment => &self.treatment_data,
            HoldoutGroup::Control => &self.control_data,
        };
        data.entry(*campaign_id)
            .and_modify(|(total, conversions)| {
                *total += 1;
                if converted {
                    *conversions += 1;
                }
            })
            .or_insert(if converted { (1, 1) } else { (1, 0) });
    }

    pub fn get_report(&self, campaign_id: &Uuid) -> IncrementalityReport {
        let (t_total, t_conv) = self
            .treatment_data
            .get(campaign_id)
            .map(|d| *d)
            .unwrap_or((0, 0));
        let (c_total, c_conv) = self
            .control_data
            .get(campaign_id)
            .map(|d| *d)
            .unwrap_or((0, 0));

        let t_rate = if t_total > 0 {
            t_conv as f64 / t_total as f64
        } else {
            0.0
        };
        let c_rate = if c_total > 0 {
            c_conv as f64 / c_total as f64
        } else {
            0.0
        };

        let abs_lift = t_rate - c_rate;
        let rel_lift = if c_rate > 0.0 { abs_lift / c_rate } else { 0.0 };

        let p_value = Self::two_proportion_z_test(t_conv, t_total, c_conv, c_total);
        let ci_width = if t_total > 0 && c_total > 0 {
            1.96 * ((t_rate * (1.0 - t_rate) / t_total as f64)
                + (c_rate * (1.0 - c_rate) / c_total as f64))
                .sqrt()
        } else {
            0.0
        };

        let incremental_conv = ((t_rate - c_rate) * t_total as f64).max(0.0) as u64;

        IncrementalityReport {
            campaign_id: *campaign_id,
            treatment_conversions: t_conv,
            treatment_total: t_total,
            treatment_rate: t_rate,
            control_conversions: c_conv,
            control_total: c_total,
            control_rate: c_rate,
            absolute_lift: abs_lift,
            relative_lift: rel_lift,
            p_value,
            is_significant: p_value < 0.05,
            confidence_interval_lower: abs_lift - ci_width,
            confidence_interval_upper: abs_lift + ci_width,
            incremental_conversions: incremental_conv,
            incremental_revenue: incremental_conv as f64 * 50.0,
            computed_at: Utc::now(),
        }
    }

    fn hash_user(user_id: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in user_id.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    fn two_proportion_z_test(x1: u64, n1: u64, x2: u64, n2: u64) -> f64 {
        if n1 == 0 || n2 == 0 {
            return 1.0;
        }
        let p1 = x1 as f64 / n1 as f64;
        let p2 = x2 as f64 / n2 as f64;
        let p = (x1 + x2) as f64 / (n1 + n2) as f64;
        let se = (p * (1.0 - p) * (1.0 / n1 as f64 + 1.0 / n2 as f64)).sqrt();
        if se == 0.0 {
            return 1.0;
        }
        let z = (p1 - p2).abs() / se;
        // Approximate p-value from z-score using error function approximation
        let t = 1.0 / (1.0 + 0.2316419 * z);
        let d = 0.3989422804014327;
        let p_val = d
            * (-z * z / 2.0).exp()
            * (t * (0.3193815
                + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274)))));
        2.0 * p_val
    }
}

impl Default for HoldoutManager {
    fn default() -> Self {
        Self::new()
    }
}
