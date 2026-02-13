//! Audit logging: immutable event store with query and compliance reporting.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// A single audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub compliance_flags: Vec<String>,
}

/// Append-only audit log backed by DashMap.
pub struct AuditLogger {
    events: DashMap<Uuid, AuditEvent>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger {
    /// Create a new empty audit logger.
    pub fn new() -> Self {
        Self {
            events: DashMap::new(),
        }
    }

    /// Append an event to the log.
    pub fn log(&self, event: AuditEvent) {
        info!(
            event_id = %event.id,
            action = %event.action,
            resource = %event.resource_type,
            "Audit event logged"
        );
        self.events.insert(event.id, event);
    }

    /// Query events for a tenant with optional time range, action filter, and limit.
    pub fn query(
        &self,
        tenant_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        action: Option<&str>,
        limit: usize,
    ) -> Vec<AuditEvent> {
        let mut results: Vec<AuditEvent> = self
            .events
            .iter()
            .filter(|e| {
                let ev = e.value();
                if ev.tenant_id != tenant_id {
                    return false;
                }
                if let Some(ref f) = from {
                    if ev.timestamp < *f {
                        return false;
                    }
                }
                if let Some(ref t) = to {
                    if ev.timestamp > *t {
                        return false;
                    }
                }
                if let Some(a) = action {
                    if ev.action != a {
                        return false;
                    }
                }
                true
            })
            .map(|e| e.value().clone())
            .collect();

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(limit);
        results
    }

    /// Generate a compliance report summarising events in a time range.
    pub fn export_compliance_report(
        &self,
        tenant_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> serde_json::Value {
        let mut action_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();
        let mut total: u64 = 0;
        let mut compliance_flagged: u64 = 0;

        for entry in self.events.iter() {
            let ev = entry.value();
            if ev.tenant_id != tenant_id || ev.timestamp < from || ev.timestamp > to {
                continue;
            }
            *action_counts.entry(ev.action.clone()).or_default() += 1;
            total += 1;
            if !ev.compliance_flags.is_empty() {
                compliance_flagged += 1;
            }
        }

        serde_json::json!({
            "tenant_id": tenant_id,
            "period": { "from": from, "to": to },
            "total_events": total,
            "compliance_flagged_events": compliance_flagged,
            "events_by_action": action_counts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_and_query() {
        let logger = AuditLogger::new();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Log a few events.
        for action in &["campaign.create", "campaign.update", "campaign.delete"] {
            logger.log(AuditEvent {
                id: Uuid::new_v4(),
                tenant_id,
                user_id,
                action: action.to_string(),
                resource_type: "campaign".into(),
                resource_id: Uuid::new_v4().to_string(),
                details: serde_json::json!({"test": true}),
                ip_address: Some("127.0.0.1".into()),
                user_agent: None,
                timestamp: Utc::now(),
                compliance_flags: vec![],
            });
        }

        // Query all events for this tenant.
        let all = logger.query(tenant_id, None, None, None, 100);
        assert_eq!(all.len(), 3);

        // Filter by action.
        let creates = logger.query(tenant_id, None, None, Some("campaign.create"), 100);
        assert_eq!(creates.len(), 1);

        // Compliance report.
        let report = logger.export_compliance_report(
            tenant_id,
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::hours(1),
        );
        assert_eq!(report["total_events"], 3);
    }
}
