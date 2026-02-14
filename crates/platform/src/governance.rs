//! Data Governance: schema registry, PII classification, lineage tracking,
//! data quality monitoring, and policy-as-code guardrails.
//!
//! Addresses FR-DG-001 through FR-DG-005.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ─── Schema Registry ────────────────────────────────────────────────────

/// Classification of a field's sensitivity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    /// Personally identifiable information (must be protected).
    Pii,
    /// Sensitive personal data (requires explicit consent).
    Sensitive,
    /// Quasi-identifier (can re-identify when combined).
    QuasiIdentifier,
    /// Business confidential.
    Confidential,
    /// Internal use only.
    Internal,
    /// Public / non-sensitive.
    Public,
}

/// A single field definition in a schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub data_type: String,
    pub classification: DataClassification,
    pub required: bool,
    pub pii_type: Option<PiiType>,
    pub retention_days: Option<u32>,
    pub description: String,
}

/// Specific PII type for a classified field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PiiType {
    Email,
    Phone,
    Name,
    Address,
    IpAddress,
    DeviceId,
    SocialSecurityNumber,
    CreditCard,
    DateOfBirth,
    Biometric,
    Location,
    Custom(String),
}

/// A versioned schema in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSchema {
    pub id: Uuid,
    pub name: String,
    pub version: u32,
    pub fields: Vec<FieldDefinition>,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub deprecated: bool,
}

/// Schema registry for tracking data structures across the platform.
pub struct SchemaRegistry {
    schemas: DashMap<String, Vec<RegisteredSchema>>,
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: DashMap::new(),
        }
    }

    /// Register a new schema version. Returns the assigned version number.
    pub fn register(
        &self,
        name: String,
        fields: Vec<FieldDefinition>,
        owner: String,
    ) -> RegisteredSchema {
        let mut versions = self.schemas.entry(name.clone()).or_default();
        let version = versions.len() as u32 + 1;

        let schema = RegisteredSchema {
            id: Uuid::new_v4(),
            name: name.clone(),
            version,
            fields,
            owner,
            created_at: Utc::now(),
            deprecated: false,
        };

        versions.push(schema.clone());
        info!(schema = %name, version = version, "schema registered");
        schema
    }

    /// Get the latest version of a schema.
    pub fn get_latest(&self, name: &str) -> Option<RegisteredSchema> {
        self.schemas.get(name)?.last().cloned()
    }

    /// Get a specific version of a schema.
    pub fn get_version(&self, name: &str, version: u32) -> Option<RegisteredSchema> {
        let versions = self.schemas.get(name)?;
        versions.iter().find(|s| s.version == version).cloned()
    }

    /// List all registered schema names.
    pub fn list_schemas(&self) -> Vec<String> {
        self.schemas.iter().map(|e| e.key().clone()).collect()
    }

    /// Find all fields classified as PII across all schemas.
    pub fn find_pii_fields(&self) -> Vec<(String, FieldDefinition)> {
        let mut results = Vec::new();
        for entry in self.schemas.iter() {
            if let Some(latest) = entry.value().last() {
                for field in &latest.fields {
                    if field.classification == DataClassification::Pii {
                        results.push((latest.name.clone(), field.clone()));
                    }
                }
            }
        }
        results
    }

    /// Seed demo schemas.
    pub fn seed_demo(&self) {
        self.register(
            "user_profile".to_string(),
            vec![
                FieldDefinition {
                    name: "user_id".to_string(),
                    data_type: "string".to_string(),
                    classification: DataClassification::Internal,
                    required: true,
                    pii_type: None,
                    retention_days: None,
                    description: "Internal user identifier".to_string(),
                },
                FieldDefinition {
                    name: "email".to_string(),
                    data_type: "string".to_string(),
                    classification: DataClassification::Pii,
                    required: true,
                    pii_type: Some(PiiType::Email),
                    retention_days: Some(365),
                    description: "User email address".to_string(),
                },
                FieldDefinition {
                    name: "phone".to_string(),
                    data_type: "string".to_string(),
                    classification: DataClassification::Pii,
                    required: false,
                    pii_type: Some(PiiType::Phone),
                    retention_days: Some(365),
                    description: "User phone number".to_string(),
                },
                FieldDefinition {
                    name: "ip_address".to_string(),
                    data_type: "string".to_string(),
                    classification: DataClassification::Pii,
                    required: false,
                    pii_type: Some(PiiType::IpAddress),
                    retention_days: Some(90),
                    description: "Last known IP address".to_string(),
                },
                FieldDefinition {
                    name: "segments".to_string(),
                    data_type: "array<u32>".to_string(),
                    classification: DataClassification::Internal,
                    required: false,
                    pii_type: None,
                    retention_days: None,
                    description: "Segment memberships".to_string(),
                },
            ],
            "platform-team".to_string(),
        );

        self.register(
            "analytics_event".to_string(),
            vec![
                FieldDefinition {
                    name: "event_id".to_string(),
                    data_type: "uuid".to_string(),
                    classification: DataClassification::Internal,
                    required: true,
                    pii_type: None,
                    retention_days: None,
                    description: "Unique event identifier".to_string(),
                },
                FieldDefinition {
                    name: "user_id".to_string(),
                    data_type: "string".to_string(),
                    classification: DataClassification::QuasiIdentifier,
                    required: false,
                    pii_type: None,
                    retention_days: Some(730),
                    description: "Associated user ID".to_string(),
                },
                FieldDefinition {
                    name: "campaign_id".to_string(),
                    data_type: "string".to_string(),
                    classification: DataClassification::Internal,
                    required: false,
                    pii_type: None,
                    retention_days: None,
                    description: "Campaign identifier".to_string(),
                },
            ],
            "analytics-team".to_string(),
        );

        info!("demo schemas seeded");
    }
}

// ─── PII Classifier ─────────────────────────────────────────────────────

/// Automatic PII detection on arbitrary JSON data.
pub struct PiiClassifier {
    /// Patterns: field name substring -> PII type.
    patterns: Vec<(String, PiiType)>,
}

impl Default for PiiClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl PiiClassifier {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                ("email".to_string(), PiiType::Email),
                ("phone".to_string(), PiiType::Phone),
                ("name".to_string(), PiiType::Name),
                ("address".to_string(), PiiType::Address),
                ("ip_address".to_string(), PiiType::IpAddress),
                ("ip_addr".to_string(), PiiType::IpAddress),
                ("device_id".to_string(), PiiType::DeviceId),
                ("ssn".to_string(), PiiType::SocialSecurityNumber),
                ("credit_card".to_string(), PiiType::CreditCard),
                ("card_number".to_string(), PiiType::CreditCard),
                ("dob".to_string(), PiiType::DateOfBirth),
                ("date_of_birth".to_string(), PiiType::DateOfBirth),
                ("latitude".to_string(), PiiType::Location),
                ("longitude".to_string(), PiiType::Location),
            ],
        }
    }

    /// Classify all fields in a JSON object, returning detected PII fields.
    pub fn classify(&self, data: &serde_json::Value) -> Vec<(String, PiiType)> {
        let mut results = Vec::new();
        if let serde_json::Value::Object(map) = data {
            for key in map.keys() {
                let lower = key.to_lowercase();
                for (pattern, pii_type) in &self.patterns {
                    if lower.contains(pattern) {
                        results.push((key.clone(), pii_type.clone()));
                        break;
                    }
                }
            }
        }
        results
    }
}

// ─── Data Lineage ───────────────────────────────────────────────────────

/// A node in the data lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: LineageNodeType,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageNodeType {
    Source,
    Transformation,
    Sink,
    Model,
}

/// An edge in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    pub id: Uuid,
    pub from_node: Uuid,
    pub to_node: Uuid,
    pub transformation: String,
    pub fields_affected: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Data lineage tracker.
pub struct LineageTracker {
    nodes: DashMap<Uuid, LineageNode>,
    edges: DashMap<Uuid, LineageEdge>,
}

impl Default for LineageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LineageTracker {
    pub fn new() -> Self {
        Self {
            nodes: DashMap::new(),
            edges: DashMap::new(),
        }
    }

    /// Register a lineage node (source, transformation, or sink).
    pub fn register_node(
        &self,
        name: String,
        node_type: LineageNodeType,
        metadata: HashMap<String, String>,
    ) -> LineageNode {
        let node = LineageNode {
            id: Uuid::new_v4(),
            name,
            node_type,
            metadata,
            created_at: Utc::now(),
        };
        self.nodes.insert(node.id, node.clone());
        node
    }

    /// Add a lineage edge connecting two nodes.
    pub fn add_edge(
        &self,
        from_node: Uuid,
        to_node: Uuid,
        transformation: String,
        fields_affected: Vec<String>,
    ) -> Option<LineageEdge> {
        if !self.nodes.contains_key(&from_node) || !self.nodes.contains_key(&to_node) {
            return None;
        }
        let edge = LineageEdge {
            id: Uuid::new_v4(),
            from_node,
            to_node,
            transformation,
            fields_affected,
            created_at: Utc::now(),
        };
        self.edges.insert(edge.id, edge.clone());
        Some(edge)
    }

    /// Trace the lineage of data flowing into or out of a node.
    pub fn trace_upstream(&self, node_id: Uuid) -> Vec<LineageNode> {
        let mut visited: HashSet<Uuid> = HashSet::new();
        let mut queue: Vec<Uuid> = vec![node_id];
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(node) = self.nodes.get(&current) {
                result.push(node.clone());
            }
            for edge in self.edges.iter() {
                if edge.to_node == current && !visited.contains(&edge.from_node) {
                    queue.push(edge.from_node);
                }
            }
        }

        result
    }

    /// Trace downstream dependents of a node.
    pub fn trace_downstream(&self, node_id: Uuid) -> Vec<LineageNode> {
        let mut visited: HashSet<Uuid> = HashSet::new();
        let mut queue: Vec<Uuid> = vec![node_id];
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(node) = self.nodes.get(&current) {
                result.push(node.clone());
            }
            for edge in self.edges.iter() {
                if edge.from_node == current && !visited.contains(&edge.to_node) {
                    queue.push(edge.to_node);
                }
            }
        }

        result
    }

    /// Get full lineage graph as JSON.
    pub fn export_graph(&self) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = self
            .nodes
            .iter()
            .map(|e| serde_json::to_value(e.value()).unwrap_or_default())
            .collect();
        let edges: Vec<serde_json::Value> = self
            .edges
            .iter()
            .map(|e| serde_json::to_value(e.value()).unwrap_or_default())
            .collect();

        serde_json::json!({
            "nodes": nodes,
            "edges": edges,
        })
    }

    /// Seed demo lineage data.
    pub fn seed_demo(&self) {
        let cdp_source = self.register_node(
            "CDP Inbound".to_string(),
            LineageNodeType::Source,
            HashMap::from([("platform".to_string(), "salesforce".to_string())]),
        );
        let pii_transform = self.register_node(
            "PII Anonymizer".to_string(),
            LineageNodeType::Transformation,
            HashMap::from([("type".to_string(), "sha256_hash".to_string())]),
        );
        let profile_store = self.register_node(
            "Redis Profile Cache".to_string(),
            LineageNodeType::Sink,
            HashMap::from([("store".to_string(), "redis".to_string())]),
        );
        let analytics_sink = self.register_node(
            "ClickHouse Analytics".to_string(),
            LineageNodeType::Sink,
            HashMap::from([("store".to_string(), "clickhouse".to_string())]),
        );
        let model_node = self.register_node(
            "NPU Inference Model".to_string(),
            LineageNodeType::Model,
            HashMap::from([("engine".to_string(), "npu".to_string())]),
        );

        self.add_edge(
            cdp_source.id,
            pii_transform.id,
            "inbound_sync".to_string(),
            vec!["email".to_string(), "phone".to_string(), "name".to_string()],
        );
        self.add_edge(
            pii_transform.id,
            profile_store.id,
            "store_profile".to_string(),
            vec!["user_id".to_string(), "segments".to_string()],
        );
        self.add_edge(
            profile_store.id,
            model_node.id,
            "inference_input".to_string(),
            vec!["interests".to_string(), "segments".to_string()],
        );
        self.add_edge(
            model_node.id,
            analytics_sink.id,
            "log_inference".to_string(),
            vec!["score".to_string(), "bid_price".to_string()],
        );

        info!("demo lineage data seeded");
    }
}

// ─── Data Quality Monitor ───────────────────────────────────────────────

/// Quality metrics for a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetrics {
    pub source_name: String,
    pub completeness: f64,
    pub accuracy: f64,
    pub freshness_secs: u64,
    pub records_checked: u64,
    pub issues_found: u64,
    pub measured_at: DateTime<Utc>,
}

/// Data quality monitor that tracks metrics per source.
pub struct DataQualityMonitor {
    metrics: DashMap<String, DataQualityMetrics>,
}

impl Default for DataQualityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DataQualityMonitor {
    pub fn new() -> Self {
        Self {
            metrics: DashMap::new(),
        }
    }

    /// Record quality metrics for a data source.
    pub fn record(
        &self,
        source: String,
        completeness: f64,
        accuracy: f64,
        freshness_secs: u64,
        records: u64,
        issues: u64,
    ) {
        let metrics = DataQualityMetrics {
            source_name: source.clone(),
            completeness,
            accuracy,
            freshness_secs,
            records_checked: records,
            issues_found: issues,
            measured_at: Utc::now(),
        };
        self.metrics.insert(source, metrics);
    }

    /// Get quality metrics for all sources.
    pub fn all_metrics(&self) -> Vec<DataQualityMetrics> {
        self.metrics.iter().map(|e| e.value().clone()).collect()
    }

    /// Check if any source has quality below a threshold.
    pub fn check_thresholds(
        &self,
        min_completeness: f64,
        min_accuracy: f64,
    ) -> Vec<DataQualityMetrics> {
        self.metrics
            .iter()
            .filter(|e| {
                e.value().completeness < min_completeness || e.value().accuracy < min_accuracy
            })
            .map(|e| e.value().clone())
            .collect()
    }

    /// Seed demo metrics.
    pub fn seed_demo(&self) {
        self.record("cdp_salesforce".to_string(), 0.97, 0.99, 300, 50_000, 12);
        self.record("cdp_segment".to_string(), 0.95, 0.98, 120, 120_000, 45);
        self.record(
            "analytics_clickhouse".to_string(),
            0.999,
            0.995,
            5,
            10_000_000,
            3,
        );
        self.record("redis_profiles".to_string(), 0.92, 0.97, 1, 2_000_000, 180);
        info!("demo data quality metrics seeded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_registry() {
        let registry = SchemaRegistry::new();
        let schema = registry.register(
            "test_schema".to_string(),
            vec![FieldDefinition {
                name: "email".to_string(),
                data_type: "string".to_string(),
                classification: DataClassification::Pii,
                required: true,
                pii_type: Some(PiiType::Email),
                retention_days: Some(365),
                description: "User email".to_string(),
            }],
            "test-team".to_string(),
        );
        assert_eq!(schema.version, 1);

        // Register another version
        let v2 = registry.register(
            "test_schema".to_string(),
            vec![FieldDefinition {
                name: "email".to_string(),
                data_type: "string".to_string(),
                classification: DataClassification::Pii,
                required: true,
                pii_type: Some(PiiType::Email),
                retention_days: Some(365),
                description: "User email (updated)".to_string(),
            }],
            "test-team".to_string(),
        );
        assert_eq!(v2.version, 2);

        let latest = registry.get_latest("test_schema").unwrap();
        assert_eq!(latest.version, 2);

        let pii = registry.find_pii_fields();
        assert_eq!(pii.len(), 1);
        assert_eq!(pii[0].1.name, "email");
    }

    #[test]
    fn test_pii_classifier() {
        let classifier = PiiClassifier::new();
        let data = serde_json::json!({
            "user_email": "alice@example.com",
            "phone_number": "+1555",
            "campaign_id": "camp-001",
            "home_address": "123 Main St",
        });

        let results = classifier.classify(&data);
        assert_eq!(results.len(), 3); // email, phone, address
    }

    #[test]
    fn test_lineage_tracker() {
        let tracker = LineageTracker::new();

        let source = tracker.register_node(
            "Source".to_string(),
            LineageNodeType::Source,
            HashMap::new(),
        );
        let transform = tracker.register_node(
            "Transform".to_string(),
            LineageNodeType::Transformation,
            HashMap::new(),
        );
        let sink = tracker.register_node("Sink".to_string(), LineageNodeType::Sink, HashMap::new());

        tracker.add_edge(
            source.id,
            transform.id,
            "etl".to_string(),
            vec!["col1".to_string()],
        );
        tracker.add_edge(
            transform.id,
            sink.id,
            "load".to_string(),
            vec!["col1".to_string()],
        );

        let upstream = tracker.trace_upstream(sink.id);
        assert_eq!(upstream.len(), 3); // sink, transform, source

        let downstream = tracker.trace_downstream(source.id);
        assert_eq!(downstream.len(), 3);
    }

    #[test]
    fn test_data_quality_thresholds() {
        let monitor = DataQualityMonitor::new();
        monitor.record("good_source".to_string(), 0.99, 0.99, 10, 1000, 1);
        monitor.record("bad_source".to_string(), 0.80, 0.70, 3600, 500, 50);

        let violations = monitor.check_thresholds(0.90, 0.90);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].source_name, "bad_source");
    }
}
