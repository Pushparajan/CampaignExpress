//! Audit logging: tamper-evident event store with cryptographic hash chaining,
//! query capabilities, compliance reporting, and data access event logging.
//!
//! Addresses FR-SEC-003 through FR-SEC-005.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;
use uuid::Uuid;

/// A single audit event with tamper-evident hash chaining.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub sequence: u64,
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
    /// SHA-256 hash of this event's content.
    pub event_hash: String,
    /// Hash of the previous event in the chain (empty for genesis).
    pub previous_hash: String,
}

/// Category of data access for access logging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataAccessType {
    Read,
    Write,
    Delete,
    Export,
    Anonymize,
}

/// A data access event (who accessed what data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub access_type: DataAccessType,
    pub resource_type: String,
    pub resource_id: String,
    pub fields_accessed: Vec<String>,
    pub pii_accessed: bool,
    pub timestamp: DateTime<Utc>,
}

/// Tamper-evident append-only audit log with hash chaining.
pub struct AuditLogger {
    events: DashMap<Uuid, AuditEvent>,
    data_access_log: DashMap<Uuid, DataAccessEvent>,
    sequence: parking_lot::Mutex<u64>,
    last_hash: parking_lot::Mutex<String>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger {
    /// Create a new empty audit logger with genesis hash.
    pub fn new() -> Self {
        Self {
            events: DashMap::new(),
            data_access_log: DashMap::new(),
            sequence: parking_lot::Mutex::new(0),
            last_hash: parking_lot::Mutex::new("genesis".to_string()),
        }
    }

    /// Append an event to the log with hash chaining.
    pub fn log(&self, event: AuditEvent) {
        let (seq, chained_event) = self.chain_event(event);
        info!(
            event_id = %chained_event.id,
            sequence = seq,
            action = %chained_event.action,
            resource = %chained_event.resource_type,
            "Audit event logged (hash-chained)"
        );
        self.events.insert(chained_event.id, chained_event);
    }

    /// Create a new audit event and log it (convenience builder).
    #[allow(clippy::too_many_arguments)]
    pub fn log_action(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        action: String,
        resource_type: String,
        resource_id: String,
        details: serde_json::Value,
        ip_address: Option<String>,
        compliance_flags: Vec<String>,
    ) -> AuditEvent {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            sequence: 0,
            tenant_id,
            user_id,
            action,
            resource_type,
            resource_id,
            details,
            ip_address,
            user_agent: None,
            timestamp: Utc::now(),
            compliance_flags,
            event_hash: String::new(),
            previous_hash: String::new(),
        };
        let (_, chained) = self.chain_event(event);
        self.events.insert(chained.id, chained.clone());
        chained
    }

    /// Log a data access event (for SOC2 / compliance).
    #[allow(clippy::too_many_arguments)]
    pub fn log_data_access(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        access_type: DataAccessType,
        resource_type: String,
        resource_id: String,
        fields_accessed: Vec<String>,
        pii_accessed: bool,
    ) -> DataAccessEvent {
        let event = DataAccessEvent {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            access_type,
            resource_type,
            resource_id,
            fields_accessed,
            pii_accessed,
            timestamp: Utc::now(),
        };
        self.data_access_log.insert(event.id, event.clone());
        event
    }

    /// Chain an event: assign sequence, compute hash, link to previous.
    fn chain_event(&self, mut event: AuditEvent) -> (u64, AuditEvent) {
        let mut seq = self.sequence.lock();
        *seq += 1;
        event.sequence = *seq;

        let mut prev_hash = self.last_hash.lock();
        event.previous_hash = prev_hash.clone();

        // Compute SHA-256 over: sequence + action + resource + timestamp + previous_hash
        let content = format!(
            "{}:{}:{}:{}:{}:{}",
            event.sequence,
            event.action,
            event.resource_type,
            event.resource_id,
            event.timestamp.to_rfc3339(),
            event.previous_hash,
        );
        let hash = sha256_hex(&content);
        event.event_hash = hash.clone();
        *prev_hash = hash;

        (*seq, event)
    }

    /// Verify the integrity of the audit chain.
    pub fn verify_chain(&self) -> ChainVerification {
        let mut events: Vec<AuditEvent> = self.events.iter().map(|e| e.value().clone()).collect();
        events.sort_by_key(|e| e.sequence);

        let total = events.len();
        let mut valid = 0;
        let mut tampered = Vec::new();
        let mut expected_prev = "genesis".to_string();

        for event in &events {
            if event.previous_hash != expected_prev {
                tampered.push(event.sequence);
            } else {
                // Re-compute hash to verify
                let content = format!(
                    "{}:{}:{}:{}:{}:{}",
                    event.sequence,
                    event.action,
                    event.resource_type,
                    event.resource_id,
                    event.timestamp.to_rfc3339(),
                    event.previous_hash,
                );
                let expected_hash = sha256_hex(&content);
                if expected_hash == event.event_hash {
                    valid += 1;
                } else {
                    tampered.push(event.sequence);
                }
            }
            expected_prev = event.event_hash.clone();
        }

        ChainVerification {
            total_events: total,
            valid_events: valid,
            tampered_sequences: tampered,
            chain_intact: valid == total,
        }
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

    /// Query data access events for compliance reporting.
    pub fn query_data_access(
        &self,
        tenant_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        pii_only: bool,
    ) -> Vec<DataAccessEvent> {
        self.data_access_log
            .iter()
            .filter(|e| {
                let ev = e.value();
                if ev.tenant_id != tenant_id {
                    return false;
                }
                if pii_only && !ev.pii_accessed {
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
                true
            })
            .map(|e| e.value().clone())
            .collect()
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

        // Data access summary
        let data_access_events = self.query_data_access(tenant_id, Some(from), Some(to), false);
        let pii_access_count = data_access_events.iter().filter(|e| e.pii_accessed).count();

        // Chain verification
        let chain = self.verify_chain();

        serde_json::json!({
            "tenant_id": tenant_id,
            "period": { "from": from, "to": to },
            "total_events": total,
            "compliance_flagged_events": compliance_flagged,
            "events_by_action": action_counts,
            "data_access_events": data_access_events.len(),
            "pii_access_events": pii_access_count,
            "chain_integrity": {
                "total": chain.total_events,
                "valid": chain.valid_events,
                "intact": chain.chain_intact,
            },
        })
    }
}

/// Result of verifying the audit chain integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerification {
    pub total_events: usize,
    pub valid_events: usize,
    pub tampered_sequences: Vec<u64>,
    pub chain_intact: bool,
}

/// Compute SHA-256 hex digest.
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// ─── RBAC Middleware ────────────────────────────────────────────────────

/// RBAC middleware that enforces permission checks on request paths.
pub struct RbacMiddleware {
    /// Route pattern -> required permission name.
    route_permissions: Vec<(String, String)>,
}

impl Default for RbacMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl RbacMiddleware {
    pub fn new() -> Self {
        Self {
            route_permissions: Self::default_route_permissions(),
        }
    }

    /// Check if a user's permission set allows access to a route.
    pub fn check_access(
        &self,
        route: &str,
        method: &str,
        user_permissions: &[String],
    ) -> AccessDecision {
        // Find matching route
        for (pattern, required_perm) in &self.route_permissions {
            if route_matches(route, pattern) {
                // Write methods need write permission
                let effective_perm = if matches!(method, "POST" | "PUT" | "DELETE" | "PATCH") {
                    required_perm.replace("_read", "_write")
                } else {
                    required_perm.clone()
                };

                if user_permissions.contains(&effective_perm)
                    || user_permissions.contains(&"system_admin".to_string())
                {
                    return AccessDecision::Allowed;
                }

                return AccessDecision::Denied {
                    required_permission: effective_perm,
                    route: route.to_string(),
                };
            }
        }

        // No matching route rule — allow (open endpoint)
        AccessDecision::Allowed
    }

    fn default_route_permissions() -> Vec<(String, String)> {
        vec![
            (
                "/api/v1/management/campaigns".to_string(),
                "campaign_read".to_string(),
            ),
            (
                "/api/v1/management/creatives".to_string(),
                "creative_read".to_string(),
            ),
            (
                "/api/v1/management/journeys".to_string(),
                "journey_read".to_string(),
            ),
            (
                "/api/v1/management/experiments".to_string(),
                "experiment_read".to_string(),
            ),
            ("/api/v1/management/dco".to_string(), "dco_read".to_string()),
            ("/api/v1/management/cdp".to_string(), "cdp_read".to_string()),
            (
                "/api/v1/management/monitoring".to_string(),
                "analytics_read".to_string(),
            ),
            (
                "/api/v1/management/billing".to_string(),
                "billing_read".to_string(),
            ),
            (
                "/api/v1/management/users".to_string(),
                "user_manage".to_string(),
            ),
            (
                "/api/v1/management/platform".to_string(),
                "tenant_admin".to_string(),
            ),
            (
                "/api/v1/management/ops".to_string(),
                "system_admin".to_string(),
            ),
        ]
    }
}

/// Result of an RBAC access check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDecision {
    Allowed,
    Denied {
        required_permission: String,
        route: String,
    },
}

/// Simple route prefix matching.
fn route_matches(route: &str, pattern: &str) -> bool {
    route.starts_with(pattern)
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
                sequence: 0,
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
                event_hash: String::new(),
                previous_hash: String::new(),
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

    #[test]
    fn test_hash_chain_integrity() {
        let logger = AuditLogger::new();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        for i in 0..5 {
            logger.log(AuditEvent {
                id: Uuid::new_v4(),
                sequence: 0,
                tenant_id,
                user_id,
                action: format!("action_{i}"),
                resource_type: "test".into(),
                resource_id: format!("res-{i}"),
                details: serde_json::json!({}),
                ip_address: None,
                user_agent: None,
                timestamp: Utc::now(),
                compliance_flags: vec![],
                event_hash: String::new(),
                previous_hash: String::new(),
            });
        }

        let verification = logger.verify_chain();
        assert_eq!(verification.total_events, 5);
        assert_eq!(verification.valid_events, 5);
        assert!(verification.chain_intact);
        assert!(verification.tampered_sequences.is_empty());
    }

    #[test]
    fn test_data_access_logging() {
        let logger = AuditLogger::new();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        logger.log_data_access(
            tenant_id,
            user_id,
            DataAccessType::Read,
            "user_profile".to_string(),
            "usr-001".to_string(),
            vec!["email".to_string(), "phone".to_string()],
            true,
        );

        logger.log_data_access(
            tenant_id,
            user_id,
            DataAccessType::Read,
            "campaign".to_string(),
            "camp-001".to_string(),
            vec!["name".to_string(), "budget".to_string()],
            false,
        );

        let all = logger.query_data_access(tenant_id, None, None, false);
        assert_eq!(all.len(), 2);

        let pii_only = logger.query_data_access(tenant_id, None, None, true);
        assert_eq!(pii_only.len(), 1);
        assert!(pii_only[0].pii_accessed);
    }

    #[test]
    fn test_rbac_middleware() {
        let middleware = RbacMiddleware::new();

        // Admin should have access
        let result = middleware.check_access(
            "/api/v1/management/campaigns",
            "GET",
            &["system_admin".to_string()],
        );
        assert_eq!(result, AccessDecision::Allowed);

        // Campaign read permission
        let result = middleware.check_access(
            "/api/v1/management/campaigns",
            "GET",
            &["campaign_read".to_string()],
        );
        assert_eq!(result, AccessDecision::Allowed);

        // POST requires write permission
        let result = middleware.check_access(
            "/api/v1/management/campaigns",
            "POST",
            &["campaign_read".to_string()],
        );
        assert!(matches!(result, AccessDecision::Denied { .. }));

        // Campaign write allows POST
        let result = middleware.check_access(
            "/api/v1/management/campaigns",
            "POST",
            &["campaign_write".to_string()],
        );
        assert_eq!(result, AccessDecision::Allowed);
    }
}
