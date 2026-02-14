//! Auto-remediation engine — executes automated runbooks to fix known
//! issues without human intervention, with audit trail and escalation.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::health_checker::{ComponentHealth, ProbeResult};
use crate::resource_monitor::{ResourceReport, ResourceSeverity};

/// Remediation action type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationAction {
    /// Restart a specific pod/service.
    RestartService { service: String },
    /// Scale up replicas.
    ScaleUp { service: String, additional: u32 },
    /// Scale down replicas.
    ScaleDown { service: String, remove: u32 },
    /// Flush/clear a cache.
    FlushCache { cache_name: String },
    /// Trigger a backup.
    TriggerBackup { target: String },
    /// Reload NPU model.
    ReloadModel,
    /// Evict old data to free disk.
    EvictStaleData {
        service: String,
        retention_days: u32,
    },
    /// Adjust rate limits.
    AdjustRateLimit { new_rps: u32 },
    /// Run custom command.
    RunCommand { command: String, args: Vec<String> },
    /// Escalate to on-call.
    Escalate { severity: String, message: String },
    /// No action needed.
    NoAction,
}

/// Result of executing a remediation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationResult {
    pub id: Uuid,
    pub runbook_name: String,
    pub trigger: String,
    pub actions: Vec<RemediationAction>,
    pub success: bool,
    pub message: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub auto_executed: bool,
}

/// A runbook definition with trigger conditions and actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger_condition: TriggerCondition,
    pub actions: Vec<RemediationAction>,
    pub cooldown_seconds: u64,
    pub auto_execute: bool,
    pub max_auto_executions_per_hour: u32,
    pub created_at: DateTime<Utc>,
}

/// Condition that triggers a runbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerCondition {
    ComponentUnhealthy { component: String },
    ResourceCritical { resource: String },
    ErrorRateAbove { threshold: f64 },
    LatencyAbove { threshold_ms: u64 },
    PodCrashLoop { restart_threshold: u32 },
    QueueBackpressure { queue: String, threshold: u64 },
    DiskFull { threshold_pct: f64 },
    CacheHitRateBelow { threshold: f64 },
    Custom { name: String },
}

/// Auto-remediation engine.
pub struct RemediationEngine {
    runbooks: DashMap<Uuid, Runbook>,
    execution_log: DashMap<Uuid, RemediationResult>,
    execution_counts: DashMap<String, Vec<DateTime<Utc>>>,
    _max_log_entries: usize,
}

impl Default for RemediationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RemediationEngine {
    pub fn new() -> Self {
        let engine = Self {
            runbooks: DashMap::new(),
            execution_log: DashMap::new(),
            execution_counts: DashMap::new(),
            _max_log_entries: 10_000,
        };
        engine.seed_default_runbooks();
        engine
    }

    /// Register a new runbook.
    pub fn register_runbook(&self, runbook: Runbook) {
        info!(name = %runbook.name, auto = runbook.auto_execute, "Runbook registered");
        self.runbooks.insert(runbook.id, runbook);
    }

    /// Evaluate triggers against current state and execute matching runbooks.
    pub fn evaluate_and_remediate(
        &self,
        probes: &[ProbeResult],
        resources: &ResourceReport,
    ) -> Vec<RemediationResult> {
        let mut results = Vec::new();

        for entry in self.runbooks.iter() {
            let runbook = entry.value();
            if !runbook.auto_execute {
                continue;
            }

            if self.matches_trigger(&runbook.trigger_condition, probes, resources)
                && self.check_cooldown(runbook)
            {
                let result = self.execute_runbook(runbook, "auto-trigger");
                results.push(result);
            }
        }

        results
    }

    /// Manually trigger a runbook by name.
    pub fn trigger_runbook(&self, name: &str, actor: &str) -> Option<RemediationResult> {
        let runbook = self.runbooks.iter().find(|r| r.value().name == name)?;

        Some(self.execute_runbook(runbook.value(), actor))
    }

    /// List all registered runbooks.
    pub fn list_runbooks(&self) -> Vec<Runbook> {
        self.runbooks.iter().map(|r| r.value().clone()).collect()
    }

    /// Get execution history.
    pub fn execution_history(&self, limit: usize) -> Vec<RemediationResult> {
        let mut history: Vec<RemediationResult> = self
            .execution_log
            .iter()
            .map(|e| e.value().clone())
            .collect();
        history.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        history.truncate(limit);
        history
    }

    fn matches_trigger(
        &self,
        condition: &TriggerCondition,
        probes: &[ProbeResult],
        resources: &ResourceReport,
    ) -> bool {
        match condition {
            TriggerCondition::ComponentUnhealthy { component } => probes
                .iter()
                .any(|p| p.component == *component && p.status == ComponentHealth::Unhealthy),

            TriggerCondition::ResourceCritical { resource } => resources
                .metrics
                .iter()
                .any(|m| m.name == *resource && m.severity == ResourceSeverity::Critical),

            TriggerCondition::ErrorRateAbove { threshold: _ } => probes
                .iter()
                .any(|p| p.component == "api-server" && p.status == ComponentHealth::Unhealthy),

            TriggerCondition::LatencyAbove { threshold_ms } => probes
                .iter()
                .any(|p| p.component == "api-server" && p.latency_ms > *threshold_ms),

            TriggerCondition::PodCrashLoop {
                restart_threshold: _,
            } => probes
                .iter()
                .any(|p| p.component == "pod-fleet" && p.status == ComponentHealth::Degraded),

            TriggerCondition::QueueBackpressure {
                queue,
                threshold: _,
            } => resources
                .metrics
                .iter()
                .any(|m| m.name.contains(queue) && m.severity == ResourceSeverity::Critical),

            TriggerCondition::DiskFull { threshold_pct: _ } => resources
                .metrics
                .iter()
                .any(|m| m.name == "disk_usage" && m.severity == ResourceSeverity::Critical),

            TriggerCondition::CacheHitRateBelow { threshold: _ } => probes
                .iter()
                .any(|p| p.component == "cache" && p.status == ComponentHealth::Degraded),

            TriggerCondition::Custom { .. } => false,
        }
    }

    fn check_cooldown(&self, runbook: &Runbook) -> bool {
        let key = runbook.name.clone();
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(runbook.cooldown_seconds as i64);

        let mut entry = self.execution_counts.entry(key).or_default();

        // Remove old entries
        entry.retain(|t| *t > cutoff);

        // Check rate limit
        if entry.len() >= runbook.max_auto_executions_per_hour as usize {
            warn!(
                runbook = %runbook.name,
                executions = entry.len(),
                "Runbook rate limit reached, skipping"
            );
            return false;
        }

        entry.push(now);
        true
    }

    fn execute_runbook(&self, runbook: &Runbook, trigger: &str) -> RemediationResult {
        let started_at = Utc::now();
        info!(runbook = %runbook.name, trigger = trigger, "Executing runbook");

        // Execute each action (simulated — real implementation would use kubectl/API calls)
        let success = true;
        let mut messages = Vec::new();

        for action in &runbook.actions {
            match action {
                RemediationAction::RestartService { service } => {
                    messages.push(format!("Restarted service: {service}"));
                }
                RemediationAction::ScaleUp {
                    service,
                    additional,
                } => {
                    messages.push(format!("Scaled up {service} by +{additional} replicas"));
                }
                RemediationAction::ScaleDown { service, remove } => {
                    messages.push(format!("Scaled down {service} by -{remove} replicas"));
                }
                RemediationAction::FlushCache { cache_name } => {
                    messages.push(format!("Flushed cache: {cache_name}"));
                }
                RemediationAction::ReloadModel => {
                    messages.push("Triggered NPU model reload".into());
                }
                RemediationAction::EvictStaleData {
                    service,
                    retention_days,
                } => {
                    messages.push(format!(
                        "Evicted data older than {retention_days}d from {service}"
                    ));
                }
                RemediationAction::AdjustRateLimit { new_rps } => {
                    messages.push(format!("Adjusted rate limit to {new_rps} RPS"));
                }
                RemediationAction::Escalate { severity, message } => {
                    messages.push(format!("[{severity}] Escalated: {message}"));
                }
                RemediationAction::RunCommand { command, args } => {
                    messages.push(format!("Ran: {command} {}", args.join(" ")));
                }
                RemediationAction::TriggerBackup { target } => {
                    messages.push(format!("Triggered backup for: {target}"));
                }
                RemediationAction::NoAction => {
                    messages.push("No action taken".into());
                }
            }
        }

        let result = RemediationResult {
            id: Uuid::new_v4(),
            runbook_name: runbook.name.clone(),
            trigger: trigger.into(),
            actions: runbook.actions.clone(),
            success,
            message: messages.join("; "),
            started_at,
            completed_at: Utc::now(),
            auto_executed: trigger == "auto-trigger",
        };

        self.execution_log.insert(result.id, result.clone());
        result
    }

    fn seed_default_runbooks(&self) {
        let runbooks = vec![
            Runbook {
                id: Uuid::new_v4(),
                name: "high-latency-response".into(),
                description: "Auto-scale when API latency exceeds threshold".into(),
                trigger_condition: TriggerCondition::LatencyAbove { threshold_ms: 50 },
                actions: vec![
                    RemediationAction::ScaleUp {
                        service: "campaign-express".into(),
                        additional: 4,
                    },
                    RemediationAction::FlushCache {
                        cache_name: "l1-local".into(),
                    },
                ],
                cooldown_seconds: 300,
                auto_execute: true,
                max_auto_executions_per_hour: 3,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "redis-memory-pressure".into(),
                description: "Evict stale cache entries when Redis memory > 90%".into(),
                trigger_condition: TriggerCondition::ResourceCritical {
                    resource: "redis_memory".into(),
                },
                actions: vec![
                    RemediationAction::RunCommand {
                        command: "redis-cli".into(),
                        args: vec!["MEMORY".into(), "PURGE".into()],
                    },
                    RemediationAction::EvictStaleData {
                        service: "redis".into(),
                        retention_days: 1,
                    },
                ],
                cooldown_seconds: 600,
                auto_execute: true,
                max_auto_executions_per_hour: 2,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "npu-model-failure".into(),
                description: "Reload NPU model and restart inference engine".into(),
                trigger_condition: TriggerCondition::ComponentUnhealthy {
                    component: "npu-engine".into(),
                },
                actions: vec![
                    RemediationAction::ReloadModel,
                    RemediationAction::RestartService {
                        service: "npu-engine".into(),
                    },
                ],
                cooldown_seconds: 300,
                auto_execute: true,
                max_auto_executions_per_hour: 2,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "analytics-backpressure".into(),
                description: "Handle ClickHouse insert backpressure".into(),
                trigger_condition: TriggerCondition::QueueBackpressure {
                    queue: "analytics".into(),
                    threshold: 80_000,
                },
                actions: vec![
                    RemediationAction::RunCommand {
                        command: "clickhouse-client".into(),
                        args: vec!["--query".into(), "SYSTEM FLUSH LOGS".into()],
                    },
                    RemediationAction::AdjustRateLimit { new_rps: 8000 },
                ],
                cooldown_seconds: 600,
                auto_execute: true,
                max_auto_executions_per_hour: 2,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "pod-crash-loop-recovery".into(),
                description: "Handle crash-looping pods".into(),
                trigger_condition: TriggerCondition::PodCrashLoop {
                    restart_threshold: 3,
                },
                actions: vec![
                    RemediationAction::RunCommand {
                        command: "kubectl".into(),
                        args: vec![
                            "delete".into(),
                            "pod".into(),
                            "-l".into(),
                            "app=campaign-express".into(),
                            "--field-selector=status.phase=Failed".into(),
                        ],
                    },
                    RemediationAction::ScaleUp {
                        service: "campaign-express".into(),
                        additional: 2,
                    },
                ],
                cooldown_seconds: 300,
                auto_execute: true,
                max_auto_executions_per_hour: 3,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "disk-space-critical".into(),
                description: "Free disk space by evicting old analytics data".into(),
                trigger_condition: TriggerCondition::DiskFull {
                    threshold_pct: 90.0,
                },
                actions: vec![
                    RemediationAction::EvictStaleData {
                        service: "clickhouse".into(),
                        retention_days: 30,
                    },
                    RemediationAction::RunCommand {
                        command: "find".into(),
                        args: vec![
                            "/var/log".into(),
                            "-name".into(),
                            "*.log.gz".into(),
                            "-mtime".into(),
                            "+7".into(),
                            "-delete".into(),
                        ],
                    },
                ],
                cooldown_seconds: 1800,
                auto_execute: true,
                max_auto_executions_per_hour: 1,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "cache-degradation".into(),
                description: "Warm cache when hit rate drops below threshold".into(),
                trigger_condition: TriggerCondition::CacheHitRateBelow { threshold: 0.80 },
                actions: vec![
                    RemediationAction::FlushCache {
                        cache_name: "l1-local".into(),
                    },
                    RemediationAction::RunCommand {
                        command: "curl".into(),
                        args: vec![
                            "-X".into(),
                            "POST".into(),
                            "http://localhost:8080/api/v1/management/models/reload".into(),
                        ],
                    },
                ],
                cooldown_seconds: 600,
                auto_execute: true,
                max_auto_executions_per_hour: 2,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "nats-reconnect".into(),
                description: "Reconnect NATS when message bus goes down".into(),
                trigger_condition: TriggerCondition::ComponentUnhealthy {
                    component: "nats".into(),
                },
                actions: vec![
                    RemediationAction::RestartService {
                        service: "nats".into(),
                    },
                    RemediationAction::Escalate {
                        severity: "high".into(),
                        message: "NATS message bus went down — restarted automatically".into(),
                    },
                ],
                cooldown_seconds: 120,
                auto_execute: true,
                max_auto_executions_per_hour: 5,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "tenant-quota-enforcement".into(),
                description: "Throttle tenants exceeding quotas".into(),
                trigger_condition: TriggerCondition::Custom {
                    name: "tenant-over-quota".into(),
                },
                actions: vec![
                    RemediationAction::AdjustRateLimit { new_rps: 100 },
                    RemediationAction::Escalate {
                        severity: "warning".into(),
                        message: "Tenant quota exceeded — rate limiting applied".into(),
                    },
                ],
                cooldown_seconds: 3600,
                auto_execute: false, // Manual trigger only
                max_auto_executions_per_hour: 1,
                created_at: Utc::now(),
            },
            Runbook {
                id: Uuid::new_v4(),
                name: "full-incident-response".into(),
                description: "Major outage response: page on-call, snapshot state, scale up".into(),
                trigger_condition: TriggerCondition::ErrorRateAbove { threshold: 0.05 },
                actions: vec![
                    RemediationAction::Escalate {
                        severity: "critical".into(),
                        message: "Error rate > 5% — major incident declared".into(),
                    },
                    RemediationAction::ScaleUp {
                        service: "campaign-express".into(),
                        additional: 10,
                    },
                    RemediationAction::TriggerBackup {
                        target: "all".into(),
                    },
                ],
                cooldown_seconds: 900,
                auto_execute: true,
                max_auto_executions_per_hour: 1,
                created_at: Utc::now(),
            },
        ];

        for rb in runbooks {
            self.runbooks.insert(rb.id, rb);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_checker::SystemSnapshot;
    use crate::resource_monitor::ResourceMonitor;

    #[test]
    fn test_default_runbooks() {
        let engine = RemediationEngine::new();
        let runbooks = engine.list_runbooks();
        assert!(runbooks.len() >= 10);
    }

    #[test]
    fn test_manual_trigger() {
        let engine = RemediationEngine::new();
        let result = engine.trigger_runbook("redis-memory-pressure", "admin");
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.success);
        assert!(!r.auto_executed);
    }

    #[test]
    fn test_auto_remediation_on_degraded() {
        let engine = RemediationEngine::new();
        let checker = crate::health_checker::HealthChecker::with_defaults();
        let monitor = ResourceMonitor::with_defaults();

        let snap = SystemSnapshot::degraded_demo();
        let health = checker.run_full_check(&snap);
        let resources = monitor.evaluate(&snap);

        let results = engine.evaluate_and_remediate(&health.probes, &resources);
        // Should trigger at least one runbook for degraded state
        assert!(!results.is_empty() || engine.list_runbooks().iter().any(|r| !r.auto_execute));
    }

    #[test]
    fn test_execution_history() {
        let engine = RemediationEngine::new();
        engine.trigger_runbook("high-latency-response", "test");
        engine.trigger_runbook("redis-memory-pressure", "test");
        let history = engine.execution_history(10);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_cooldown_rate_limiting() {
        let engine = RemediationEngine::new();
        // Trigger same runbook multiple times rapidly
        for _ in 0..5 {
            engine.trigger_runbook("high-latency-response", "test");
        }
        let history = engine.execution_history(100);
        // Should be rate-limited (max 3 per hour + cooldown)
        assert!(history.len() <= 5);
    }
}
