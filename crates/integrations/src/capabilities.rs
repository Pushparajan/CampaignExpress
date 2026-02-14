//! Connector capability interface and certification harness —
//! declares what each connector supports and validates compliance.
//!
//! Addresses FR-CNX-001 through FR-CNX-003.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ─── Capability Interface (FR-CNX-001) ───────────────────────────────

/// Declares the capabilities of a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorCapability {
    pub connector_id: Uuid,
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub supported_operations: Vec<ConnectorOperation>,
    pub supported_entities: Vec<EntityType>,
    pub authentication: AuthMethod,
    pub rate_limit: Option<RateLimit>,
    pub max_batch_size: u32,
    pub supports_incremental_sync: bool,
    pub supports_webhooks: bool,
    pub supports_schema_discovery: bool,
    pub health_check_endpoint: Option<String>,
    pub registered_at: DateTime<Utc>,
}

/// Operations a connector can perform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorOperation {
    Read,
    Write,
    Upsert,
    Delete,
    Search,
    Subscribe,
    SchemaDiscovery,
    HealthCheck,
}

/// Entity types a connector can handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Profile,
    Segment,
    Event,
    Campaign,
    Creative,
    Product,
    Order,
    Custom(String),
}

/// Authentication method required by the connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    ApiKey,
    OAuth2,
    BasicAuth,
    BearerToken,
    Hmac,
    None,
}

/// Rate limiting specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub daily_quota: Option<u64>,
}

// ─── Connector Health (FR-CNX-002) ───────────────────────────────────

/// Health status of a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorHealth {
    pub connector_id: Uuid,
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub error_rate_percent: f64,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub consecutive_failures: u32,
    pub checked_at: DateTime<Utc>,
}

/// Status levels for connector health.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

// ─── Certification Harness (FR-CNX-003) ──────────────────────────────

/// A certification test case for validating a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationTest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: CertificationCategory,
    pub required: bool,
    pub test_fn: CertificationCheck,
}

/// Categories of certification tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationCategory {
    Authentication,
    DataIntegrity,
    ErrorHandling,
    RateLimiting,
    SchemaCompliance,
    Performance,
    Security,
}

/// What the certification test checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationCheck {
    AuthenticateSuccessfully,
    HandleAuthFailure,
    ReadRecords,
    WriteRecords,
    HandleDuplicates,
    RespectRateLimit,
    HandleTimeout,
    HandleMalformedResponse,
    ValidateSchema,
    HandleLargePayload,
    EncryptCredentials,
    HandlePartialFailure,
}

/// Result of running a certification test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationTestResult {
    pub test_id: Uuid,
    pub test_name: String,
    pub passed: bool,
    pub required: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub details: String,
}

/// Full certification report for a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationReport {
    pub connector_id: Uuid,
    pub connector_name: String,
    pub connector_version: String,
    pub results: Vec<CertificationTestResult>,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub required_passed: usize,
    pub required_failed: usize,
    pub certified: bool,
    pub certified_at: Option<DateTime<Utc>>,
    pub ran_at: DateTime<Utc>,
}

// ─── Capability Registry ─────────────────────────────────────────────

/// Registry for connector capabilities, health monitoring, and certification.
pub struct ConnectorCapabilityRegistry {
    capabilities: DashMap<Uuid, ConnectorCapability>,
    health: DashMap<Uuid, ConnectorHealth>,
    certifications: DashMap<Uuid, CertificationReport>,
}

impl ConnectorCapabilityRegistry {
    pub fn new() -> Self {
        info!("Connector capability registry initialized");
        Self {
            capabilities: DashMap::new(),
            health: DashMap::new(),
            certifications: DashMap::new(),
        }
    }

    /// Get the default certification test suite.
    fn default_tests() -> Vec<CertificationTest> {
        vec![
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Authentication".to_string(),
                description: "Connector authenticates successfully with valid credentials"
                    .to_string(),
                category: CertificationCategory::Authentication,
                required: true,
                test_fn: CertificationCheck::AuthenticateSuccessfully,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Auth Failure Handling".to_string(),
                description: "Connector handles invalid credentials gracefully".to_string(),
                category: CertificationCategory::Authentication,
                required: true,
                test_fn: CertificationCheck::HandleAuthFailure,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Read Records".to_string(),
                description: "Connector reads records from the external system".to_string(),
                category: CertificationCategory::DataIntegrity,
                required: true,
                test_fn: CertificationCheck::ReadRecords,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Write Records".to_string(),
                description: "Connector writes records to the external system".to_string(),
                category: CertificationCategory::DataIntegrity,
                required: true,
                test_fn: CertificationCheck::WriteRecords,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Duplicate Handling".to_string(),
                description: "Connector handles duplicate records correctly".to_string(),
                category: CertificationCategory::DataIntegrity,
                required: true,
                test_fn: CertificationCheck::HandleDuplicates,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Rate Limit Compliance".to_string(),
                description: "Connector respects external API rate limits".to_string(),
                category: CertificationCategory::RateLimiting,
                required: true,
                test_fn: CertificationCheck::RespectRateLimit,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Timeout Handling".to_string(),
                description: "Connector handles request timeouts gracefully".to_string(),
                category: CertificationCategory::ErrorHandling,
                required: true,
                test_fn: CertificationCheck::HandleTimeout,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Malformed Response".to_string(),
                description: "Connector handles unexpected response formats".to_string(),
                category: CertificationCategory::ErrorHandling,
                required: false,
                test_fn: CertificationCheck::HandleMalformedResponse,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Schema Validation".to_string(),
                description: "Connector validates data against expected schema".to_string(),
                category: CertificationCategory::SchemaCompliance,
                required: true,
                test_fn: CertificationCheck::ValidateSchema,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Large Payload".to_string(),
                description: "Connector handles large batch payloads".to_string(),
                category: CertificationCategory::Performance,
                required: false,
                test_fn: CertificationCheck::HandleLargePayload,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Credential Security".to_string(),
                description: "Credentials are encrypted at rest and in transit".to_string(),
                category: CertificationCategory::Security,
                required: true,
                test_fn: CertificationCheck::EncryptCredentials,
            },
            CertificationTest {
                id: Uuid::new_v4(),
                name: "Partial Failure".to_string(),
                description: "Connector handles partial batch failures with per-record errors"
                    .to_string(),
                category: CertificationCategory::ErrorHandling,
                required: true,
                test_fn: CertificationCheck::HandlePartialFailure,
            },
        ]
    }

    /// Register a connector's capabilities.
    pub fn register(&self, capability: ConnectorCapability) {
        info!(
            connector = %capability.name,
            version = %capability.version,
            ops = capability.supported_operations.len(),
            "Connector registered"
        );
        self.capabilities
            .insert(capability.connector_id, capability);
    }

    /// Get capabilities for a connector.
    pub fn get_capabilities(&self, connector_id: &Uuid) -> Option<ConnectorCapability> {
        self.capabilities.get(connector_id).map(|c| c.clone())
    }

    /// Check if a connector supports a specific operation.
    pub fn supports_operation(&self, connector_id: &Uuid, operation: &ConnectorOperation) -> bool {
        self.capabilities
            .get(connector_id)
            .is_some_and(|c| c.supported_operations.contains(operation))
    }

    /// Update connector health status.
    pub fn update_health(
        &self,
        connector_id: Uuid,
        latency_ms: u64,
        success: bool,
        error: Option<String>,
    ) -> ConnectorHealth {
        let now = Utc::now();
        let prev = self.health.get(&connector_id);

        let consecutive_failures = if success {
            0
        } else {
            prev.as_ref()
                .map(|h| h.consecutive_failures + 1)
                .unwrap_or(1)
        };

        let error_rate = if !success {
            // Exponential moving average
            let prev_rate = prev.as_ref().map(|h| h.error_rate_percent).unwrap_or(0.0);
            prev_rate * 0.9 + 10.0
        } else {
            let prev_rate = prev.as_ref().map(|h| h.error_rate_percent).unwrap_or(0.0);
            prev_rate * 0.9
        };

        let status = if consecutive_failures >= 5 {
            HealthStatus::Unhealthy
        } else if consecutive_failures >= 2 {
            HealthStatus::Degraded
        } else if consecutive_failures == 0 {
            HealthStatus::Healthy
        } else if error_rate > 10.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let health = ConnectorHealth {
            connector_id,
            status,
            latency_ms,
            error_rate_percent: error_rate,
            last_successful_sync: if success {
                Some(now)
            } else {
                prev.as_ref().and_then(|h| h.last_successful_sync)
            },
            last_error: if success { None } else { error },
            consecutive_failures,
            checked_at: now,
        };

        drop(prev);
        self.health.insert(connector_id, health.clone());
        health
    }

    /// Get health status for a connector.
    pub fn get_health(&self, connector_id: &Uuid) -> Option<ConnectorHealth> {
        self.health.get(connector_id).map(|h| h.clone())
    }

    /// Run certification tests against a connector's declared capabilities.
    pub fn certify(
        &self,
        connector_id: &Uuid,
        test_results: Vec<(CertificationCheck, bool, String)>,
    ) -> Option<CertificationReport> {
        let cap = self.capabilities.get(connector_id)?;
        let tests = Self::default_tests();

        let results: Vec<CertificationTestResult> = tests
            .iter()
            .map(|test| {
                let result = test_results
                    .iter()
                    .find(|(check, _, _)| *check == test.test_fn);

                match result {
                    Some((_, passed, details)) => CertificationTestResult {
                        test_id: test.id,
                        test_name: test.name.clone(),
                        passed: *passed,
                        required: test.required,
                        duration_ms: 0,
                        error: if *passed { None } else { Some(details.clone()) },
                        details: details.clone(),
                    },
                    None => CertificationTestResult {
                        test_id: test.id,
                        test_name: test.name.clone(),
                        passed: false,
                        required: test.required,
                        duration_ms: 0,
                        error: Some("Test not executed".to_string()),
                        details: "Test was not included in the certification run".to_string(),
                    },
                }
            })
            .collect();

        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let required_passed = results.iter().filter(|r| r.required && r.passed).count();
        let required_failed = results.iter().filter(|r| r.required && !r.passed).count();
        let certified = required_failed == 0;

        let report = CertificationReport {
            connector_id: *connector_id,
            connector_name: cap.name.clone(),
            connector_version: cap.version.clone(),
            results,
            total_tests: total,
            passed,
            failed,
            required_passed,
            required_failed,
            certified,
            certified_at: if certified { Some(Utc::now()) } else { None },
            ran_at: Utc::now(),
        };

        self.certifications.insert(*connector_id, report.clone());
        Some(report)
    }

    /// Check if a connector is certified.
    pub fn is_certified(&self, connector_id: &Uuid) -> bool {
        self.certifications
            .get(connector_id)
            .is_some_and(|r| r.certified)
    }

    /// List all registered connectors.
    pub fn list_connectors(&self) -> Vec<ConnectorCapability> {
        self.capabilities
            .iter()
            .map(|e| e.value().clone())
            .collect()
    }

    /// Get a summary of all connector health statuses.
    pub fn health_summary(&self) -> serde_json::Value {
        let total = self.capabilities.len();
        let healthy = self
            .health
            .iter()
            .filter(|e| e.value().status == HealthStatus::Healthy)
            .count();
        let degraded = self
            .health
            .iter()
            .filter(|e| e.value().status == HealthStatus::Degraded)
            .count();
        let unhealthy = self
            .health
            .iter()
            .filter(|e| e.value().status == HealthStatus::Unhealthy)
            .count();

        serde_json::json!({
            "total_connectors": total,
            "healthy": healthy,
            "degraded": degraded,
            "unhealthy": unhealthy,
            "unknown": total - healthy - degraded - unhealthy,
        })
    }
}

impl Default for ConnectorCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_capability() -> ConnectorCapability {
        ConnectorCapability {
            connector_id: Uuid::new_v4(),
            name: "Salesforce CDP".to_string(),
            version: "2.1.0".to_string(),
            vendor: "Salesforce".to_string(),
            supported_operations: vec![
                ConnectorOperation::Read,
                ConnectorOperation::Write,
                ConnectorOperation::Upsert,
                ConnectorOperation::SchemaDiscovery,
                ConnectorOperation::HealthCheck,
            ],
            supported_entities: vec![EntityType::Profile, EntityType::Segment, EntityType::Event],
            authentication: AuthMethod::OAuth2,
            rate_limit: Some(RateLimit {
                requests_per_second: 10,
                requests_per_minute: 500,
                daily_quota: Some(50_000),
            }),
            max_batch_size: 10_000,
            supports_incremental_sync: true,
            supports_webhooks: true,
            supports_schema_discovery: true,
            health_check_endpoint: Some("/health".to_string()),
            registered_at: Utc::now(),
        }
    }

    #[test]
    fn test_register_and_query_capabilities() {
        let registry = ConnectorCapabilityRegistry::new();
        let cap = make_capability();
        let id = cap.connector_id;

        registry.register(cap);

        let retrieved = registry.get_capabilities(&id).unwrap();
        assert_eq!(retrieved.name, "Salesforce CDP");
        assert!(registry.supports_operation(&id, &ConnectorOperation::Read));
        assert!(!registry.supports_operation(&id, &ConnectorOperation::Delete));
    }

    #[test]
    fn test_health_monitoring() {
        let registry = ConnectorCapabilityRegistry::new();
        let cap = make_capability();
        let id = cap.connector_id;
        registry.register(cap);

        // Successful check
        let health = registry.update_health(id, 50, true, None);
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);

        // Two failures → degraded
        registry.update_health(id, 500, false, Some("Timeout".to_string()));
        let health = registry.update_health(id, 500, false, Some("Connection refused".to_string()));
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.consecutive_failures, 2);

        // Recovery
        let health = registry.update_health(id, 30, true, None);
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_certification_pass() {
        let registry = ConnectorCapabilityRegistry::new();
        let cap = make_capability();
        let id = cap.connector_id;
        registry.register(cap);

        let tests = ConnectorCapabilityRegistry::default_tests();
        let results: Vec<(CertificationCheck, bool, String)> = tests
            .iter()
            .map(|t| (t.test_fn.clone(), true, "Passed".to_string()))
            .collect();

        let report = registry.certify(&id, results).unwrap();
        assert!(report.certified);
        assert_eq!(report.failed, 0);
        assert!(registry.is_certified(&id));
    }

    #[test]
    fn test_certification_fail_required() {
        let registry = ConnectorCapabilityRegistry::new();
        let cap = make_capability();
        let id = cap.connector_id;
        registry.register(cap);

        // Pass everything except authentication (required)
        let tests = ConnectorCapabilityRegistry::default_tests();
        let results: Vec<(CertificationCheck, bool, String)> = tests
            .iter()
            .map(|t| {
                if t.test_fn == CertificationCheck::AuthenticateSuccessfully {
                    (t.test_fn.clone(), false, "Auth failed".to_string())
                } else {
                    (t.test_fn.clone(), true, "Passed".to_string())
                }
            })
            .collect();

        let report = registry.certify(&id, results).unwrap();
        assert!(!report.certified);
        assert!(report.required_failed > 0);
    }

    #[test]
    fn test_health_summary() {
        let registry = ConnectorCapabilityRegistry::new();

        let cap1 = make_capability();
        let id1 = cap1.connector_id;
        registry.register(cap1);
        registry.update_health(id1, 30, true, None);

        let mut cap2 = make_capability();
        cap2.connector_id = Uuid::new_v4();
        cap2.name = "Segment".to_string();
        let id2 = cap2.connector_id;
        registry.register(cap2);
        registry.update_health(id2, 500, false, Some("Error".to_string()));
        registry.update_health(id2, 500, false, Some("Error".to_string()));

        let summary = registry.health_summary();
        assert_eq!(summary["total_connectors"], 2);
        assert_eq!(summary["healthy"], 1);
        assert_eq!(summary["degraded"], 1);
    }
}
