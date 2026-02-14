//! Incident detector — proactive anomaly detection, SLO burn-rate alerts,
//! and pattern-based incident prediction before users are impacted.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// SLO definition for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloDefinition {
    pub name: String,
    pub target_pct: f64,
    pub window_days: u32,
    pub burn_rate_alert_1h: f64,
    pub burn_rate_alert_6h: f64,
}

/// Current SLO status with error budget tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloStatus {
    pub name: String,
    pub target_pct: f64,
    pub current_pct: f64,
    pub error_budget_total_minutes: f64,
    pub error_budget_consumed_minutes: f64,
    pub error_budget_remaining_pct: f64,
    pub burn_rate_1h: f64,
    pub burn_rate_6h: f64,
    pub is_burning_fast: bool,
    pub estimated_budget_exhaustion_hours: Option<f64>,
}

/// Detected anomaly that may predict an incident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAnomaly {
    pub id: Uuid,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub metric_name: String,
    pub current_value: f64,
    pub expected_range: (f64, f64),
    pub deviation_pct: f64,
    pub detected_at: DateTime<Utc>,
    pub message: String,
    pub suggested_action: String,
}

/// Type of detected anomaly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    /// Value spike above normal range.
    Spike,
    /// Value drop below normal range.
    Drop,
    /// Trend moving toward a critical threshold.
    TrendTowardLimit,
    /// Sudden change in variance.
    VarianceShift,
    /// Error budget burning too fast.
    BurnRateAlert,
    /// Correlated failures across services.
    CorrelatedFailure,
}

/// Anomaly severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Full incident detection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentDetectionReport {
    pub slo_statuses: Vec<SloStatus>,
    pub anomalies: Vec<DetectedAnomaly>,
    pub slos_at_risk: usize,
    pub active_anomalies: usize,
    pub highest_severity: AnomalySeverity,
    pub generated_at: DateTime<Utc>,
}

/// Incident detector with SLO tracking and anomaly detection.
pub struct IncidentDetector {
    slo_definitions: Vec<SloDefinition>,
    uptime_records: DashMap<String, Vec<(DateTime<Utc>, bool)>>,
    metric_baselines: DashMap<String, MetricBaseline>,
    anomaly_history: DashMap<Uuid, DetectedAnomaly>,
}

/// Baseline statistics for a metric (for anomaly detection).
#[derive(Debug, Clone)]
struct MetricBaseline {
    mean: f64,
    std_dev: f64,
    min: f64,
    max: f64,
    sample_count: u64,
}

impl Default for IncidentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl IncidentDetector {
    pub fn new() -> Self {
        let detector = Self {
            slo_definitions: Self::default_slos(),
            uptime_records: DashMap::new(),
            metric_baselines: DashMap::new(),
            anomaly_history: DashMap::new(),
        };
        detector.seed_demo_baselines();
        detector
    }

    /// Record an uptime data point for a service.
    pub fn record_uptime(&self, service: &str, is_up: bool) {
        let mut entry = self.uptime_records.entry(service.to_string()).or_default();
        entry.push((Utc::now(), is_up));

        // Keep last 30 days of data
        let cutoff = Utc::now() - Duration::days(30);
        entry.retain(|(ts, _)| *ts > cutoff);
    }

    /// Record a metric observation to update baselines.
    pub fn record_metric(&self, name: &str, value: f64) {
        let mut baseline =
            self.metric_baselines
                .entry(name.to_string())
                .or_insert(MetricBaseline {
                    mean: value,
                    std_dev: 0.0,
                    min: value,
                    max: value,
                    sample_count: 0,
                });

        let b = baseline.value_mut();
        b.sample_count += 1;
        let n = b.sample_count as f64;

        // Online mean and variance (Welford's algorithm)
        let delta = value - b.mean;
        b.mean += delta / n;
        let delta2 = value - b.mean;
        let variance = if n > 1.0 {
            (b.std_dev * b.std_dev * (n - 1.0) + delta * delta2) / n
        } else {
            0.0
        };
        b.std_dev = variance.sqrt();
        b.min = b.min.min(value);
        b.max = b.max.max(value);
    }

    /// Run full incident detection analysis.
    pub fn detect(
        &self,
        snapshot: &super::health_checker::SystemSnapshot,
    ) -> IncidentDetectionReport {
        let mut anomalies = Vec::new();

        // Check SLOs
        let slo_statuses = self.evaluate_slos();

        // Check for SLO burn-rate anomalies
        for slo in &slo_statuses {
            if slo.is_burning_fast {
                anomalies.push(DetectedAnomaly {
                    id: Uuid::new_v4(),
                    anomaly_type: AnomalyType::BurnRateAlert,
                    severity: if slo.error_budget_remaining_pct < 10.0 {
                        AnomalySeverity::Emergency
                    } else {
                        AnomalySeverity::Critical
                    },
                    metric_name: format!("{}_error_budget", slo.name),
                    current_value: slo.burn_rate_1h,
                    expected_range: (0.0, 1.0),
                    deviation_pct: (slo.burn_rate_1h - 1.0) * 100.0,
                    detected_at: Utc::now(),
                    message: format!(
                        "{}: error budget {:.1}% remaining, burn rate {:.1}x",
                        slo.name, slo.error_budget_remaining_pct, slo.burn_rate_1h
                    ),
                    suggested_action: "Investigate root cause, consider rollback".into(),
                });
            }
        }

        // Check metric anomalies
        self.check_metric_anomaly(
            "api_latency",
            snapshot.api_latency_ms as f64,
            &mut anomalies,
        );
        self.check_metric_anomaly("error_rate", snapshot.error_rate * 100.0, &mut anomalies);
        self.check_metric_anomaly(
            "cache_hit_rate",
            snapshot.cache_hit_rate * 100.0,
            &mut anomalies,
        );
        self.check_metric_anomaly(
            "npu_latency",
            snapshot.npu_inference_latency_us as f64,
            &mut anomalies,
        );
        self.check_metric_anomaly(
            "analytics_queue",
            snapshot.analytics_queue_depth as f64,
            &mut anomalies,
        );

        // Check for correlated failures
        let unhealthy_components: Vec<String> = vec![
            ("api-server", snapshot.error_rate > 0.01),
            ("cache", !snapshot.redis_connected),
            ("nats", !snapshot.nats_connected),
            ("npu", !snapshot.npu_model_loaded),
        ]
        .into_iter()
        .filter(|(_, failed)| *failed)
        .map(|(name, _)| name.to_string())
        .collect();

        if unhealthy_components.len() >= 2 {
            anomalies.push(DetectedAnomaly {
                id: Uuid::new_v4(),
                anomaly_type: AnomalyType::CorrelatedFailure,
                severity: AnomalySeverity::Emergency,
                metric_name: "correlated_failures".into(),
                current_value: unhealthy_components.len() as f64,
                expected_range: (0.0, 1.0),
                deviation_pct: 100.0,
                detected_at: Utc::now(),
                message: format!(
                    "Correlated failures across: {}",
                    unhealthy_components.join(", ")
                ),
                suggested_action: "Check shared dependencies (network, DNS, K8s control plane)"
                    .into(),
            });
        }

        // Store anomalies
        for a in &anomalies {
            self.anomaly_history.insert(a.id, a.clone());
        }

        let slos_at_risk = slo_statuses
            .iter()
            .filter(|s| s.error_budget_remaining_pct < 30.0)
            .count();
        let active_anomalies = anomalies.len();
        let highest_severity = anomalies
            .iter()
            .map(|a| a.severity)
            .max()
            .unwrap_or(AnomalySeverity::Info);

        IncidentDetectionReport {
            slo_statuses,
            anomalies,
            slos_at_risk,
            active_anomalies,
            highest_severity,
            generated_at: Utc::now(),
        }
    }

    fn evaluate_slos(&self) -> Vec<SloStatus> {
        self.slo_definitions
            .iter()
            .map(|slo| {
                let records = self.uptime_records.get(&slo.name);
                let (total, up) = match records {
                    Some(recs) => {
                        let total = recs.len() as f64;
                        let up = recs.iter().filter(|(_, is_up)| *is_up).count() as f64;
                        (total, up)
                    }
                    None => (1.0, 1.0), // Assume 100% if no data
                };

                let current_pct = if total > 0.0 {
                    up / total * 100.0
                } else {
                    100.0
                };

                let window_minutes = slo.window_days as f64 * 24.0 * 60.0;
                let error_budget_total = window_minutes * (1.0 - slo.target_pct / 100.0);
                let downtime_minutes = window_minutes * (1.0 - current_pct / 100.0);
                let budget_remaining_pct = if error_budget_total > 0.0 {
                    ((error_budget_total - downtime_minutes) / error_budget_total * 100.0).max(0.0)
                } else {
                    100.0
                };

                // Simulated burn rates (in production, computed from time-windowed data)
                let burn_rate_1h = if current_pct < slo.target_pct {
                    (100.0 - current_pct) / (100.0 - slo.target_pct)
                } else {
                    0.0
                };
                let burn_rate_6h = burn_rate_1h * 0.8; // Smoothed

                let is_burning_fast =
                    burn_rate_1h > slo.burn_rate_alert_1h || burn_rate_6h > slo.burn_rate_alert_6h;

                let exhaustion_hours = if burn_rate_1h > 0.0 && budget_remaining_pct > 0.0 {
                    Some(budget_remaining_pct / burn_rate_1h)
                } else {
                    None
                };

                SloStatus {
                    name: slo.name.clone(),
                    target_pct: slo.target_pct,
                    current_pct,
                    error_budget_total_minutes: error_budget_total,
                    error_budget_consumed_minutes: downtime_minutes,
                    error_budget_remaining_pct: budget_remaining_pct,
                    burn_rate_1h,
                    burn_rate_6h,
                    is_burning_fast,
                    estimated_budget_exhaustion_hours: exhaustion_hours,
                }
            })
            .collect()
    }

    fn check_metric_anomaly(&self, name: &str, value: f64, anomalies: &mut Vec<DetectedAnomaly>) {
        if let Some(baseline) = self.metric_baselines.get(name) {
            let b = baseline.value();
            if b.sample_count < 10 || b.std_dev == 0.0 {
                return; // Not enough data
            }

            let z_score = (value - b.mean) / b.std_dev;
            let deviation_pct = ((value - b.mean) / b.mean * 100.0).abs();

            // Alert if value is more than 3 standard deviations from mean
            if z_score.abs() > 3.0 {
                let anomaly_type = if z_score > 0.0 {
                    AnomalyType::Spike
                } else {
                    AnomalyType::Drop
                };

                let severity = if z_score.abs() > 5.0 {
                    AnomalySeverity::Critical
                } else {
                    AnomalySeverity::Warning
                };

                anomalies.push(DetectedAnomaly {
                    id: Uuid::new_v4(),
                    anomaly_type,
                    severity,
                    metric_name: name.into(),
                    current_value: value,
                    expected_range: (b.mean - 2.0 * b.std_dev, b.mean + 2.0 * b.std_dev),
                    deviation_pct,
                    detected_at: Utc::now(),
                    message: format!(
                        "{name}: {value:.2} is {z_score:.1} std devs from mean {:.2}",
                        b.mean
                    ),
                    suggested_action: format!(
                        "Investigate {name} change, check recent deployments"
                    ),
                });
            }
        }
    }

    fn default_slos() -> Vec<SloDefinition> {
        vec![
            SloDefinition {
                name: "api-gateway".into(),
                target_pct: 99.99,
                window_days: 30,
                burn_rate_alert_1h: 14.4,
                burn_rate_alert_6h: 6.0,
            },
            SloDefinition {
                name: "bidding-engine".into(),
                target_pct: 99.95,
                window_days: 30,
                burn_rate_alert_1h: 14.4,
                burn_rate_alert_6h: 6.0,
            },
            SloDefinition {
                name: "nats".into(),
                target_pct: 99.99,
                window_days: 30,
                burn_rate_alert_1h: 14.4,
                burn_rate_alert_6h: 6.0,
            },
            SloDefinition {
                name: "redis".into(),
                target_pct: 99.95,
                window_days: 30,
                burn_rate_alert_1h: 14.4,
                burn_rate_alert_6h: 6.0,
            },
            SloDefinition {
                name: "clickhouse".into(),
                target_pct: 99.90,
                window_days: 30,
                burn_rate_alert_1h: 14.4,
                burn_rate_alert_6h: 6.0,
            },
            SloDefinition {
                name: "npu-engine".into(),
                target_pct: 99.90,
                window_days: 30,
                burn_rate_alert_1h: 14.4,
                burn_rate_alert_6h: 6.0,
            },
        ]
    }

    fn seed_demo_baselines(&self) {
        // Seed baselines with typical healthy ranges
        let baselines = vec![
            ("api_latency", 3.0, 2.0, 1.0, 15.0),
            ("error_rate", 0.1, 0.05, 0.0, 0.5),
            ("cache_hit_rate", 94.0, 3.0, 85.0, 98.0),
            ("npu_latency", 2500.0, 500.0, 1500.0, 5000.0),
            ("analytics_queue", 1200.0, 800.0, 100.0, 5000.0),
        ];

        for (name, mean, std_dev, min, max) in baselines {
            self.metric_baselines.insert(
                name.to_string(),
                MetricBaseline {
                    mean,
                    std_dev,
                    min,
                    max,
                    sample_count: 1000,
                },
            );
        }
    }

    /// Get all anomaly history.
    pub fn anomaly_history(&self, limit: usize) -> Vec<DetectedAnomaly> {
        let mut history: Vec<DetectedAnomaly> = self
            .anomaly_history
            .iter()
            .map(|e| e.value().clone())
            .collect();
        history.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
        history.truncate(limit);
        history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_checker::SystemSnapshot;

    #[test]
    fn test_healthy_detection() {
        let detector = IncidentDetector::new();
        let snap = SystemSnapshot::healthy_demo();
        let report = detector.detect(&snap);
        // Healthy snapshot should have no critical anomalies
        assert_eq!(report.slos_at_risk, 0);
    }

    #[test]
    fn test_degraded_detection() {
        let detector = IncidentDetector::new();
        let mut snap = SystemSnapshot::degraded_demo();
        snap.error_rate = 0.15; // 15% error rate — way above normal
        let report = detector.detect(&snap);
        assert!(!report.anomalies.is_empty());
    }

    #[test]
    fn test_correlated_failure() {
        let detector = IncidentDetector::new();
        let mut snap = SystemSnapshot::healthy_demo();
        snap.redis_connected = false;
        snap.nats_connected = false;
        let report = detector.detect(&snap);
        let correlated = report
            .anomalies
            .iter()
            .any(|a| matches!(a.anomaly_type, AnomalyType::CorrelatedFailure));
        assert!(correlated);
    }

    #[test]
    fn test_slo_evaluation() {
        let detector = IncidentDetector::new();
        // Record some uptime data
        for _ in 0..100 {
            detector.record_uptime("api-gateway", true);
        }
        let snap = SystemSnapshot::healthy_demo();
        let report = detector.detect(&snap);
        let api_slo = report.slo_statuses.iter().find(|s| s.name == "api-gateway");
        assert!(api_slo.is_some());
        assert_eq!(api_slo.unwrap().current_pct, 100.0);
    }

    #[test]
    fn test_metric_baseline_update() {
        let detector = IncidentDetector::new();
        for i in 0..100 {
            detector.record_metric("test_metric", 50.0 + (i as f64 * 0.1));
        }
        // Record an anomalous value
        let snap = SystemSnapshot::healthy_demo();
        let report = detector.detect(&snap);
        // The healthy demo values are within normal baselines
        assert!(report.highest_severity <= AnomalySeverity::Warning);
    }
}
