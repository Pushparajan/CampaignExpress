//! SLA/SLO tracking with proper rolling-window computation from measured
//! check data, error budget calculation, and burn-rate alerts.
//!
//! Addresses FR-OPS-007 through FR-OPS-010.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

// ─── Core Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaTarget {
    pub name: String,
    pub target_percent: f64,
    pub current_percent: f64,
    pub measurement_window: String,
    pub last_incident: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeRecord {
    pub id: Uuid,
    pub service: String,
    pub status: String,
    pub checked_at: DateTime<Utc>,
    pub response_time_ms: u64,
}

/// Error budget status for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBudget {
    pub service: String,
    pub target_percent: f64,
    pub window_days: u32,
    /// Total minutes in the window.
    pub total_minutes: f64,
    /// Allowed downtime minutes based on target.
    pub budget_minutes: f64,
    /// Downtime minutes consumed so far.
    pub consumed_minutes: f64,
    /// Remaining budget in minutes.
    pub remaining_minutes: f64,
    /// Percentage of budget consumed.
    pub consumed_percent: f64,
    /// Whether the budget is exhausted.
    pub exhausted: bool,
}

/// Burn-rate alert severity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    PageNow,
}

/// Burn-rate alert when error budget is being consumed too fast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRateAlert {
    pub service: String,
    pub severity: AlertSeverity,
    pub burn_rate: f64,
    pub window_hours: u32,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
}

// ─── SLA Tracker ────────────────────────────────────────────────────────

pub struct SlaTracker {
    targets: DashMap<String, SlaTarget>,
    uptime_records: DashMap<Uuid, UptimeRecord>,
}

impl SlaTracker {
    pub fn new() -> Self {
        info!("SLA tracker initialized");
        let tracker = Self {
            targets: DashMap::new(),
            uptime_records: DashMap::new(),
        };
        tracker.seed_demo_data();
        tracker
    }

    pub fn register_target(
        &self,
        name: String,
        target_percent: f64,
        measurement_window: String,
    ) -> SlaTarget {
        let target = SlaTarget {
            name: name.clone(),
            target_percent,
            current_percent: 100.0,
            measurement_window,
            last_incident: None,
        };
        self.targets.insert(name, target.clone());
        target
    }

    pub fn record_check(&self, service: &str, status: &str, response_time_ms: u64) -> UptimeRecord {
        let record = UptimeRecord {
            id: Uuid::new_v4(),
            service: service.to_string(),
            status: status.to_string(),
            checked_at: Utc::now(),
            response_time_ms,
        };
        self.uptime_records.insert(record.id, record.clone());

        // Recompute SLA from actual check data in rolling window
        if let Some(mut target) = self.targets.get_mut(service) {
            let window_days = parse_window_days(&target.measurement_window);
            let computed = self.compute_uptime(service, window_days);
            target.current_percent = computed;
            if status != "healthy" {
                target.last_incident = Some(Utc::now());
            }
        }

        record
    }

    /// Compute uptime percentage from actual check records within a rolling window.
    fn compute_uptime(&self, service: &str, window_days: u32) -> f64 {
        let cutoff = Utc::now() - Duration::days(window_days as i64);
        let mut total: u64 = 0;
        let mut healthy: u64 = 0;

        for entry in self.uptime_records.iter() {
            let r = entry.value();
            if r.service == service && r.checked_at >= cutoff {
                total += 1;
                if r.status == "healthy" {
                    healthy += 1;
                }
            }
        }

        if total == 0 {
            return 100.0;
        }
        healthy as f64 / total as f64 * 100.0
    }

    /// Calculate error budget for a service.
    pub fn error_budget(&self, service: &str) -> Option<ErrorBudget> {
        let target = self.targets.get(service)?;
        let window_days = parse_window_days(&target.measurement_window);
        let total_minutes = window_days as f64 * 24.0 * 60.0;
        let budget_minutes = total_minutes * (1.0 - target.target_percent / 100.0);

        // Count failure minutes from check data
        let cutoff = Utc::now() - Duration::days(window_days as i64);
        let mut failure_checks: u64 = 0;
        let mut total_checks: u64 = 0;

        for entry in self.uptime_records.iter() {
            let r = entry.value();
            if r.service == service && r.checked_at >= cutoff {
                total_checks += 1;
                if r.status != "healthy" {
                    failure_checks += 1;
                }
            }
        }

        // Estimate consumed downtime proportionally
        let consumed_minutes = if total_checks > 0 {
            (failure_checks as f64 / total_checks as f64) * total_minutes
        } else {
            0.0
        };

        let remaining = (budget_minutes - consumed_minutes).max(0.0);
        let consumed_pct = if budget_minutes > 0.0 {
            (consumed_minutes / budget_minutes * 100.0).min(100.0)
        } else {
            0.0
        };

        Some(ErrorBudget {
            service: service.to_string(),
            target_percent: target.target_percent,
            window_days,
            total_minutes,
            budget_minutes,
            consumed_minutes,
            remaining_minutes: remaining,
            consumed_percent: consumed_pct,
            exhausted: remaining <= 0.0,
        })
    }

    /// Calculate burn rate over a short window and generate alerts.
    ///
    /// Burn rate = (error rate in short window) / (allowed error rate).
    /// A burn rate of 1.0 means consuming budget exactly at pace;
    /// >1.0 means burning faster than allowed.
    pub fn check_burn_rate(&self, service: &str) -> Vec<BurnRateAlert> {
        let mut alerts = Vec::new();
        let target = match self.targets.get(service) {
            Some(t) => t.clone(),
            None => return alerts,
        };

        let allowed_error_rate = 1.0 - target.target_percent / 100.0;
        if allowed_error_rate <= 0.0 {
            return alerts;
        }

        // Check multiple windows for multi-window alerting
        let windows = [
            (1, AlertSeverity::PageNow),
            (6, AlertSeverity::Critical),
            (24, AlertSeverity::Warning),
        ];

        for (hours, severity) in windows {
            let cutoff = Utc::now() - Duration::hours(hours);
            let mut total: u64 = 0;
            let mut failures: u64 = 0;

            for entry in self.uptime_records.iter() {
                let r = entry.value();
                if r.service == service && r.checked_at >= cutoff {
                    total += 1;
                    if r.status != "healthy" {
                        failures += 1;
                    }
                }
            }

            if total == 0 {
                continue;
            }

            let observed_error_rate = failures as f64 / total as f64;
            let burn_rate = observed_error_rate / allowed_error_rate;

            let threshold = match severity {
                AlertSeverity::PageNow => 14.4,
                AlertSeverity::Critical => 6.0,
                AlertSeverity::Warning => 3.0,
                _ => 1.0,
            };

            if burn_rate >= threshold {
                let msg = format!(
                    "{}: burn rate {:.1}x over {}h window (threshold {:.1}x). \
                     Error rate {:.4}% vs allowed {:.4}%",
                    service,
                    burn_rate,
                    hours,
                    threshold,
                    observed_error_rate * 100.0,
                    allowed_error_rate * 100.0,
                );
                warn!(
                    service = service,
                    burn_rate = burn_rate,
                    window_hours = hours,
                    "SLO burn rate alert"
                );
                alerts.push(BurnRateAlert {
                    service: service.to_string(),
                    severity,
                    burn_rate,
                    window_hours: hours as u32,
                    message: msg,
                    triggered_at: Utc::now(),
                });
            }
        }

        alerts
    }

    pub fn get_sla_report(&self) -> serde_json::Value {
        let now = Utc::now();

        // Snapshot target data first to avoid holding DashMap read-locks while
        // calling methods that also access `self.targets`.
        let target_snapshots: Vec<SlaTarget> =
            self.targets.iter().map(|r| r.value().clone()).collect();

        let targets: Vec<serde_json::Value> = target_snapshots
            .iter()
            .map(|t| {
                let window_days = parse_window_days(&t.measurement_window);
                let computed_uptime = self.compute_uptime(&t.name, window_days);
                let meeting_sla = computed_uptime >= t.target_percent;

                // Error budget
                let budget = self.error_budget(&t.name);

                serde_json::json!({
                    "name": t.name,
                    "target_percent": t.target_percent,
                    "current_percent": computed_uptime,
                    "measurement_window": t.measurement_window,
                    "meeting_sla": meeting_sla,
                    "last_incident": t.last_incident,
                    "error_budget": budget,
                })
            })
            .collect();

        let total_checks = self.uptime_records.len();
        let healthy_checks = self
            .uptime_records
            .iter()
            .filter(|r| r.value().status == "healthy")
            .count();
        let overall_uptime = if total_checks > 0 {
            healthy_checks as f64 / total_checks as f64 * 100.0
        } else {
            100.0
        };

        // Collect burn rate alerts across all services
        let burn_rate_alerts: Vec<BurnRateAlert> = target_snapshots
            .iter()
            .flat_map(|t| self.check_burn_rate(&t.name))
            .collect();

        serde_json::json!({
            "generated_at": now,
            "overall_uptime_percent": overall_uptime,
            "total_checks": total_checks,
            "targets": targets,
            "burn_rate_alerts": burn_rate_alerts,
        })
    }

    fn seed_demo_data(&self) {
        let now = Utc::now();

        let services = vec![
            ("API Gateway", 99.99, "30d"),
            ("Bidding Engine", 99.95, "30d"),
            ("NATS Cluster", 99.99, "30d"),
            ("Redis Cluster", 99.95, "30d"),
            ("ClickHouse", 99.90, "30d"),
            ("NPU Engine", 99.90, "30d"),
        ];

        for (name, target, window) in &services {
            self.register_target(name.to_string(), *target, window.to_string());
        }

        // Seed 24h of healthy checks (1 per hour per service)
        for i in 0..24 {
            let check_time = now - Duration::hours(i);
            for (service, _, _) in &services {
                let record = UptimeRecord {
                    id: Uuid::new_v4(),
                    service: service.to_string(),
                    status: "healthy".to_string(),
                    checked_at: check_time,
                    response_time_ms: 5 + (i as u64 % 10),
                };
                self.uptime_records.insert(record.id, record);
            }
        }

        // Add a few degraded checks for realism
        let degraded_record = UptimeRecord {
            id: Uuid::new_v4(),
            service: "Redis Cluster".to_string(),
            status: "degraded".to_string(),
            checked_at: now - Duration::hours(72),
            response_time_ms: 450,
        };
        self.uptime_records
            .insert(degraded_record.id, degraded_record);

        let degraded_record2 = UptimeRecord {
            id: Uuid::new_v4(),
            service: "ClickHouse".to_string(),
            status: "degraded".to_string(),
            checked_at: now - Duration::hours(48),
            response_time_ms: 800,
        };
        self.uptime_records
            .insert(degraded_record2.id, degraded_record2);

        // Recompute current_percent from actual data for all targets.
        // Collect first to avoid holding iterator read-lock while acquiring write-lock.
        let target_info: Vec<(String, u32)> = self
            .targets
            .iter()
            .map(|e| {
                (
                    e.key().clone(),
                    parse_window_days(&e.value().measurement_window),
                )
            })
            .collect();
        for (name, window_days) in target_info {
            let computed = self.compute_uptime(&name, window_days);
            if let Some(mut target) = self.targets.get_mut(&name) {
                target.current_percent = computed;
            }
        }
    }
}

impl Default for SlaTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a window string like "30d" into days.
fn parse_window_days(window: &str) -> u32 {
    let trimmed = window.trim_end_matches('d');
    trimmed.parse::<u32>().unwrap_or(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_tracking_from_checks() {
        let tracker = SlaTracker {
            targets: DashMap::new(),
            uptime_records: DashMap::new(),
        };
        tracker.register_target("Test Service".to_string(), 99.9, "30d".to_string());

        // 99 healthy, 1 unhealthy = 99% uptime
        for _ in 0..99 {
            tracker.record_check("Test Service", "healthy", 5);
        }
        tracker.record_check("Test Service", "degraded", 500);

        let target = tracker.targets.get("Test Service").unwrap();
        assert!((target.current_percent - 99.0).abs() < 0.1);
        assert!(target.last_incident.is_some());
    }

    #[test]
    fn test_error_budget_calculation() {
        let tracker = SlaTracker {
            targets: DashMap::new(),
            uptime_records: DashMap::new(),
        };
        tracker.register_target("Budget Test".to_string(), 99.9, "30d".to_string());

        // All healthy — no budget consumed
        for _ in 0..100 {
            tracker.record_check("Budget Test", "healthy", 5);
        }

        let budget = tracker.error_budget("Budget Test").unwrap();
        assert_eq!(budget.target_percent, 99.9);
        assert!(budget.budget_minutes > 0.0);
        assert_eq!(budget.consumed_minutes, 0.0);
        assert!(!budget.exhausted);

        // Add some failures
        for _ in 0..5 {
            tracker.record_check("Budget Test", "degraded", 500);
        }

        let budget2 = tracker.error_budget("Budget Test").unwrap();
        assert!(budget2.consumed_minutes > 0.0);
        assert!(budget2.consumed_percent > 0.0);
    }

    #[test]
    fn test_burn_rate_no_alerts_when_healthy() {
        let tracker = SlaTracker {
            targets: DashMap::new(),
            uptime_records: DashMap::new(),
        };
        tracker.register_target("Healthy Svc".to_string(), 99.9, "30d".to_string());

        for _ in 0..100 {
            tracker.record_check("Healthy Svc", "healthy", 5);
        }

        let alerts = tracker.check_burn_rate("Healthy Svc");
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_sla_report_includes_budgets() {
        let tracker = SlaTracker::new();
        let report = tracker.get_sla_report();

        assert!(report.get("overall_uptime_percent").is_some());
        assert!(report.get("targets").is_some());
        assert!(report.get("burn_rate_alerts").is_some());

        let targets = report["targets"].as_array().unwrap();
        assert!(!targets.is_empty());

        // Each target should have an error_budget field
        for target in targets {
            assert!(target.get("error_budget").is_some());
        }
    }

    #[test]
    fn test_parse_window_days() {
        assert_eq!(parse_window_days("30d"), 30);
        assert_eq!(parse_window_days("7d"), 7);
        assert_eq!(parse_window_days("90d"), 90);
        assert_eq!(parse_window_days("invalid"), 30); // fallback
    }
}
