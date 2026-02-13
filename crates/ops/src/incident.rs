//! Incident management and runbook tracking for Campaign Express operations.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Detected,
    Investigating,
    Identified,
    Monitoring,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub affected_components: Vec<String>,
    pub timeline: Vec<IncidentUpdate>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub postmortem_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentUpdate {
    pub message: String,
    pub status: IncidentStatus,
    pub timestamp: DateTime<Utc>,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    pub order: u32,
    pub action: String,
    pub expected_result: String,
    pub escalation_if_failed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub steps: Vec<RunbookStep>,
    pub created_at: DateTime<Utc>,
}

pub struct IncidentManager {
    incidents: DashMap<Uuid, Incident>,
    runbooks: DashMap<Uuid, Runbook>,
}

impl IncidentManager {
    pub fn new() -> Self {
        info!("Incident manager initialized");
        let mgr = Self {
            incidents: DashMap::new(),
            runbooks: DashMap::new(),
        };
        mgr.seed_demo_data();
        mgr
    }

    pub fn create_incident(
        &self,
        title: String,
        description: String,
        severity: IncidentSeverity,
        affected_components: Vec<String>,
    ) -> Incident {
        let now = Utc::now();
        let incident = Incident {
            id: Uuid::new_v4(),
            title,
            description,
            severity,
            status: IncidentStatus::Detected,
            affected_components,
            timeline: vec![IncidentUpdate {
                message: "Incident detected".to_string(),
                status: IncidentStatus::Detected,
                timestamp: now,
                author: "system".to_string(),
            }],
            created_at: now,
            resolved_at: None,
            postmortem_url: None,
        };
        self.incidents.insert(incident.id, incident.clone());
        incident
    }

    pub fn update_incident_status(
        &self,
        id: Uuid,
        status: IncidentStatus,
        message: String,
        author: String,
    ) -> Option<Incident> {
        self.incidents.get_mut(&id).map(|mut entry| {
            let incident = entry.value_mut();
            incident.status = status.clone();
            incident.timeline.push(IncidentUpdate {
                message,
                status,
                timestamp: Utc::now(),
                author,
            });
            incident.clone()
        })
    }

    pub fn resolve_incident(
        &self,
        id: Uuid,
        message: String,
        author: String,
        postmortem_url: Option<String>,
    ) -> Option<Incident> {
        self.incidents.get_mut(&id).map(|mut entry| {
            let incident = entry.value_mut();
            incident.status = IncidentStatus::Resolved;
            incident.resolved_at = Some(Utc::now());
            incident.postmortem_url = postmortem_url;
            incident.timeline.push(IncidentUpdate {
                message,
                status: IncidentStatus::Resolved,
                timestamp: Utc::now(),
                author,
            });
            incident.clone()
        })
    }

    pub fn list_incidents(&self) -> Vec<Incident> {
        let mut incidents: Vec<Incident> =
            self.incidents.iter().map(|r| r.value().clone()).collect();
        incidents.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        incidents
    }

    pub fn create_runbook(
        &self,
        name: String,
        description: String,
        severity: IncidentSeverity,
        steps: Vec<RunbookStep>,
    ) -> Runbook {
        let runbook = Runbook {
            id: Uuid::new_v4(),
            name,
            description,
            severity,
            steps,
            created_at: Utc::now(),
        };
        self.runbooks.insert(runbook.id, runbook.clone());
        runbook
    }

    pub fn list_runbooks(&self) -> Vec<Runbook> {
        self.runbooks.iter().map(|r| r.value().clone()).collect()
    }

    fn seed_demo_data(&self) {
        let now = Utc::now();

        // Runbook 1: High Latency Response
        let high_latency_runbook = Runbook {
            id: Uuid::new_v4(),
            name: "High Latency Response".to_string(),
            description: "Steps to diagnose and resolve high latency in the bidding pipeline"
                .to_string(),
            severity: IncidentSeverity::Major,
            steps: vec![
                RunbookStep {
                    order: 1,
                    action: "Check Prometheus dashboard for latency spike patterns".to_string(),
                    expected_result: "Identify which service(s) are contributing to latency"
                        .to_string(),
                    escalation_if_failed: Some("Contact on-call SRE".to_string()),
                },
                RunbookStep {
                    order: 2,
                    action: "Review NATS queue depth and consumer lag".to_string(),
                    expected_result: "Queue depth should be under 10000 messages".to_string(),
                    escalation_if_failed: Some("Scale NATS consumers".to_string()),
                },
                RunbookStep {
                    order: 3,
                    action: "Check Redis cluster memory and connection pool usage".to_string(),
                    expected_result: "Memory usage below 80%, connections below pool limit"
                        .to_string(),
                    escalation_if_failed: Some(
                        "Flush expired keys or scale Redis shards".to_string(),
                    ),
                },
                RunbookStep {
                    order: 4,
                    action: "Review NPU engine inference latency metrics".to_string(),
                    expected_result: "P99 inference latency under 5ms".to_string(),
                    escalation_if_failed: Some("Restart NPU device plugin pods".to_string()),
                },
                RunbookStep {
                    order: 5,
                    action: "Scale bidding engine replicas if load-related".to_string(),
                    expected_result: "Latency returns to normal within 5 minutes".to_string(),
                    escalation_if_failed: Some("Escalate to engineering lead".to_string()),
                },
            ],
            created_at: now - Duration::days(30),
        };
        self.runbooks
            .insert(high_latency_runbook.id, high_latency_runbook);

        // Runbook 2: Node Failure Recovery
        let node_failure_runbook = Runbook {
            id: Uuid::new_v4(),
            name: "Node Failure Recovery".to_string(),
            description: "Steps to recover from a node failure in the bidding cluster".to_string(),
            severity: IncidentSeverity::Critical,
            steps: vec![
                RunbookStep {
                    order: 1,
                    action: "Verify node status via kubectl get nodes".to_string(),
                    expected_result: "Identify failed node(s) in NotReady state".to_string(),
                    escalation_if_failed: Some(
                        "Check cloud provider console for VM status".to_string(),
                    ),
                },
                RunbookStep {
                    order: 2,
                    action: "Check if pods have been rescheduled to healthy nodes".to_string(),
                    expected_result: "All bidding engine pods running on healthy nodes".to_string(),
                    escalation_if_failed: Some(
                        "Manually cordon failed node and trigger reschedule".to_string(),
                    ),
                },
                RunbookStep {
                    order: 3,
                    action: "Verify NATS consumer group rebalancing".to_string(),
                    expected_result: "All partitions assigned to active consumers".to_string(),
                    escalation_if_failed: Some("Restart NATS consumer pods".to_string()),
                },
                RunbookStep {
                    order: 4,
                    action: "Monitor bid throughput and error rates for 15 minutes".to_string(),
                    expected_result: "Throughput restored to pre-incident levels".to_string(),
                    escalation_if_failed: Some(
                        "Engage infrastructure team for node replacement".to_string(),
                    ),
                },
            ],
            created_at: now - Duration::days(15),
        };
        self.runbooks
            .insert(node_failure_runbook.id, node_failure_runbook);
    }
}

impl Default for IncidentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_lifecycle() {
        let manager = IncidentManager::new();

        // Create an incident
        let incident = manager.create_incident(
            "High latency on bidding engine".to_string(),
            "P99 latency exceeding 50ms threshold".to_string(),
            IncidentSeverity::Major,
            vec!["Bidding Engine".to_string(), "API Gateway".to_string()],
        );
        assert_eq!(incident.status, IncidentStatus::Detected);
        assert_eq!(incident.timeline.len(), 1);

        // Update to investigating
        let updated = manager
            .update_incident_status(
                incident.id,
                IncidentStatus::Investigating,
                "Investigating root cause".to_string(),
                "oncall-engineer".to_string(),
            )
            .unwrap();
        assert_eq!(updated.status, IncidentStatus::Investigating);
        assert_eq!(updated.timeline.len(), 2);

        // Resolve
        let resolved = manager
            .resolve_incident(
                incident.id,
                "Issue resolved by scaling Redis pool".to_string(),
                "oncall-engineer".to_string(),
                Some("https://wiki.internal/postmortems/2024-01-15".to_string()),
            )
            .unwrap();
        assert_eq!(resolved.status, IncidentStatus::Resolved);
        assert!(resolved.resolved_at.is_some());
        assert!(resolved.postmortem_url.is_some());
        assert_eq!(resolved.timeline.len(), 3);

        // Verify runbooks were seeded
        let runbooks = manager.list_runbooks();
        assert_eq!(runbooks.len(), 2);
    }
}
