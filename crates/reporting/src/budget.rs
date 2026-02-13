//! Budget tracking, pacing, and ROI/ROAS calculation for campaigns.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Tracks how a campaign's budget is allocated and consumed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    pub campaign_id: Uuid,
    pub total_budget: f64,
    pub daily_budget: f64,
    pub spent_total: f64,
    pub spent_today: f64,
    pub remaining: f64,
    pub pacing_status: PacingStatus,
    pub currency: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Describes whether a campaign is spending at the expected rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacingStatus {
    OnTrack,
    Underspending,
    Overspending,
    Exhausted,
    NotStarted,
}

#[allow(clippy::derivable_impls)]
impl Default for PacingStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

/// A single spend event against a campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendRecord {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub amount: f64,
    pub channel: String,
    /// One of "impression", "click", "conversion".
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
}

/// An alert generated when budget thresholds are crossed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub alert_type: BudgetAlertType,
    pub threshold_percent: f64,
    pub current_percent: f64,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
}

/// The kind of budget alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetAlertType {
    /// Total spend >= 80% of total budget.
    NearingLimit,
    /// Daily spend exceeds daily budget.
    OverDailyBudget,
    /// Total spend exceeds total budget.
    OverTotalBudget,
    /// Spend pacing is behind schedule.
    PacingBehind,
    /// Spend pacing is ahead of schedule.
    PacingAhead,
    /// Budget is fully exhausted (>= 100%).
    BudgetExhausted,
}

/// Return‐on‐ad‐spend report for a campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoasReport {
    pub campaign_id: Uuid,
    pub total_spend: f64,
    pub total_revenue: f64,
    /// `revenue / spend` (0.0 when spend is zero).
    pub roas: f64,
    /// `(revenue - spend) / spend * 100` (0.0 when spend is zero).
    pub roi_percent: f64,
    pub cost_per_acquisition: f64,
    pub cost_per_click: f64,
    pub cost_per_mille: f64,
    pub conversions: u64,
    pub clicks: u64,
    pub impressions: u64,
    pub computed_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// BudgetTracker
// ---------------------------------------------------------------------------

/// Concurrent, lock‐free budget tracker backed by `DashMap`.
pub struct BudgetTracker {
    /// campaign_id -> allocation
    allocations: DashMap<Uuid, BudgetAllocation>,
    /// campaign_id -> spend records
    spend_records: DashMap<Uuid, Vec<SpendRecord>>,
    /// campaign_id -> alerts
    alerts: DashMap<Uuid, Vec<BudgetAlert>>,
}

impl BudgetTracker {
    /// Create a new, empty tracker.
    pub fn new() -> Self {
        Self {
            allocations: DashMap::new(),
            spend_records: DashMap::new(),
            alerts: DashMap::new(),
        }
    }

    /// Create or update a budget allocation for a campaign.
    pub fn set_budget(
        &self,
        campaign_id: Uuid,
        total: f64,
        daily: f64,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) {
        let now = Utc::now();
        let pacing_status = if now < start {
            PacingStatus::NotStarted
        } else {
            PacingStatus::OnTrack
        };

        let allocation = BudgetAllocation {
            campaign_id,
            total_budget: total,
            daily_budget: daily,
            spent_total: 0.0,
            spent_today: 0.0,
            remaining: total,
            pacing_status,
            currency: "USD".to_string(),
            start_date: start,
            end_date: end,
            updated_at: now,
        };

        self.allocations.insert(campaign_id, allocation);
    }

    /// Record a spend event and check alert conditions.
    pub fn record_spend(&self, campaign_id: Uuid, amount: f64, channel: &str, event_type: &str) {
        let now = Utc::now();

        let record = SpendRecord {
            id: Uuid::new_v4(),
            campaign_id,
            amount,
            channel: channel.to_string(),
            event_type: event_type.to_string(),
            timestamp: now,
        };

        // Append spend record.
        self.spend_records
            .entry(campaign_id)
            .or_default()
            .push(record);

        // Update the allocation totals.
        if let Some(mut alloc) = self.allocations.get_mut(&campaign_id) {
            alloc.spent_total += amount;
            alloc.spent_today += amount;
            alloc.remaining = alloc.total_budget - alloc.spent_total;
            alloc.updated_at = now;

            // --- alert checks ---------------------------------------------------
            let spend_pct = if alloc.total_budget > 0.0 {
                alloc.spent_total / alloc.total_budget * 100.0
            } else {
                0.0
            };

            // Budget exhausted (>= 100%).
            if alloc.spent_total >= alloc.total_budget {
                alloc.pacing_status = PacingStatus::Exhausted;
                self.push_alert(
                    campaign_id,
                    BudgetAlertType::BudgetExhausted,
                    100.0,
                    spend_pct,
                    format!(
                        "Campaign {} budget exhausted ({:.1}% spent)",
                        campaign_id, spend_pct
                    ),
                );
            } else if spend_pct >= 80.0 {
                // Nearing limit (>= 80%).
                self.push_alert(
                    campaign_id,
                    BudgetAlertType::NearingLimit,
                    80.0,
                    spend_pct,
                    format!(
                        "Campaign {} nearing budget limit ({:.1}% spent)",
                        campaign_id, spend_pct
                    ),
                );
            }

            // Daily budget exceeded.
            if alloc.spent_today > alloc.daily_budget {
                let daily_pct = if alloc.daily_budget > 0.0 {
                    alloc.spent_today / alloc.daily_budget * 100.0
                } else {
                    0.0
                };
                self.push_alert(
                    campaign_id,
                    BudgetAlertType::OverDailyBudget,
                    100.0,
                    daily_pct,
                    format!(
                        "Campaign {} exceeded daily budget ({:.1}% of daily)",
                        campaign_id, daily_pct
                    ),
                );
            }
        }
    }

    /// Return a snapshot of the allocation for a campaign, if one exists.
    pub fn get_allocation(&self, campaign_id: &Uuid) -> Option<BudgetAllocation> {
        self.allocations.get(campaign_id).map(|r| r.clone())
    }

    /// Compute current pacing status for a campaign.
    pub fn calculate_pacing(&self, campaign_id: &Uuid) -> Option<PacingStatus> {
        let alloc = self.allocations.get(campaign_id)?;
        let now = Utc::now();

        if now < alloc.start_date {
            return Some(PacingStatus::NotStarted);
        }
        if alloc.spent_total >= alloc.total_budget {
            return Some(PacingStatus::Exhausted);
        }

        let total_days = (alloc.end_date - alloc.start_date).num_seconds().max(1) as f64;
        let elapsed_days = (now - alloc.start_date).num_seconds().max(0) as f64;
        let elapsed_fraction = elapsed_days / total_days;

        let spend_fraction = if alloc.total_budget > 0.0 {
            alloc.spent_total / alloc.total_budget
        } else {
            0.0
        };

        let status = if spend_fraction > elapsed_fraction * 1.1 {
            PacingStatus::Overspending
        } else if spend_fraction < elapsed_fraction * 0.8 {
            PacingStatus::Underspending
        } else {
            PacingStatus::OnTrack
        };

        Some(status)
    }

    /// Return all alerts for a campaign.
    pub fn get_alerts(&self, campaign_id: &Uuid) -> Vec<BudgetAlert> {
        self.alerts
            .get(campaign_id)
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Acknowledge (mark as seen) a specific alert. Returns `true` if found.
    pub fn acknowledge_alert(&self, alert_id: &Uuid) -> bool {
        for mut entry in self.alerts.iter_mut() {
            for alert in entry.value_mut().iter_mut() {
                if alert.id == *alert_id {
                    alert.acknowledged = true;
                    return true;
                }
            }
        }
        false
    }

    /// Build a ROAS / ROI report for a campaign given external revenue data.
    pub fn calculate_roas(
        &self,
        campaign_id: &Uuid,
        revenue: f64,
        conversions: u64,
        clicks: u64,
        impressions: u64,
    ) -> Option<RoasReport> {
        let alloc = self.allocations.get(campaign_id)?;
        let spend = alloc.spent_total;

        let roas = if spend > 0.0 { revenue / spend } else { 0.0 };
        let roi_percent = if spend > 0.0 {
            (revenue - spend) / spend * 100.0
        } else {
            0.0
        };
        let cost_per_acquisition = if conversions > 0 {
            spend / conversions as f64
        } else {
            0.0
        };
        let cost_per_click = if clicks > 0 {
            spend / clicks as f64
        } else {
            0.0
        };
        let cost_per_mille = if impressions > 0 {
            spend / impressions as f64 * 1000.0
        } else {
            0.0
        };

        Some(RoasReport {
            campaign_id: *campaign_id,
            total_spend: spend,
            total_revenue: revenue,
            roas,
            roi_percent,
            cost_per_acquisition,
            cost_per_click,
            cost_per_mille,
            conversions,
            clicks,
            impressions,
            computed_at: Utc::now(),
        })
    }

    /// Per‐day spend totals for the last `days` days (most recent first).
    pub fn get_daily_spend_breakdown(&self, campaign_id: &Uuid, days: u32) -> Vec<(String, f64)> {
        let records = match self.spend_records.get(campaign_id) {
            Some(r) => r.clone(),
            None => return Vec::new(),
        };

        let now = Utc::now();
        let mut breakdown: Vec<(String, f64)> = Vec::new();

        for i in 0..days {
            let day = (now - Duration::days(i as i64))
                .format("%Y-%m-%d")
                .to_string();
            let total: f64 = records
                .iter()
                .filter(|r| r.timestamp.format("%Y-%m-%d").to_string() == day)
                .map(|r| r.amount)
                .sum();
            breakdown.push((day, total));
        }

        breakdown
    }

    /// Aggregate spend per channel for a campaign.
    pub fn get_channel_spend_breakdown(&self, campaign_id: &Uuid) -> Vec<(String, f64)> {
        let records = match self.spend_records.get(campaign_id) {
            Some(r) => r.clone(),
            None => return Vec::new(),
        };

        let mut map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for rec in &records {
            *map.entry(rec.channel.clone()).or_insert(0.0) += rec.amount;
        }

        let mut result: Vec<(String, f64)> = map.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    // -- internal helpers ---------------------------------------------------

    fn push_alert(
        &self,
        campaign_id: Uuid,
        alert_type: BudgetAlertType,
        threshold_percent: f64,
        current_percent: f64,
        message: String,
    ) {
        let alert = BudgetAlert {
            id: Uuid::new_v4(),
            campaign_id,
            alert_type,
            threshold_percent,
            current_percent,
            message,
            triggered_at: Utc::now(),
            acknowledged: false,
        };

        self.alerts.entry(campaign_id).or_default().push(alert);
    }
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_tracker_with_budget() -> (BudgetTracker, Uuid) {
        let tracker = BudgetTracker::new();
        let cid = Uuid::new_v4();
        let start = Utc::now() - Duration::days(5);
        let end = Utc::now() + Duration::days(25);
        tracker.set_budget(cid, 10_000.0, 500.0, start, end);
        (tracker, cid)
    }

    // 1. Set budget and record spend ----------------------------------------

    #[test]
    fn test_set_budget_and_record_spend() {
        let (tracker, cid) = make_tracker_with_budget();

        tracker.record_spend(cid, 150.0, "email", "impression");
        tracker.record_spend(cid, 50.0, "social", "click");

        let alloc = tracker.get_allocation(&cid).unwrap();
        assert!((alloc.spent_total - 200.0).abs() < f64::EPSILON);
        assert!((alloc.remaining - 9_800.0).abs() < f64::EPSILON);
    }

    // 2. Alert generation ---------------------------------------------------

    #[test]
    fn test_alert_nearing_limit() {
        let (tracker, cid) = make_tracker_with_budget();

        // Spend exactly 80% of 10 000 = 8 000
        tracker.record_spend(cid, 8_000.0, "display", "impression");

        let alerts = tracker.get_alerts(&cid);
        assert!(
            alerts
                .iter()
                .any(|a| a.alert_type == BudgetAlertType::NearingLimit),
            "Expected NearingLimit alert at 80%"
        );
    }

    #[test]
    fn test_alert_budget_exhausted() {
        let (tracker, cid) = make_tracker_with_budget();

        tracker.record_spend(cid, 10_000.0, "display", "impression");

        let alerts = tracker.get_alerts(&cid);
        assert!(
            alerts
                .iter()
                .any(|a| a.alert_type == BudgetAlertType::BudgetExhausted),
            "Expected BudgetExhausted alert at 100%"
        );
    }

    #[test]
    fn test_alert_over_daily_budget() {
        let (tracker, cid) = make_tracker_with_budget();

        // daily budget = 500; spend 600 in one shot
        tracker.record_spend(cid, 600.0, "search", "click");

        let alerts = tracker.get_alerts(&cid);
        assert!(
            alerts
                .iter()
                .any(|a| a.alert_type == BudgetAlertType::OverDailyBudget),
            "Expected OverDailyBudget alert"
        );
    }

    #[test]
    fn test_acknowledge_alert() {
        let (tracker, cid) = make_tracker_with_budget();
        tracker.record_spend(cid, 8_000.0, "display", "impression");

        let alerts = tracker.get_alerts(&cid);
        let alert_id = alerts[0].id;

        assert!(tracker.acknowledge_alert(&alert_id));
        let alerts = tracker.get_alerts(&cid);
        assert!(alerts.iter().any(|a| a.id == alert_id && a.acknowledged));
    }

    // 3. Pacing calculation -------------------------------------------------

    #[test]
    fn test_pacing_on_track() {
        let tracker = BudgetTracker::new();
        let cid = Uuid::new_v4();

        // 10-day campaign, 5 days elapsed, budget 10 000 -> expected ~50% spent
        let start = Utc::now() - Duration::days(5);
        let end = Utc::now() + Duration::days(5);
        tracker.set_budget(cid, 10_000.0, 1_000.0, start, end);

        // Spend 5 000 (50%) — within [40%..55%] of elapsed 50%
        tracker.record_spend(cid, 5_000.0, "display", "impression");

        let status = tracker.calculate_pacing(&cid).unwrap();
        assert_eq!(status, PacingStatus::OnTrack);
    }

    #[test]
    fn test_pacing_overspending() {
        let tracker = BudgetTracker::new();
        let cid = Uuid::new_v4();

        let start = Utc::now() - Duration::days(2);
        let end = Utc::now() + Duration::days(8);
        tracker.set_budget(cid, 10_000.0, 1_000.0, start, end);

        // 20% elapsed but 50% spent -> overspending
        tracker.record_spend(cid, 5_000.0, "display", "impression");

        let status = tracker.calculate_pacing(&cid).unwrap();
        assert_eq!(status, PacingStatus::Overspending);
    }

    #[test]
    fn test_pacing_underspending() {
        let tracker = BudgetTracker::new();
        let cid = Uuid::new_v4();

        let start = Utc::now() - Duration::days(8);
        let end = Utc::now() + Duration::days(2);
        tracker.set_budget(cid, 10_000.0, 1_000.0, start, end);

        // 80% elapsed but only 10% spent -> underspending
        tracker.record_spend(cid, 1_000.0, "display", "impression");

        let status = tracker.calculate_pacing(&cid).unwrap();
        assert_eq!(status, PacingStatus::Underspending);
    }

    #[test]
    fn test_pacing_not_started() {
        let tracker = BudgetTracker::new();
        let cid = Uuid::new_v4();

        let start = Utc::now() + Duration::days(5);
        let end = Utc::now() + Duration::days(35);
        tracker.set_budget(cid, 10_000.0, 500.0, start, end);

        let status = tracker.calculate_pacing(&cid).unwrap();
        assert_eq!(status, PacingStatus::NotStarted);
    }

    // 4. ROAS calculation ---------------------------------------------------

    #[test]
    fn test_roas_positive_roi() {
        let (tracker, cid) = make_tracker_with_budget();
        tracker.record_spend(cid, 2_000.0, "display", "impression");

        let report = tracker
            .calculate_roas(&cid, 6_000.0, 100, 500, 50_000)
            .unwrap();

        assert!((report.roas - 3.0).abs() < f64::EPSILON);
        assert!((report.roi_percent - 200.0).abs() < f64::EPSILON);
        assert!((report.cost_per_acquisition - 20.0).abs() < f64::EPSILON);
        assert!((report.cost_per_click - 4.0).abs() < f64::EPSILON);
        assert!((report.cost_per_mille - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_roas_negative_roi() {
        let (tracker, cid) = make_tracker_with_budget();
        tracker.record_spend(cid, 5_000.0, "display", "impression");

        // Revenue < spend -> negative ROI
        let report = tracker
            .calculate_roas(&cid, 2_500.0, 50, 200, 100_000)
            .unwrap();

        assert!((report.roas - 0.5).abs() < f64::EPSILON);
        assert!((report.roi_percent - (-50.0)).abs() < f64::EPSILON);
    }

    // 5. Channel spend breakdown --------------------------------------------

    #[test]
    fn test_channel_spend_breakdown() {
        let (tracker, cid) = make_tracker_with_budget();

        tracker.record_spend(cid, 100.0, "email", "impression");
        tracker.record_spend(cid, 200.0, "social", "click");
        tracker.record_spend(cid, 50.0, "email", "click");
        tracker.record_spend(cid, 300.0, "display", "impression");

        let breakdown = tracker.get_channel_spend_breakdown(&cid);
        // Sorted alphabetically: display, email, social
        assert_eq!(breakdown.len(), 3);
        assert_eq!(breakdown[0].0, "display");
        assert!((breakdown[0].1 - 300.0).abs() < f64::EPSILON);
        assert_eq!(breakdown[1].0, "email");
        assert!((breakdown[1].1 - 150.0).abs() < f64::EPSILON);
        assert_eq!(breakdown[2].0, "social");
        assert!((breakdown[2].1 - 200.0).abs() < f64::EPSILON);
    }

    // 6. Daily spend breakdown ----------------------------------------------

    #[test]
    fn test_daily_spend_breakdown() {
        let (tracker, cid) = make_tracker_with_budget();

        // All spend happens "now" so only today has a non-zero value.
        tracker.record_spend(cid, 100.0, "email", "impression");
        tracker.record_spend(cid, 200.0, "social", "click");

        let breakdown = tracker.get_daily_spend_breakdown(&cid, 3);
        assert_eq!(breakdown.len(), 3);

        // First entry is today — should contain 300.
        assert!((breakdown[0].1 - 300.0).abs() < f64::EPSILON);
        // Yesterday and day before should be 0.
        assert!((breakdown[1].1).abs() < f64::EPSILON);
        assert!((breakdown[2].1).abs() < f64::EPSILON);
    }
}
