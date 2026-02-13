//! SLA tracking and uptime monitoring for Campaign Express services.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

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

        // Update current percent based on recent checks
        if let Some(mut target) = self.targets.get_mut(service) {
            if status != "healthy" {
                target.current_percent = (target.current_percent - 0.01).max(0.0);
                target.last_incident = Some(Utc::now());
            }
        }

        record
    }

    pub fn get_sla_report(&self) -> serde_json::Value {
        let targets: Vec<serde_json::Value> = self
            .targets
            .iter()
            .map(|r| {
                let t = r.value();
                let meeting_sla = t.current_percent >= t.target_percent;
                serde_json::json!({
                    "name": t.name,
                    "target_percent": t.target_percent,
                    "current_percent": t.current_percent,
                    "measurement_window": t.measurement_window,
                    "meeting_sla": meeting_sla,
                    "last_incident": t.last_incident,
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

        serde_json::json!({
            "generated_at": Utc::now(),
            "overall_uptime_percent": overall_uptime,
            "total_checks": total_checks,
            "targets": targets,
        })
    }

    fn seed_demo_data(&self) {
        let now = Utc::now();

        let services = vec![
            ("API Gateway", 99.99, "30d", 99.997),
            ("Bidding Engine", 99.95, "30d", 99.98),
            ("NATS Cluster", 99.99, "30d", 99.999),
            ("Redis Cluster", 99.95, "30d", 99.96),
            ("ClickHouse", 99.90, "30d", 99.92),
            ("NPU Engine", 99.90, "30d", 99.95),
        ];

        for (name, target, window, current) in services {
            let sla_target = SlaTarget {
                name: name.to_string(),
                target_percent: target,
                current_percent: current,
                measurement_window: window.to_string(),
                last_incident: if current < 99.99 {
                    Some(now - Duration::days(3))
                } else {
                    None
                },
            };
            self.targets.insert(name.to_string(), sla_target);
        }

        // Seed some uptime records
        for i in 0..24 {
            let check_time = now - Duration::hours(i);
            for service in &[
                "API Gateway",
                "Bidding Engine",
                "NATS Cluster",
                "Redis Cluster",
                "ClickHouse",
                "NPU Engine",
            ] {
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
    }
}

impl Default for SlaTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_tracking() {
        let tracker = SlaTracker::new();
        let target = tracker.register_target("Test Service".to_string(), 99.9, "30d".to_string());
        assert_eq!(target.current_percent, 100.0);
        assert!(target.last_incident.is_none());

        // Record a healthy check
        let check = tracker.record_check("Test Service", "healthy", 5);
        assert_eq!(check.status, "healthy");

        // Record an unhealthy check
        tracker.record_check("Test Service", "degraded", 500);

        let report = tracker.get_sla_report();
        assert!(report.get("overall_uptime_percent").is_some());
        assert!(report.get("targets").is_some());
    }
}
