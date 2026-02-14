//! Identity Resolution: identity graph, deterministic/probabilistic linking,
//! merge/unmerge, golden record survivorship with source-of-truth precedence.
//!
//! Addresses FR-IDR-001 through FR-IDR-006.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ─── Identity Types ─────────────────────────────────────────────────────

/// Namespace for an identifier (e.g. "email", "phone", "cookie_id").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityNamespace {
    Email,
    Phone,
    CookieId,
    DeviceId,
    CrmId,
    AdvertisingId,
    Custom(String),
}

/// A single identifier within a namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRecord {
    pub namespace: IdentityNamespace,
    pub value: String,
    pub confidence: f64,
    pub source: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Link type between two identity nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Exact match on a known identifier.
    Deterministic,
    /// Statistical match based on shared attributes.
    Probabilistic,
    /// Manual operator-driven merge.
    Manual,
}

/// An edge in the identity graph linking two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityLink {
    pub id: Uuid,
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    pub link_type: LinkType,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Source-of-truth precedence for survivorship rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePrecedence {
    pub field: String,
    pub sources: Vec<String>,
}

/// A golden record produced by merging identities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenRecord {
    pub id: Uuid,
    pub identities: Vec<IdentityRecord>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub source_precedence: HashMap<String, String>,
    pub merge_history: Vec<MergeEvent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Record of a merge or unmerge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeEvent {
    pub id: Uuid,
    pub operation: MergeOperation,
    pub node_ids: Vec<Uuid>,
    pub reason: String,
    pub performed_by: String,
    pub performed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeOperation {
    Merge,
    Unmerge,
    Link,
    Unlink,
}

// ─── Identity Graph ─────────────────────────────────────────────────────

/// Identity graph service managing identity resolution across all namespaces.
pub struct IdentityGraph {
    /// Golden records by ID.
    records: DashMap<Uuid, GoldenRecord>,
    /// Index: namespace+value -> golden record ID for fast lookup.
    identity_index: DashMap<String, Uuid>,
    /// Links between golden records (reserved for cross-record relationship edges).
    #[allow(dead_code)]
    links: DashMap<Uuid, IdentityLink>,
    /// Default source precedence rules.
    default_precedence: Vec<SourcePrecedence>,
    /// Minimum confidence for probabilistic linking.
    probabilistic_threshold: f64,
}

impl Default for IdentityGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityGraph {
    pub fn new() -> Self {
        Self {
            records: DashMap::new(),
            identity_index: DashMap::new(),
            links: DashMap::new(),
            default_precedence: Self::default_source_precedence(),
            probabilistic_threshold: 0.75,
        }
    }

    /// Set the minimum confidence threshold for probabilistic linking.
    pub fn with_probabilistic_threshold(mut self, threshold: f64) -> Self {
        self.probabilistic_threshold = threshold;
        self
    }

    /// Resolve or create a golden record for the given identities.
    ///
    /// Performs deterministic matching first (exact namespace+value),
    /// then probabilistic matching on shared attributes.
    pub fn resolve(
        &self,
        identities: Vec<IdentityRecord>,
        attributes: HashMap<String, serde_json::Value>,
        source: &str,
    ) -> GoldenRecord {
        // Phase 1: deterministic — look up existing records by exact identity match
        let mut matched_record_ids: HashSet<Uuid> = HashSet::new();
        for identity in &identities {
            let key = index_key(&identity.namespace, &identity.value);
            if let Some(record_id) = self.identity_index.get(&key) {
                matched_record_ids.insert(*record_id);
            }
        }

        if matched_record_ids.len() == 1 {
            // Single match: update the existing golden record
            let record_id = *matched_record_ids.iter().next().unwrap();
            self.update_record(record_id, identities, attributes, source);
            return self.records.get(&record_id).unwrap().clone();
        }

        if matched_record_ids.len() > 1 {
            // Multiple matches: merge them
            let ids: Vec<Uuid> = matched_record_ids.into_iter().collect();
            let merged_id = self.merge_records(&ids, "auto-deterministic", "system");
            self.update_record(merged_id, identities, attributes, source);
            return self.records.get(&merged_id).unwrap().clone();
        }

        // Phase 2: probabilistic — compare attributes for fuzzy matching
        if let Some(best_match) = self.find_probabilistic_match(&attributes) {
            self.update_record(best_match, identities, attributes, source);
            return self.records.get(&best_match).unwrap().clone();
        }

        // Phase 3: no match — create new golden record
        self.create_record(identities, attributes, source)
    }

    /// Create a new golden record.
    fn create_record(
        &self,
        identities: Vec<IdentityRecord>,
        attributes: HashMap<String, serde_json::Value>,
        source: &str,
    ) -> GoldenRecord {
        let now = Utc::now();
        let id = Uuid::new_v4();

        let mut source_precedence = HashMap::new();
        for field in attributes.keys() {
            source_precedence.insert(field.clone(), source.to_string());
        }

        let record = GoldenRecord {
            id,
            identities: identities.clone(),
            attributes,
            source_precedence,
            merge_history: vec![],
            created_at: now,
            updated_at: now,
        };

        // Index all identities
        for identity in &identities {
            let key = index_key(&identity.namespace, &identity.value);
            self.identity_index.insert(key, id);
        }

        self.records.insert(id, record.clone());
        info!(record_id = %id, identity_count = identities.len(), "golden record created");
        record
    }

    /// Update an existing golden record with new identities and attributes.
    fn update_record(
        &self,
        record_id: Uuid,
        new_identities: Vec<IdentityRecord>,
        new_attributes: HashMap<String, serde_json::Value>,
        source: &str,
    ) {
        if let Some(mut entry) = self.records.get_mut(&record_id) {
            // Add new identities (avoid duplicates by namespace+value)
            let existing_keys: HashSet<String> = entry
                .identities
                .iter()
                .map(|i| index_key(&i.namespace, &i.value))
                .collect();

            for identity in &new_identities {
                let key = index_key(&identity.namespace, &identity.value);
                if !existing_keys.contains(&key) {
                    entry.identities.push(identity.clone());
                    self.identity_index.insert(key, record_id);
                }
            }

            // Apply survivorship rules for attributes
            for (field, value) in &new_attributes {
                if self.should_update_field(field, source, entry.source_precedence.get(field)) {
                    entry.attributes.insert(field.clone(), value.clone());
                    entry
                        .source_precedence
                        .insert(field.clone(), source.to_string());
                }
            }

            entry.updated_at = Utc::now();
        }
    }

    /// Merge multiple golden records into one, applying survivorship rules.
    pub fn merge_records(&self, record_ids: &[Uuid], reason: &str, actor: &str) -> Uuid {
        if record_ids.is_empty() {
            return Uuid::nil();
        }
        if record_ids.len() == 1 {
            return record_ids[0];
        }

        let primary_id = record_ids[0];
        let now = Utc::now();

        // Collect data from all records to merge
        let mut all_identities: Vec<IdentityRecord> = Vec::new();
        let mut merged_attributes: HashMap<String, serde_json::Value> = HashMap::new();
        let mut merged_precedence: HashMap<String, String> = HashMap::new();
        let mut merged_history: Vec<MergeEvent> = Vec::new();

        for &rid in record_ids {
            if let Some(record) = self.records.get(&rid) {
                all_identities.extend(record.identities.clone());
                for (field, value) in &record.attributes {
                    if self.should_update_field(
                        field,
                        record
                            .source_precedence
                            .get(field)
                            .map(|s| s.as_str())
                            .unwrap_or("unknown"),
                        merged_precedence.get(field),
                    ) {
                        merged_attributes.insert(field.clone(), value.clone());
                        if let Some(src) = record.source_precedence.get(field) {
                            merged_precedence.insert(field.clone(), src.clone());
                        }
                    }
                }
                merged_history.extend(record.merge_history.clone());
            }
        }

        // Deduplicate identities
        let mut seen_keys = HashSet::new();
        all_identities.retain(|i| {
            let key = index_key(&i.namespace, &i.value);
            seen_keys.insert(key)
        });

        // Record merge event
        merged_history.push(MergeEvent {
            id: Uuid::new_v4(),
            operation: MergeOperation::Merge,
            node_ids: record_ids.to_vec(),
            reason: reason.to_string(),
            performed_by: actor.to_string(),
            performed_at: now,
        });

        // Create merged record under primary ID
        let merged = GoldenRecord {
            id: primary_id,
            identities: all_identities.clone(),
            attributes: merged_attributes,
            source_precedence: merged_precedence,
            merge_history: merged_history,
            created_at: now,
            updated_at: now,
        };

        // Remove secondary records, re-index identities to primary
        for &rid in &record_ids[1..] {
            self.records.remove(&rid);
        }

        // Re-index all identities to primary
        for identity in &all_identities {
            let key = index_key(&identity.namespace, &identity.value);
            self.identity_index.insert(key, primary_id);
        }

        self.records.insert(primary_id, merged);
        info!(
            primary = %primary_id,
            merged_count = record_ids.len(),
            reason = reason,
            "golden records merged"
        );
        primary_id
    }

    /// Unmerge: split identities out of a golden record into a new one.
    pub fn unmerge(
        &self,
        record_id: Uuid,
        identity_keys: &[String],
        reason: &str,
        actor: &str,
    ) -> Option<GoldenRecord> {
        let mut entry = self.records.get_mut(&record_id)?;
        let now = Utc::now();

        let key_set: HashSet<&String> = identity_keys.iter().collect();

        // Partition identities
        let (split_out, remaining): (Vec<IdentityRecord>, Vec<IdentityRecord>) = entry
            .identities
            .drain(..)
            .partition(|i| key_set.contains(&index_key(&i.namespace, &i.value)));

        if split_out.is_empty() || remaining.is_empty() {
            // Restore if split is invalid (can't split everything or nothing)
            entry.identities = if split_out.is_empty() {
                remaining
            } else {
                let mut all = split_out;
                all.extend(remaining);
                all
            };
            return None;
        }

        // Update original record
        entry.identities = remaining;
        entry.merge_history.push(MergeEvent {
            id: Uuid::new_v4(),
            operation: MergeOperation::Unmerge,
            node_ids: vec![record_id],
            reason: reason.to_string(),
            performed_by: actor.to_string(),
            performed_at: now,
        });
        entry.updated_at = now;
        drop(entry);

        // Create new record from split-out identities
        let new_id = Uuid::new_v4();
        let new_record = GoldenRecord {
            id: new_id,
            identities: split_out.clone(),
            attributes: HashMap::new(),
            source_precedence: HashMap::new(),
            merge_history: vec![MergeEvent {
                id: Uuid::new_v4(),
                operation: MergeOperation::Unmerge,
                node_ids: vec![record_id, new_id],
                reason: reason.to_string(),
                performed_by: actor.to_string(),
                performed_at: now,
            }],
            created_at: now,
            updated_at: now,
        };

        // Re-index split identities
        for identity in &split_out {
            let key = index_key(&identity.namespace, &identity.value);
            self.identity_index.insert(key, new_id);
        }

        self.records.insert(new_id, new_record.clone());
        info!(
            original = %record_id,
            new_record = %new_id,
            split_count = split_out.len(),
            "golden record unmerged"
        );
        Some(new_record)
    }

    /// Look up a golden record by any identifier.
    pub fn lookup(&self, namespace: &IdentityNamespace, value: &str) -> Option<GoldenRecord> {
        let key = index_key(namespace, value);
        let record_id = self.identity_index.get(&key)?;
        self.records.get(&*record_id).map(|r| r.clone())
    }

    /// Get a golden record by its ID.
    pub fn get(&self, id: Uuid) -> Option<GoldenRecord> {
        self.records.get(&id).map(|r| r.clone())
    }

    /// List all golden records.
    pub fn list_records(&self) -> Vec<GoldenRecord> {
        self.records.iter().map(|e| e.value().clone()).collect()
    }

    /// Count of golden records.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Find a probabilistic match by comparing attributes.
    fn find_probabilistic_match(
        &self,
        attributes: &HashMap<String, serde_json::Value>,
    ) -> Option<Uuid> {
        let mut best_id: Option<Uuid> = None;
        let mut best_score: f64 = 0.0;

        for entry in self.records.iter() {
            let record = entry.value();
            let score = self.compute_similarity(attributes, &record.attributes);
            if score > self.probabilistic_threshold && score > best_score {
                best_score = score;
                best_id = Some(record.id);
            }
        }

        if let Some(id) = best_id {
            info!(
                record_id = %id,
                confidence = best_score,
                "probabilistic identity match"
            );
        }
        best_id
    }

    /// Compute Jaccard-like similarity between two attribute maps.
    fn compute_similarity(
        &self,
        a: &HashMap<String, serde_json::Value>,
        b: &HashMap<String, serde_json::Value>,
    ) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 0.0;
        }
        let all_keys: HashSet<&String> = a.keys().chain(b.keys()).collect();
        if all_keys.is_empty() {
            return 0.0;
        }
        let matching = all_keys
            .iter()
            .filter(|k| a.get(**k) == b.get(**k) && a.contains_key(**k))
            .count();
        matching as f64 / all_keys.len() as f64
    }

    /// Check if a field should be updated based on source precedence.
    fn should_update_field(
        &self,
        field: &str,
        new_source: &str,
        current_source: Option<&String>,
    ) -> bool {
        let current = match current_source {
            Some(s) => s.as_str(),
            None => return true,
        };
        if current == new_source {
            return true;
        }
        // Apply source precedence ordering
        for rule in &self.default_precedence {
            if rule.field == field || rule.field == "*" {
                let cur_rank = rule.sources.iter().position(|s| s == current);
                let new_rank = rule.sources.iter().position(|s| s == new_source);
                return match (cur_rank, new_rank) {
                    (Some(c), Some(n)) => n <= c,
                    (None, Some(_)) => true,
                    _ => false,
                };
            }
        }
        // No rule: accept the update
        true
    }

    /// Default source-of-truth precedence: CRM > CDP > website > inferred.
    fn default_source_precedence() -> Vec<SourcePrecedence> {
        vec![SourcePrecedence {
            field: "*".to_string(),
            sources: vec![
                "crm".to_string(),
                "cdp".to_string(),
                "website".to_string(),
                "mobile_app".to_string(),
                "inferred".to_string(),
            ],
        }]
    }

    /// Seed demo identity data.
    pub fn seed_demo_data(&self) {
        let now = Utc::now();

        // User 1: multi-identity
        let mut attrs1 = HashMap::new();
        attrs1.insert("first_name".to_string(), serde_json::json!("Alice"));
        attrs1.insert("last_name".to_string(), serde_json::json!("Johnson"));
        attrs1.insert("ltv".to_string(), serde_json::json!(1250.0));

        self.resolve(
            vec![
                IdentityRecord {
                    namespace: IdentityNamespace::Email,
                    value: "alice@example.com".to_string(),
                    confidence: 1.0,
                    source: "crm".to_string(),
                    verified: true,
                    created_at: now,
                    last_seen: now,
                },
                IdentityRecord {
                    namespace: IdentityNamespace::Phone,
                    value: "+1-555-0100".to_string(),
                    confidence: 1.0,
                    source: "crm".to_string(),
                    verified: true,
                    created_at: now,
                    last_seen: now,
                },
                IdentityRecord {
                    namespace: IdentityNamespace::CookieId,
                    value: "ck_abc123def456".to_string(),
                    confidence: 0.85,
                    source: "website".to_string(),
                    verified: false,
                    created_at: now,
                    last_seen: now,
                },
            ],
            attrs1,
            "crm",
        );

        // User 2: single ID
        let mut attrs2 = HashMap::new();
        attrs2.insert("first_name".to_string(), serde_json::json!("Bob"));
        attrs2.insert("last_name".to_string(), serde_json::json!("Smith"));

        self.resolve(
            vec![IdentityRecord {
                namespace: IdentityNamespace::Email,
                value: "bob@example.com".to_string(),
                confidence: 1.0,
                source: "cdp".to_string(),
                verified: true,
                created_at: now,
                last_seen: now,
            }],
            attrs2,
            "cdp",
        );

        info!(
            count = self.records.len(),
            "identity graph demo data seeded"
        );
    }
}

/// Build a unique index key for an identity.
fn index_key(namespace: &IdentityNamespace, value: &str) -> String {
    let ns = match namespace {
        IdentityNamespace::Email => "email",
        IdentityNamespace::Phone => "phone",
        IdentityNamespace::CookieId => "cookie_id",
        IdentityNamespace::DeviceId => "device_id",
        IdentityNamespace::CrmId => "crm_id",
        IdentityNamespace::AdvertisingId => "advertising_id",
        IdentityNamespace::Custom(s) => s.as_str(),
    };
    format!("{ns}:{value}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_creates_new_record() {
        let graph = IdentityGraph::new();
        let now = Utc::now();

        let mut attrs = HashMap::new();
        attrs.insert("first_name".to_string(), serde_json::json!("Alice"));

        let record = graph.resolve(
            vec![IdentityRecord {
                namespace: IdentityNamespace::Email,
                value: "alice@test.com".to_string(),
                confidence: 1.0,
                source: "crm".to_string(),
                verified: true,
                created_at: now,
                last_seen: now,
            }],
            attrs,
            "crm",
        );

        assert_eq!(record.identities.len(), 1);
        assert_eq!(graph.record_count(), 1);
    }

    #[test]
    fn test_resolve_deduplicates_on_same_identity() {
        let graph = IdentityGraph::new();
        let now = Utc::now();

        let mk_identity = || IdentityRecord {
            namespace: IdentityNamespace::Email,
            value: "alice@test.com".to_string(),
            confidence: 1.0,
            source: "crm".to_string(),
            verified: true,
            created_at: now,
            last_seen: now,
        };

        graph.resolve(vec![mk_identity()], HashMap::new(), "crm");
        graph.resolve(vec![mk_identity()], HashMap::new(), "crm");

        assert_eq!(graph.record_count(), 1);
    }

    #[test]
    fn test_merge_and_unmerge() {
        let graph = IdentityGraph::new();
        let now = Utc::now();

        let r1 = graph.resolve(
            vec![IdentityRecord {
                namespace: IdentityNamespace::Email,
                value: "a@test.com".to_string(),
                confidence: 1.0,
                source: "crm".to_string(),
                verified: true,
                created_at: now,
                last_seen: now,
            }],
            HashMap::new(),
            "crm",
        );

        let r2 = graph.resolve(
            vec![IdentityRecord {
                namespace: IdentityNamespace::Phone,
                value: "+1-555-0001".to_string(),
                confidence: 1.0,
                source: "crm".to_string(),
                verified: true,
                created_at: now,
                last_seen: now,
            }],
            HashMap::new(),
            "crm",
        );

        assert_eq!(graph.record_count(), 2);

        // Merge
        let merged_id = graph.merge_records(&[r1.id, r2.id], "same person", "admin");
        assert_eq!(graph.record_count(), 1);

        let merged = graph.get(merged_id).unwrap();
        assert_eq!(merged.identities.len(), 2);

        // Unmerge the phone identity back out
        let new_record = graph.unmerge(
            merged_id,
            &["phone:+1-555-0001".to_string()],
            "mistake",
            "admin",
        );
        assert!(new_record.is_some());
        assert_eq!(graph.record_count(), 2);
    }

    #[test]
    fn test_lookup_by_identity() {
        let graph = IdentityGraph::new();
        let now = Utc::now();

        graph.resolve(
            vec![IdentityRecord {
                namespace: IdentityNamespace::Email,
                value: "lookup@test.com".to_string(),
                confidence: 1.0,
                source: "crm".to_string(),
                verified: true,
                created_at: now,
                last_seen: now,
            }],
            HashMap::new(),
            "crm",
        );

        let found = graph.lookup(&IdentityNamespace::Email, "lookup@test.com");
        assert!(found.is_some());

        let not_found = graph.lookup(&IdentityNamespace::Email, "missing@test.com");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_survivorship_source_precedence() {
        let graph = IdentityGraph::new();
        let now = Utc::now();

        let mk_id = || IdentityRecord {
            namespace: IdentityNamespace::Email,
            value: "surv@test.com".to_string(),
            confidence: 1.0,
            source: "crm".to_string(),
            verified: true,
            created_at: now,
            last_seen: now,
        };

        // First: set name from website
        let mut attrs1 = HashMap::new();
        attrs1.insert("first_name".to_string(), serde_json::json!("alice_web"));
        graph.resolve(vec![mk_id()], attrs1, "website");

        // Second: override from CRM (higher precedence)
        let mut attrs2 = HashMap::new();
        attrs2.insert("first_name".to_string(), serde_json::json!("Alice"));
        graph.resolve(vec![mk_id()], attrs2, "crm");

        let record = graph
            .lookup(&IdentityNamespace::Email, "surv@test.com")
            .unwrap();
        assert_eq!(record.attributes["first_name"], "Alice");
        assert_eq!(record.source_precedence["first_name"], "crm");
    }
}
