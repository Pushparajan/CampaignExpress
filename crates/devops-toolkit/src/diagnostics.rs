//! Diagnostics — full-stack triage tool that runs all checks and produces
//! a unified diagnosis for rapid incident response.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auto_remediation::{RemediationEngine, RemediationResult};
use crate::health_checker::{ClusterHealthReport, ComponentHealth, HealthChecker, SystemSnapshot};
use crate::incident_detector::{AnomalySeverity, IncidentDetectionReport, IncidentDetector};
use crate::resource_monitor::{ResourceMonitor, ResourceReport, ResourceSeverity};

/// Overall system diagnosis severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisSeverity {
    Healthy,
    Attention,
    Warning,
    Critical,
    Emergency,
}

/// Unified system diagnosis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDiagnosis {
    pub severity: DiagnosisSeverity,
    pub summary: String,
    pub health: ClusterHealthReport,
    pub resources: ResourceReport,
    pub incidents: IncidentDetectionReport,
    pub actions_taken: Vec<RemediationResult>,
    pub action_items: Vec<ActionItem>,
    pub generated_at: DateTime<Utc>,
}

/// A prioritized action item for the ops team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub priority: u8,
    pub category: String,
    pub title: String,
    pub description: String,
    pub suggested_command: Option<String>,
}

/// The main diagnostics runner that orchestrates all tools.
pub struct DiagnosticsRunner {
    health_checker: HealthChecker,
    resource_monitor: ResourceMonitor,
    incident_detector: IncidentDetector,
    remediation_engine: RemediationEngine,
}

impl Default for DiagnosticsRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsRunner {
    pub fn new() -> Self {
        Self {
            health_checker: HealthChecker::with_defaults(),
            resource_monitor: ResourceMonitor::with_defaults(),
            incident_detector: IncidentDetector::new(),
            remediation_engine: RemediationEngine::new(),
        }
    }

    /// Run full-stack diagnosis against a system snapshot.
    pub fn diagnose(&self, snapshot: &SystemSnapshot) -> SystemDiagnosis {
        // 1. Health check
        let health = self.health_checker.run_full_check(snapshot);

        // 2. Resource evaluation
        let resources = self.resource_monitor.evaluate(snapshot);

        // 3. Incident detection
        let incidents = self.incident_detector.detect(snapshot);

        // 4. Auto-remediation
        let actions_taken = self
            .remediation_engine
            .evaluate_and_remediate(&health.probes, &resources);

        // 5. Generate action items
        let action_items = self.generate_action_items(&health, &resources, &incidents);

        // 6. Compute overall severity
        let severity = self.compute_severity(&health, &resources, &incidents);

        let summary = self.generate_summary(&health, &resources, &incidents, &actions_taken);

        SystemDiagnosis {
            severity,
            summary,
            health,
            resources,
            incidents,
            actions_taken,
            action_items,
            generated_at: Utc::now(),
        }
    }

    /// Render a text-based triage report for terminal output.
    pub fn render_triage(diagnosis: &SystemDiagnosis) -> String {
        let mut out = String::new();

        let severity_icon = match diagnosis.severity {
            DiagnosisSeverity::Healthy => "[OK]",
            DiagnosisSeverity::Attention => "[--]",
            DiagnosisSeverity::Warning => "[!!]",
            DiagnosisSeverity::Critical => "[XX]",
            DiagnosisSeverity::Emergency => "[!!EMERGENCY!!]",
        };

        out.push_str(&format!(
            "\n{} SYSTEM DIAGNOSIS — {}\n",
            severity_icon,
            diagnosis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        out.push_str(&format!("{}\n\n", "=".repeat(60)));
        out.push_str(&format!("Summary: {}\n\n", diagnosis.summary));

        // Health probes
        out.push_str("HEALTH PROBES:\n");
        for probe in &diagnosis.health.probes {
            let icon = match probe.status {
                ComponentHealth::Healthy => " OK ",
                ComponentHealth::Degraded => "WARN",
                ComponentHealth::Unhealthy => "FAIL",
                ComponentHealth::Unknown => " ?? ",
            };
            out.push_str(&format!(
                "  [{}] {:<20} {}ms  {}\n",
                icon, probe.component, probe.latency_ms, probe.message
            ));
        }

        // Resources
        out.push_str("\nRESOURCE UTILIZATION:\n");
        for metric in &diagnosis.resources.metrics {
            let icon = match metric.severity {
                ResourceSeverity::Normal => " OK ",
                ResourceSeverity::Warning => "WARN",
                ResourceSeverity::Critical => "CRIT",
            };
            out.push_str(&format!(
                "  [{}] {:<20} {:.1}%",
                icon, metric.name, metric.usage_pct
            ));
            if let Some(rec) = &metric.recommendation {
                out.push_str(&format!("  -> {rec}"));
            }
            out.push('\n');
        }

        // SLO Status
        out.push_str("\nSLO STATUS:\n");
        for slo in &diagnosis.incidents.slo_statuses {
            let icon = if slo.error_budget_remaining_pct > 50.0 {
                " OK "
            } else if slo.error_budget_remaining_pct > 10.0 {
                "WARN"
            } else {
                "CRIT"
            };
            out.push_str(&format!(
                "  [{}] {:<20} {:.2}% uptime  budget: {:.1}% remaining\n",
                icon, slo.name, slo.current_pct, slo.error_budget_remaining_pct
            ));
        }

        // Anomalies
        if !diagnosis.incidents.anomalies.is_empty() {
            out.push_str("\nANOMALIES DETECTED:\n");
            for a in &diagnosis.incidents.anomalies {
                out.push_str(&format!(
                    "  [{:?}] {} — {}\n",
                    a.severity, a.metric_name, a.message
                ));
                out.push_str(&format!("         Action: {}\n", a.suggested_action));
            }
        }

        // Auto-remediation
        if !diagnosis.actions_taken.is_empty() {
            out.push_str("\nAUTO-REMEDIATION EXECUTED:\n");
            for action in &diagnosis.actions_taken {
                out.push_str(&format!(
                    "  [{}] {} — {}\n",
                    if action.success { "OK" } else { "FAIL" },
                    action.runbook_name,
                    action.message,
                ));
            }
        }

        // Action items
        if !diagnosis.action_items.is_empty() {
            out.push_str("\nACTION ITEMS:\n");
            for (i, item) in diagnosis.action_items.iter().enumerate() {
                out.push_str(&format!(
                    "  {}. [P{}] [{}] {}\n",
                    i + 1,
                    item.priority,
                    item.category,
                    item.title
                ));
                out.push_str(&format!("     {}\n", item.description));
                if let Some(cmd) = &item.suggested_command {
                    out.push_str(&format!("     $ {cmd}\n"));
                }
            }
        }

        out.push_str(&format!("\n{}\n", "=".repeat(60)));
        out
    }

    fn compute_severity(
        &self,
        health: &ClusterHealthReport,
        resources: &ResourceReport,
        incidents: &IncidentDetectionReport,
    ) -> DiagnosisSeverity {
        if incidents.highest_severity >= AnomalySeverity::Emergency || health.unhealthy_count >= 2 {
            DiagnosisSeverity::Emergency
        } else if health.unhealthy_count > 0
            || resources.critical_count > 0
            || incidents.highest_severity >= AnomalySeverity::Critical
        {
            DiagnosisSeverity::Critical
        } else if health.degraded_count > 0
            || resources.warning_count > 0
            || incidents.highest_severity >= AnomalySeverity::Warning
        {
            DiagnosisSeverity::Warning
        } else if incidents.slos_at_risk > 0 {
            DiagnosisSeverity::Attention
        } else {
            DiagnosisSeverity::Healthy
        }
    }

    fn generate_summary(
        &self,
        health: &ClusterHealthReport,
        resources: &ResourceReport,
        incidents: &IncidentDetectionReport,
        actions: &[RemediationResult],
    ) -> String {
        let health_str = match health.overall {
            ComponentHealth::Healthy => "All services healthy",
            ComponentHealth::Degraded => &format!("{} services degraded", health.degraded_count),
            ComponentHealth::Unhealthy => &format!("{} services unhealthy", health.unhealthy_count),
            ComponentHealth::Unknown => "Health status unknown",
        };

        let resource_str = if resources.critical_count > 0 {
            format!(", {} resource(s) critical", resources.critical_count)
        } else if resources.warning_count > 0 {
            format!(", {} resource(s) at warning", resources.warning_count)
        } else {
            String::new()
        };

        let anomaly_str = if !incidents.anomalies.is_empty() {
            format!(", {} anomalies detected", incidents.anomalies.len())
        } else {
            String::new()
        };

        let action_str = if !actions.is_empty() {
            format!(", {} auto-remediations executed", actions.len())
        } else {
            String::new()
        };

        format!("{health_str}{resource_str}{anomaly_str}{action_str}")
    }

    fn generate_action_items(
        &self,
        health: &ClusterHealthReport,
        resources: &ResourceReport,
        incidents: &IncidentDetectionReport,
    ) -> Vec<ActionItem> {
        let mut items = Vec::new();

        // Action items from unhealthy probes
        for probe in &health.probes {
            if probe.status == ComponentHealth::Unhealthy {
                items.push(ActionItem {
                    priority: 1,
                    category: "health".into(),
                    title: format!("{} is unhealthy", probe.component),
                    description: probe.message.clone(),
                    suggested_command: Some(format!(
                        "kubectl logs -l app=campaign-express -c {} --tail=100",
                        probe.component
                    )),
                });
            }
        }

        // Action items from critical resources
        for metric in &resources.metrics {
            if metric.severity == ResourceSeverity::Critical {
                items.push(ActionItem {
                    priority: 1,
                    category: "resource".into(),
                    title: format!("{} at {:.1}% capacity", metric.name, metric.usage_pct),
                    description: metric
                        .recommendation
                        .clone()
                        .unwrap_or_else(|| "Scale up or clean old data".into()),
                    suggested_command: None,
                });
            }
        }

        // Action items from anomalies
        for anomaly in &incidents.anomalies {
            items.push(ActionItem {
                priority: match anomaly.severity {
                    AnomalySeverity::Emergency => 1,
                    AnomalySeverity::Critical => 2,
                    AnomalySeverity::Warning => 3,
                    AnomalySeverity::Info => 4,
                },
                category: "anomaly".into(),
                title: anomaly.message.clone(),
                description: anomaly.suggested_action.clone(),
                suggested_command: None,
            });
        }

        // Action items from SLOs at risk
        for slo in &incidents.slo_statuses {
            if slo.error_budget_remaining_pct < 30.0 {
                items.push(ActionItem {
                    priority: 2,
                    category: "slo".into(),
                    title: format!(
                        "{} SLO at risk ({:.1}% budget remaining)",
                        slo.name, slo.error_budget_remaining_pct
                    ),
                    description: format!(
                        "Current uptime {:.2}% vs target {:.2}%. Error budget burn rate: {:.1}x",
                        slo.current_pct, slo.target_pct, slo.burn_rate_1h
                    ),
                    suggested_command: None,
                });
            }
        }

        items.sort_by_key(|i| i.priority);
        items
    }

    /// Access the underlying components for advanced usage.
    pub fn health_checker(&self) -> &HealthChecker {
        &self.health_checker
    }

    pub fn resource_monitor(&self) -> &ResourceMonitor {
        &self.resource_monitor
    }

    pub fn incident_detector(&self) -> &IncidentDetector {
        &self.incident_detector
    }

    pub fn remediation_engine(&self) -> &RemediationEngine {
        &self.remediation_engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_diagnosis() {
        let runner = DiagnosticsRunner::new();
        let snap = SystemSnapshot::healthy_demo();
        let diagnosis = runner.diagnose(&snap);
        assert!(
            diagnosis.severity == DiagnosisSeverity::Healthy
                || diagnosis.severity == DiagnosisSeverity::Attention
        );
        assert!(diagnosis.action_items.is_empty() || diagnosis.action_items[0].priority >= 3);
    }

    #[test]
    fn test_degraded_diagnosis() {
        let runner = DiagnosticsRunner::new();
        let snap = SystemSnapshot::degraded_demo();
        let diagnosis = runner.diagnose(&snap);
        assert!(diagnosis.severity >= DiagnosisSeverity::Warning);
        assert!(!diagnosis.summary.is_empty());
    }

    #[test]
    fn test_emergency_diagnosis() {
        let runner = DiagnosticsRunner::new();
        let mut snap = SystemSnapshot::healthy_demo();
        snap.redis_connected = false;
        snap.nats_connected = false;
        snap.error_rate = 0.15;
        let diagnosis = runner.diagnose(&snap);
        assert!(diagnosis.severity >= DiagnosisSeverity::Critical);
        assert!(!diagnosis.action_items.is_empty());
    }

    #[test]
    fn test_triage_render() {
        let runner = DiagnosticsRunner::new();
        let snap = SystemSnapshot::degraded_demo();
        let diagnosis = runner.diagnose(&snap);
        let text = DiagnosticsRunner::render_triage(&diagnosis);
        assert!(text.contains("SYSTEM DIAGNOSIS"));
        assert!(text.contains("HEALTH PROBES"));
        assert!(text.contains("RESOURCE UTILIZATION"));
        assert!(text.contains("SLO STATUS"));
    }

    #[test]
    fn test_action_items_prioritized() {
        let runner = DiagnosticsRunner::new();
        let mut snap = SystemSnapshot::healthy_demo();
        snap.redis_connected = false;
        let diagnosis = runner.diagnose(&snap);
        if diagnosis.action_items.len() >= 2 {
            assert!(diagnosis.action_items[0].priority <= diagnosis.action_items[1].priority);
        }
    }
}
