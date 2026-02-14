//! Deep health checker — probes every service dependency and reports
//! readiness with latency measurements and degradation detection.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Health status of a single component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// A single health probe result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub component: String,
    pub status: ComponentHealth,
    pub latency_ms: u64,
    pub message: String,
    pub checked_at: DateTime<Utc>,
}

/// Aggregated cluster health report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealthReport {
    pub overall: ComponentHealth,
    pub probes: Vec<ProbeResult>,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub total_probes: usize,
    pub generated_at: DateTime<Utc>,
    pub cluster_uptime_pct: f64,
}

/// Threshold configuration for degradation detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    /// API latency above this (ms) = degraded.
    pub api_latency_degraded_ms: u64,
    /// API latency above this (ms) = unhealthy.
    pub api_latency_unhealthy_ms: u64,
    /// Cache hit rate below this = degraded.
    pub cache_hit_rate_degraded: f64,
    /// Error rate above this = degraded.
    pub error_rate_degraded: f64,
    /// Error rate above this = unhealthy.
    pub error_rate_unhealthy: f64,
    /// Queue depth above this = degraded.
    pub queue_depth_degraded: u64,
    /// Memory usage % above this = degraded.
    pub memory_pct_degraded: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            api_latency_degraded_ms: 20,
            api_latency_unhealthy_ms: 50,
            cache_hit_rate_degraded: 0.80,
            error_rate_degraded: 0.005,
            error_rate_unhealthy: 0.01,
            queue_depth_degraded: 50_000,
            memory_pct_degraded: 0.85,
        }
    }
}

/// Deep health checker for the entire Campaign Express stack.
pub struct HealthChecker {
    thresholds: HealthThresholds,
    history: DashMap<String, Vec<ProbeResult>>,
    max_history: usize,
}

impl HealthChecker {
    pub fn new(thresholds: HealthThresholds) -> Self {
        Self {
            thresholds,
            history: DashMap::new(),
            max_history: 1000,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(HealthThresholds::default())
    }

    /// Run a full cluster health check across all components.
    pub fn run_full_check(&self, snapshot: &SystemSnapshot) -> ClusterHealthReport {
        let probes = vec![
            self.check_api(snapshot),
            self.check_cache(snapshot),
            self.check_analytics(snapshot),
            self.check_npu(snapshot),
            self.check_message_bus(snapshot),
            self.check_pod_fleet(snapshot),
            self.check_storage(snapshot),
            self.check_tenant_quotas(snapshot),
        ];

        // Record history
        for probe in &probes {
            let mut entry = self.history.entry(probe.component.clone()).or_default();
            entry.push(probe.clone());
            if entry.len() > self.max_history {
                let excess = entry.len() - self.max_history;
                entry.drain(..excess);
            }
        }

        let healthy_count = probes
            .iter()
            .filter(|p| p.status == ComponentHealth::Healthy)
            .count();
        let degraded_count = probes
            .iter()
            .filter(|p| p.status == ComponentHealth::Degraded)
            .count();
        let unhealthy_count = probes
            .iter()
            .filter(|p| p.status == ComponentHealth::Unhealthy)
            .count();
        let total_probes = probes.len();

        let overall = if unhealthy_count > 0 {
            ComponentHealth::Unhealthy
        } else if degraded_count > 0 {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        let cluster_uptime_pct = self.calculate_uptime_pct();

        if overall != ComponentHealth::Healthy {
            warn!(
                overall = ?overall,
                unhealthy = unhealthy_count,
                degraded = degraded_count,
                "Cluster health check completed with issues"
            );
        } else {
            info!("Cluster health check: all {} probes healthy", total_probes);
        }

        ClusterHealthReport {
            overall,
            probes,
            healthy_count,
            degraded_count,
            unhealthy_count,
            total_probes,
            generated_at: Utc::now(),
            cluster_uptime_pct,
        }
    }

    fn check_api(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if snap.api_latency_ms > self.thresholds.api_latency_unhealthy_ms {
            ComponentHealth::Unhealthy
        } else if snap.api_latency_ms > self.thresholds.api_latency_degraded_ms {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        let message = if snap.error_rate > self.thresholds.error_rate_unhealthy {
            format!(
                "Error rate {:.3}% exceeds threshold",
                snap.error_rate * 100.0
            )
        } else {
            format!(
                "Latency {}ms, error rate {:.4}%",
                snap.api_latency_ms,
                snap.error_rate * 100.0
            )
        };

        // Override status if error rate is critical
        let status = if snap.error_rate > self.thresholds.error_rate_unhealthy {
            ComponentHealth::Unhealthy
        } else if snap.error_rate > self.thresholds.error_rate_degraded {
            ComponentHealth::Degraded
        } else {
            status
        };

        ProbeResult {
            component: "api-server".into(),
            status,
            latency_ms: snap.api_latency_ms,
            message,
            checked_at: Utc::now(),
        }
    }

    fn check_cache(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if !snap.redis_connected {
            ComponentHealth::Unhealthy
        } else if snap.cache_hit_rate < self.thresholds.cache_hit_rate_degraded {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "cache".into(),
            status,
            latency_ms: snap.redis_latency_ms,
            message: format!(
                "Hit rate {:.1}%, Redis {}",
                snap.cache_hit_rate * 100.0,
                if snap.redis_connected {
                    "connected"
                } else {
                    "DISCONNECTED"
                }
            ),
            checked_at: Utc::now(),
        }
    }

    fn check_analytics(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if !snap.clickhouse_connected {
            ComponentHealth::Unhealthy
        } else if snap.analytics_queue_depth > self.thresholds.queue_depth_degraded {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "analytics-pipeline".into(),
            status,
            latency_ms: snap.clickhouse_latency_ms,
            message: format!(
                "Queue depth {}, dropped {}",
                snap.analytics_queue_depth, snap.analytics_dropped
            ),
            checked_at: Utc::now(),
        }
    }

    fn check_npu(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if snap.npu_inference_latency_us > 10_000 {
            ComponentHealth::Unhealthy
        } else if snap.npu_inference_latency_us > 5_000 {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "npu-engine".into(),
            status,
            latency_ms: snap.npu_inference_latency_us / 1000,
            message: format!(
                "Inference {}us, model {}",
                snap.npu_inference_latency_us,
                if snap.npu_model_loaded {
                    "loaded"
                } else {
                    "NOT LOADED"
                }
            ),
            checked_at: Utc::now(),
        }
    }

    fn check_message_bus(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if !snap.nats_connected {
            ComponentHealth::Unhealthy
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "nats".into(),
            status,
            latency_ms: snap.nats_latency_ms,
            message: format!(
                "NATS {} ({} pending messages)",
                if snap.nats_connected {
                    "connected"
                } else {
                    "DISCONNECTED"
                },
                snap.nats_pending_messages,
            ),
            checked_at: Utc::now(),
        }
    }

    fn check_pod_fleet(&self, snap: &SystemSnapshot) -> ProbeResult {
        let ready_pct = if snap.total_pods > 0 {
            snap.ready_pods as f64 / snap.total_pods as f64
        } else {
            0.0
        };

        let status = if ready_pct < 0.5 {
            ComponentHealth::Unhealthy
        } else if ready_pct < 0.8 {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "pod-fleet".into(),
            status,
            latency_ms: 0,
            message: format!(
                "{}/{} pods ready ({:.0}%), {} restarting",
                snap.ready_pods,
                snap.total_pods,
                ready_pct * 100.0,
                snap.restarting_pods,
            ),
            checked_at: Utc::now(),
        }
    }

    fn check_storage(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if snap.disk_usage_pct > 0.95 {
            ComponentHealth::Unhealthy
        } else if snap.disk_usage_pct > self.thresholds.memory_pct_degraded {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "storage".into(),
            status,
            latency_ms: 0,
            message: format!("Disk usage {:.1}%", snap.disk_usage_pct * 100.0),
            checked_at: Utc::now(),
        }
    }

    fn check_tenant_quotas(&self, snap: &SystemSnapshot) -> ProbeResult {
        let status = if snap.tenants_over_quota > 0 {
            ComponentHealth::Degraded
        } else {
            ComponentHealth::Healthy
        };

        ProbeResult {
            component: "tenant-quotas".into(),
            status,
            latency_ms: 0,
            message: format!(
                "{} tenants near quota, {} over quota",
                snap.tenants_near_quota, snap.tenants_over_quota,
            ),
            checked_at: Utc::now(),
        }
    }

    fn calculate_uptime_pct(&self) -> f64 {
        let api_history = self.history.get("api-server");
        match api_history {
            Some(records) if !records.is_empty() => {
                let healthy = records
                    .iter()
                    .filter(|r| r.status == ComponentHealth::Healthy)
                    .count();
                healthy as f64 / records.len() as f64 * 100.0
            }
            _ => 100.0,
        }
    }

    /// Get probe history for a specific component.
    pub fn get_history(&self, component: &str) -> Vec<ProbeResult> {
        self.history
            .get(component)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get the last N probe results across all components.
    pub fn recent_probes(&self, limit: usize) -> Vec<ProbeResult> {
        let mut all: Vec<ProbeResult> = self
            .history
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect();
        all.sort_by(|a, b| b.checked_at.cmp(&a.checked_at));
        all.truncate(limit);
        all
    }
}

/// Point-in-time snapshot of system metrics used by the health checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    // API
    pub api_latency_ms: u64,
    pub error_rate: f64,
    pub requests_per_sec: f64,
    // Cache
    pub cache_hit_rate: f64,
    pub redis_connected: bool,
    pub redis_latency_ms: u64,
    pub redis_memory_pct: f64,
    pub redis_connections: u32,
    // Analytics
    pub clickhouse_connected: bool,
    pub clickhouse_latency_ms: u64,
    pub analytics_queue_depth: u64,
    pub analytics_dropped: u64,
    // NPU
    pub npu_inference_latency_us: u64,
    pub npu_model_loaded: bool,
    // NATS
    pub nats_connected: bool,
    pub nats_latency_ms: u64,
    pub nats_pending_messages: u64,
    // Pods
    pub total_pods: u32,
    pub ready_pods: u32,
    pub restarting_pods: u32,
    // Storage
    pub disk_usage_pct: f64,
    // Tenants
    pub tenants_near_quota: u32,
    pub tenants_over_quota: u32,
    // Timestamp
    pub captured_at: DateTime<Utc>,
}

impl SystemSnapshot {
    /// Create a healthy snapshot (for testing / demo).
    pub fn healthy_demo() -> Self {
        Self {
            api_latency_ms: 3,
            error_rate: 0.001,
            requests_per_sec: 14_000.0,
            cache_hit_rate: 0.94,
            redis_connected: true,
            redis_latency_ms: 1,
            redis_memory_pct: 0.62,
            redis_connections: 180,
            clickhouse_connected: true,
            clickhouse_latency_ms: 5,
            analytics_queue_depth: 1200,
            analytics_dropped: 0,
            npu_inference_latency_us: 2500,
            npu_model_loaded: true,
            nats_connected: true,
            nats_latency_ms: 1,
            nats_pending_messages: 45,
            total_pods: 20,
            ready_pods: 20,
            restarting_pods: 0,
            disk_usage_pct: 0.45,
            tenants_near_quota: 2,
            tenants_over_quota: 0,
            captured_at: Utc::now(),
        }
    }

    /// Create a degraded snapshot (for testing).
    pub fn degraded_demo() -> Self {
        Self {
            api_latency_ms: 35,
            error_rate: 0.008,
            requests_per_sec: 8_000.0,
            cache_hit_rate: 0.72,
            redis_connected: true,
            redis_latency_ms: 15,
            redis_memory_pct: 0.88,
            redis_connections: 450,
            clickhouse_connected: true,
            clickhouse_latency_ms: 50,
            analytics_queue_depth: 75_000,
            analytics_dropped: 340,
            npu_inference_latency_us: 6_500,
            npu_model_loaded: true,
            nats_connected: true,
            nats_latency_ms: 8,
            nats_pending_messages: 12_000,
            total_pods: 20,
            ready_pods: 16,
            restarting_pods: 2,
            disk_usage_pct: 0.87,
            tenants_near_quota: 5,
            tenants_over_quota: 1,
            captured_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_cluster() {
        let checker = HealthChecker::with_defaults();
        let snap = SystemSnapshot::healthy_demo();
        let report = checker.run_full_check(&snap);
        assert_eq!(report.overall, ComponentHealth::Healthy);
        assert_eq!(report.unhealthy_count, 0);
        assert_eq!(report.degraded_count, 0);
        assert_eq!(report.total_probes, 8);
    }

    #[test]
    fn test_degraded_cluster() {
        let checker = HealthChecker::with_defaults();
        let snap = SystemSnapshot::degraded_demo();
        let report = checker.run_full_check(&snap);
        assert_ne!(report.overall, ComponentHealth::Healthy);
        assert!(report.degraded_count > 0 || report.unhealthy_count > 0);
    }

    #[test]
    fn test_unhealthy_redis() {
        let checker = HealthChecker::with_defaults();
        let mut snap = SystemSnapshot::healthy_demo();
        snap.redis_connected = false;
        let report = checker.run_full_check(&snap);
        let cache_probe = report
            .probes
            .iter()
            .find(|p| p.component == "cache")
            .unwrap();
        assert_eq!(cache_probe.status, ComponentHealth::Unhealthy);
    }

    #[test]
    fn test_probe_history() {
        let checker = HealthChecker::with_defaults();
        let snap = SystemSnapshot::healthy_demo();
        checker.run_full_check(&snap);
        checker.run_full_check(&snap);
        let history = checker.get_history("api-server");
        assert_eq!(history.len(), 2);
    }
}
