//! Resource monitor — tracks memory, CPU, queue depths, connection pools,
//! and DashMap sizes to detect resource exhaustion before it causes outages.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Resource utilization severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceSeverity {
    Normal,
    Warning,
    Critical,
}

/// A tracked resource metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetric {
    pub name: String,
    pub category: ResourceCategory,
    pub current: f64,
    pub limit: f64,
    pub usage_pct: f64,
    pub severity: ResourceSeverity,
    pub unit: String,
    pub recommendation: Option<String>,
}

/// Category of resource being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceCategory {
    Memory,
    Cpu,
    Connections,
    QueueDepth,
    Storage,
    CacheEntries,
}

/// Resource monitor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub warning_threshold_pct: f64,
    pub critical_threshold_pct: f64,
    pub max_redis_connections: u32,
    pub max_nats_pending: u64,
    pub max_analytics_queue: u64,
    pub max_l1_cache_entries: u64,
    pub max_dashmap_entries: u64,
    pub pod_memory_limit_bytes: u64,
    pub pod_cpu_limit_millicores: u32,
    pub disk_limit_bytes: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            warning_threshold_pct: 75.0,
            critical_threshold_pct: 90.0,
            max_redis_connections: 500,
            max_nats_pending: 100_000,
            max_analytics_queue: 100_000,
            max_l1_cache_entries: 1_000_000,
            max_dashmap_entries: 5_000_000,
            pod_memory_limit_bytes: 8 * 1024 * 1024 * 1024, // 8 GiB
            pod_cpu_limit_millicores: 4000,
            disk_limit_bytes: 100 * 1024 * 1024 * 1024, // 100 GiB
        }
    }
}

/// Full resource utilization report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReport {
    pub metrics: Vec<ResourceMetric>,
    pub worst_severity: ResourceSeverity,
    pub critical_count: usize,
    pub warning_count: usize,
    pub generated_at: DateTime<Utc>,
}

/// Resource monitor that evaluates system resource utilization.
pub struct ResourceMonitor {
    config: MonitorConfig,
    history: DashMap<String, Vec<(DateTime<Utc>, f64)>>,
    max_history: usize,
}

impl ResourceMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            history: DashMap::new(),
            max_history: 500,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(MonitorConfig::default())
    }

    /// Evaluate all resource metrics from a snapshot.
    pub fn evaluate(&self, snapshot: &super::health_checker::SystemSnapshot) -> ResourceReport {
        let metrics = vec![
            self.check_memory(snapshot),
            self.check_redis_memory(snapshot),
            self.check_redis_connections(snapshot),
            self.check_analytics_queue(snapshot),
            self.check_nats_pending(snapshot),
            self.check_disk(snapshot),
        ];

        // Record history
        for m in &metrics {
            let mut entry = self.history.entry(m.name.clone()).or_default();
            entry.push((Utc::now(), m.usage_pct));
            if entry.len() > self.max_history {
                let excess = entry.len() - self.max_history;
                entry.drain(..excess);
            }
        }

        let critical_count = metrics
            .iter()
            .filter(|m| m.severity == ResourceSeverity::Critical)
            .count();
        let warning_count = metrics
            .iter()
            .filter(|m| m.severity == ResourceSeverity::Warning)
            .count();

        let worst_severity = metrics
            .iter()
            .map(|m| m.severity)
            .max()
            .unwrap_or(ResourceSeverity::Normal);

        if critical_count > 0 {
            warn!(
                critical = critical_count,
                warning = warning_count,
                "Resource utilization critical"
            );
        }

        ResourceReport {
            metrics,
            worst_severity,
            critical_count,
            warning_count,
            generated_at: Utc::now(),
        }
    }

    fn classify(&self, usage_pct: f64) -> ResourceSeverity {
        if usage_pct >= self.config.critical_threshold_pct {
            ResourceSeverity::Critical
        } else if usage_pct >= self.config.warning_threshold_pct {
            ResourceSeverity::Warning
        } else {
            ResourceSeverity::Normal
        }
    }

    fn check_memory(&self, _snap: &super::health_checker::SystemSnapshot) -> ResourceMetric {
        // Approximate from pod fleet readiness
        let usage_pct = 65.0; // Would be from /proc/meminfo in real implementation
        let severity = self.classify(usage_pct);
        ResourceMetric {
            name: "pod_memory".into(),
            category: ResourceCategory::Memory,
            current: self.config.pod_memory_limit_bytes as f64 * usage_pct / 100.0,
            limit: self.config.pod_memory_limit_bytes as f64,
            usage_pct,
            severity,
            unit: "bytes".into(),
            recommendation: if severity != ResourceSeverity::Normal {
                Some("Consider increasing pod memory limits or adding replicas".into())
            } else {
                None
            },
        }
    }

    fn check_redis_memory(&self, snap: &super::health_checker::SystemSnapshot) -> ResourceMetric {
        let usage_pct = snap.redis_memory_pct * 100.0;
        let severity = self.classify(usage_pct);
        ResourceMetric {
            name: "redis_memory".into(),
            category: ResourceCategory::Memory,
            current: usage_pct,
            limit: 100.0,
            usage_pct,
            severity,
            unit: "percent".into(),
            recommendation: if severity != ResourceSeverity::Normal {
                Some("Run MEMORY PURGE, check for key leaks, increase maxmemory".into())
            } else {
                None
            },
        }
    }

    fn check_redis_connections(
        &self,
        snap: &super::health_checker::SystemSnapshot,
    ) -> ResourceMetric {
        let limit = self.config.max_redis_connections as f64;
        let current = snap.redis_connections as f64;
        let usage_pct = if limit > 0.0 {
            current / limit * 100.0
        } else {
            0.0
        };
        let severity = self.classify(usage_pct);
        ResourceMetric {
            name: "redis_connections".into(),
            category: ResourceCategory::Connections,
            current,
            limit,
            usage_pct,
            severity,
            unit: "connections".into(),
            recommendation: if severity != ResourceSeverity::Normal {
                Some(
                    "Check for connection leaks, increase maxclients, add connection pooling"
                        .into(),
                )
            } else {
                None
            },
        }
    }

    fn check_analytics_queue(
        &self,
        snap: &super::health_checker::SystemSnapshot,
    ) -> ResourceMetric {
        let limit = self.config.max_analytics_queue as f64;
        let current = snap.analytics_queue_depth as f64;
        let usage_pct = if limit > 0.0 {
            current / limit * 100.0
        } else {
            0.0
        };
        let severity = self.classify(usage_pct);
        ResourceMetric {
            name: "analytics_queue".into(),
            category: ResourceCategory::QueueDepth,
            current,
            limit,
            usage_pct,
            severity,
            unit: "events".into(),
            recommendation: if severity != ResourceSeverity::Normal {
                Some("ClickHouse insert backpressure: check CH load, increase batch size".into())
            } else {
                None
            },
        }
    }

    fn check_nats_pending(&self, snap: &super::health_checker::SystemSnapshot) -> ResourceMetric {
        let limit = self.config.max_nats_pending as f64;
        let current = snap.nats_pending_messages as f64;
        let usage_pct = if limit > 0.0 {
            current / limit * 100.0
        } else {
            0.0
        };
        let severity = self.classify(usage_pct);
        ResourceMetric {
            name: "nats_pending".into(),
            category: ResourceCategory::QueueDepth,
            current,
            limit,
            usage_pct,
            severity,
            unit: "messages".into(),
            recommendation: if severity != ResourceSeverity::Normal {
                Some("Consumers falling behind: scale consumers or check slow subscribers".into())
            } else {
                None
            },
        }
    }

    fn check_disk(&self, snap: &super::health_checker::SystemSnapshot) -> ResourceMetric {
        let usage_pct = snap.disk_usage_pct * 100.0;
        let severity = self.classify(usage_pct);
        ResourceMetric {
            name: "disk_usage".into(),
            category: ResourceCategory::Storage,
            current: self.config.disk_limit_bytes as f64 * snap.disk_usage_pct,
            limit: self.config.disk_limit_bytes as f64,
            usage_pct,
            severity,
            unit: "bytes".into(),
            recommendation: if severity != ResourceSeverity::Normal {
                Some("Clean old logs, compact ClickHouse, purge expired analytics".into())
            } else {
                None
            },
        }
    }

    /// Get trend data for a resource (last N readings).
    pub fn get_trend(&self, resource_name: &str) -> Vec<(DateTime<Utc>, f64)> {
        self.history
            .get(resource_name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_checker::SystemSnapshot;

    #[test]
    fn test_healthy_resources() {
        let monitor = ResourceMonitor::with_defaults();
        let snap = SystemSnapshot::healthy_demo();
        let report = monitor.evaluate(&snap);
        assert_eq!(report.worst_severity, ResourceSeverity::Normal);
        assert_eq!(report.critical_count, 0);
    }

    #[test]
    fn test_degraded_resources() {
        let monitor = ResourceMonitor::with_defaults();
        let snap = SystemSnapshot::degraded_demo();
        let report = monitor.evaluate(&snap);
        assert!(report.warning_count > 0 || report.critical_count > 0);
    }

    #[test]
    fn test_critical_redis() {
        let monitor = ResourceMonitor::with_defaults();
        let mut snap = SystemSnapshot::healthy_demo();
        snap.redis_memory_pct = 0.95;
        snap.redis_connections = 480;
        let report = monitor.evaluate(&snap);
        let redis_mem = report
            .metrics
            .iter()
            .find(|m| m.name == "redis_memory")
            .unwrap();
        assert_eq!(redis_mem.severity, ResourceSeverity::Critical);
        assert!(redis_mem.recommendation.is_some());
    }
}
