//! Privacy and compliance: GDPR/CCPA DSR handling, data anonymization,
//! and compliance framework tracking.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// Data Subject Request type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DsrType {
    /// Right to be forgotten.
    Erasure,
    /// Data export / access.
    Access,
    /// Rectification of inaccurate data.
    Rectification,
    /// Restrict processing.
    Restriction,
    /// Data portability.
    Portability,
}

/// Status of a DSR request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DsrStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// A data subject request (DSR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_identifier: String,
    pub request_type: DsrType,
    pub status: DsrStatus,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub details: serde_json::Value,
}

/// Configuration for PII anonymization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationConfig {
    pub fields_to_hash: Vec<String>,
    pub fields_to_remove: Vec<String>,
    pub retention_days: u32,
}

impl Default for AnonymizationConfig {
    fn default() -> Self {
        Self {
            fields_to_hash: vec![
                "email".into(),
                "name".into(),
                "phone".into(),
                "ip_address".into(),
            ],
            fields_to_remove: vec!["ssn".into(), "credit_card".into(), "password".into()],
            retention_days: 365,
        }
    }
}

/// Supported compliance frameworks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceFramework {
    Gdpr,
    Ccpa,
    Soc2,
    Iso27001,
    Hipaa,
}

/// Status of a compliance framework within the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub framework: ComplianceFramework,
    pub status: String,
    pub last_audit: Option<DateTime<Utc>>,
    pub next_audit: Option<DateTime<Utc>>,
    pub findings: Vec<String>,
}

/// Privacy and compliance manager.
pub struct PrivacyManager {
    requests: DashMap<Uuid, DataSubjectRequest>,
    compliance_status: DashMap<String, ComplianceStatus>,
    anonymization_config: AnonymizationConfig,
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivacyManager {
    /// Create a new privacy manager with default anonymization config.
    pub fn new() -> Self {
        Self {
            requests: DashMap::new(),
            compliance_status: DashMap::new(),
            anonymization_config: AnonymizationConfig::default(),
        }
    }

    /// Submit a new data subject request.
    pub fn submit_dsr(
        &self,
        tenant_id: Uuid,
        user_identifier: String,
        request_type: DsrType,
    ) -> DataSubjectRequest {
        let dsr = DataSubjectRequest {
            id: Uuid::new_v4(),
            tenant_id,
            user_identifier,
            request_type,
            status: DsrStatus::Pending,
            requested_at: Utc::now(),
            completed_at: None,
            details: serde_json::json!({}),
        };

        info!(dsr_id = %dsr.id, dsr_type = ?dsr.request_type, "DSR submitted");
        self.requests.insert(dsr.id, dsr.clone());
        dsr
    }

    /// Process (complete) a DSR. Logs what would be deleted/exported.
    pub fn process_dsr(&self, request_id: Uuid) -> anyhow::Result<DataSubjectRequest> {
        let mut entry = self
            .requests
            .get_mut(&request_id)
            .ok_or_else(|| anyhow::anyhow!("DSR not found: {request_id}"))?;

        entry.status = DsrStatus::Completed;
        entry.completed_at = Some(Utc::now());

        let action_description = match entry.request_type {
            DsrType::Erasure => "All personal data marked for deletion",
            DsrType::Access => "Personal data export prepared",
            DsrType::Rectification => "Data rectification applied",
            DsrType::Restriction => "Processing restriction applied",
            DsrType::Portability => "Portable data export prepared",
        };

        entry.details = serde_json::json!({
            "action": action_description,
            "processed_at": Utc::now(),
            "affected_systems": ["campaign_db", "analytics", "cache"],
        });

        info!(dsr_id = %request_id, "DSR processed");
        Ok(entry.clone())
    }

    /// List all DSRs for a given tenant.
    pub fn list_dsrs(&self, tenant_id: Uuid) -> Vec<DataSubjectRequest> {
        self.requests
            .iter()
            .filter(|e| e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Anonymize PII fields in a JSON value according to the config.
    pub fn anonymize_data(&self, data: &mut serde_json::Value) {
        if let serde_json::Value::Object(map) = data {
            let fields_to_remove: Vec<String> = self
                .anonymization_config
                .fields_to_remove
                .iter()
                .filter(|f| map.contains_key(f.as_str()))
                .cloned()
                .collect();

            for field in &fields_to_remove {
                map.remove(field.as_str());
            }

            for field in &self.anonymization_config.fields_to_hash {
                if let Some(val) = map.get(field.as_str()) {
                    let hashed = format!("ANON_{:x}", simple_hash(&val.to_string()));
                    map.insert(field.clone(), serde_json::Value::String(hashed));
                }
            }
        }
    }

    /// Return compliance status for all tracked frameworks.
    pub fn get_compliance_status(&self) -> Vec<ComplianceStatus> {
        self.compliance_status
            .iter()
            .map(|e| e.value().clone())
            .collect()
    }

    /// Seed compliance framework statuses.
    pub fn seed_compliance_status(&self) {
        let now = Utc::now();

        self.compliance_status.insert(
            "gdpr".into(),
            ComplianceStatus {
                framework: ComplianceFramework::Gdpr,
                status: "compliant".into(),
                last_audit: Some(now - chrono::Duration::days(30)),
                next_audit: Some(now + chrono::Duration::days(335)),
                findings: vec![],
            },
        );

        self.compliance_status.insert(
            "ccpa".into(),
            ComplianceStatus {
                framework: ComplianceFramework::Ccpa,
                status: "compliant".into(),
                last_audit: Some(now - chrono::Duration::days(60)),
                next_audit: Some(now + chrono::Duration::days(305)),
                findings: vec![],
            },
        );

        self.compliance_status.insert(
            "soc2".into(),
            ComplianceStatus {
                framework: ComplianceFramework::Soc2,
                status: "in_progress".into(),
                last_audit: None,
                next_audit: Some(now + chrono::Duration::days(90)),
                findings: vec!["Access logging implementation pending".into()],
            },
        );

        self.compliance_status.insert(
            "iso27001".into(),
            ComplianceStatus {
                framework: ComplianceFramework::Iso27001,
                status: "planned".into(),
                last_audit: None,
                next_audit: Some(now + chrono::Duration::days(180)),
                findings: vec![],
            },
        );

        info!("Compliance statuses seeded");
    }
}

/// Simple non-cryptographic hash for anonymization (development only).
fn simple_hash(input: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(byte));
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_and_process_dsr() {
        let mgr = PrivacyManager::new();
        let tenant_id = Uuid::new_v4();

        let dsr = mgr.submit_dsr(tenant_id, "user@example.com".into(), DsrType::Erasure);
        assert_eq!(dsr.status, DsrStatus::Pending);
        assert!(dsr.completed_at.is_none());

        let processed = mgr.process_dsr(dsr.id).unwrap();
        assert_eq!(processed.status, DsrStatus::Completed);
        assert!(processed.completed_at.is_some());
        assert_eq!(
            processed.details["action"],
            "All personal data marked for deletion"
        );

        // List DSRs.
        let dsrs = mgr.list_dsrs(tenant_id);
        assert_eq!(dsrs.len(), 1);
        assert_eq!(dsrs[0].id, dsr.id);
    }

    #[test]
    fn test_anonymize_data() {
        let mgr = PrivacyManager::new();

        let mut data = serde_json::json!({
            "email": "alice@example.com",
            "name": "Alice Smith",
            "phone": "+1-555-0100",
            "ssn": "123-45-6789",
            "credit_card": "4111111111111111",
            "campaign_id": "camp-123",
        });

        mgr.anonymize_data(&mut data);

        // Removed fields should be gone.
        assert!(data.get("ssn").is_none());
        assert!(data.get("credit_card").is_none());

        // Hashed fields should be anonymized.
        let email = data["email"].as_str().unwrap();
        assert!(email.starts_with("ANON_"));

        let name = data["name"].as_str().unwrap();
        assert!(name.starts_with("ANON_"));

        // Non-PII field should be untouched.
        assert_eq!(data["campaign_id"], "camp-123");
    }
}
