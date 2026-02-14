//! Online profile/feature store with staleness tracking, computed features,
//! and schema versioning.
//!
//! Addresses FR-PRO-001 through FR-PRO-003.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ─── Feature Definitions (FR-PRO-001) ────────────────────────────────

/// A declared feature in the store with type and freshness requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub name: String,
    pub description: String,
    pub value_type: FeatureValueType,
    pub category: FeatureCategory,
    /// Maximum age in seconds before the feature is considered stale.
    pub ttl_seconds: u64,
    pub source: FeatureSource,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supported value types for features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureValueType {
    Float,
    Integer,
    String,
    Boolean,
    FloatArray,
    StringArray,
}

/// Category of a feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureCategory {
    Demographic,
    Behavioral,
    Transactional,
    Engagement,
    Contextual,
    Computed,
}

/// Where the feature value comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureSource {
    CdpSync,
    RealTimeEvent,
    BatchComputation,
    ManualEntry,
    Derived,
}

// ─── Feature Values (FR-PRO-002) ─────────────────────────────────────

/// A timestamped feature value for a specific user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureValue {
    pub feature_name: String,
    pub value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
    pub source: FeatureSource,
    pub version: u32,
}

/// A user's complete feature vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeatureVector {
    pub user_id: String,
    pub features: Vec<FeatureValue>,
    pub fetched_at: DateTime<Utc>,
    pub stale_features: Vec<String>,
}

// ─── Staleness Alerts (FR-PRO-003) ───────────────────────────────────

/// Alert severity for stale features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StalenessSeverity {
    Warning,
    Critical,
}

/// Alert when a feature value exceeds its TTL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessAlert {
    pub user_id: String,
    pub feature_name: String,
    pub severity: StalenessSeverity,
    pub age_seconds: u64,
    pub ttl_seconds: u64,
    pub last_updated: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
}

/// Computed feature definition — derived from other features at read time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedFeature {
    pub name: String,
    pub description: String,
    pub input_features: Vec<String>,
    pub computation: ComputationType,
}

/// How a computed feature is derived.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputationType {
    DaysSince {
        date_feature: String,
    },
    Ratio {
        numerator: String,
        denominator: String,
    },
    Threshold {
        feature: String,
        threshold: f64,
    },
    Sum {
        features: Vec<String>,
    },
    Average {
        features: Vec<String>,
    },
}

// ─── Feature Store Engine ────────────────────────────────────────────

/// Online feature store with real-time feature serving and staleness detection.
pub struct FeatureStore {
    definitions: DashMap<String, FeatureDefinition>,
    /// user_id -> feature_name -> value
    user_features: DashMap<String, DashMap<String, FeatureValue>>,
    computed_features: DashMap<String, ComputedFeature>,
}

impl FeatureStore {
    pub fn new() -> Self {
        info!("Feature store initialized");
        let store = Self {
            definitions: DashMap::new(),
            user_features: DashMap::new(),
            computed_features: DashMap::new(),
        };
        store.seed_default_definitions();
        store
    }

    fn seed_default_definitions(&self) {
        let defs = vec![
            FeatureDefinition {
                name: "recency_score".to_string(),
                description: "How recently the user was active (0-1)".to_string(),
                value_type: FeatureValueType::Float,
                category: FeatureCategory::Behavioral,
                ttl_seconds: 3600,
                source: FeatureSource::RealTimeEvent,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            FeatureDefinition {
                name: "lifetime_value".to_string(),
                description: "Customer lifetime value in USD".to_string(),
                value_type: FeatureValueType::Float,
                category: FeatureCategory::Transactional,
                ttl_seconds: 86400,
                source: FeatureSource::BatchComputation,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            FeatureDefinition {
                name: "purchase_count_30d".to_string(),
                description: "Number of purchases in last 30 days".to_string(),
                value_type: FeatureValueType::Integer,
                category: FeatureCategory::Transactional,
                ttl_seconds: 3600,
                source: FeatureSource::RealTimeEvent,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            FeatureDefinition {
                name: "email_open_rate".to_string(),
                description: "Rolling 90-day email open rate".to_string(),
                value_type: FeatureValueType::Float,
                category: FeatureCategory::Engagement,
                ttl_seconds: 86400,
                source: FeatureSource::BatchComputation,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            FeatureDefinition {
                name: "preferred_channel".to_string(),
                description: "User's preferred communication channel".to_string(),
                value_type: FeatureValueType::String,
                category: FeatureCategory::Behavioral,
                ttl_seconds: 604800,
                source: FeatureSource::Derived,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            FeatureDefinition {
                name: "geo_region".to_string(),
                description: "User's geographic region".to_string(),
                value_type: FeatureValueType::String,
                category: FeatureCategory::Demographic,
                ttl_seconds: 2592000,
                source: FeatureSource::CdpSync,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        for def in defs {
            self.definitions.insert(def.name.clone(), def);
        }

        // Seed computed features
        self.computed_features.insert(
            "days_since_last_purchase".to_string(),
            ComputedFeature {
                name: "days_since_last_purchase".to_string(),
                description: "Days since the user's last purchase".to_string(),
                input_features: vec!["last_purchase_date".to_string()],
                computation: ComputationType::DaysSince {
                    date_feature: "last_purchase_date".to_string(),
                },
            },
        );

        self.computed_features.insert(
            "conversion_rate".to_string(),
            ComputedFeature {
                name: "conversion_rate".to_string(),
                description: "Purchase-to-visit ratio".to_string(),
                input_features: vec![
                    "purchase_count_30d".to_string(),
                    "visit_count_30d".to_string(),
                ],
                computation: ComputationType::Ratio {
                    numerator: "purchase_count_30d".to_string(),
                    denominator: "visit_count_30d".to_string(),
                },
            },
        );
    }

    /// Register or update a feature definition.
    pub fn define_feature(&self, def: FeatureDefinition) {
        info!(feature = %def.name, version = def.version, "Feature defined");
        self.definitions.insert(def.name.clone(), def);
    }

    /// Update a feature value for a user (real-time ingestion).
    pub fn update_feature(
        &self,
        user_id: &str,
        feature_name: &str,
        value: serde_json::Value,
        source: FeatureSource,
    ) -> Result<(), String> {
        // Validate against definition
        let def = self
            .definitions
            .get(feature_name)
            .ok_or_else(|| format!("Unknown feature: {}", feature_name))?;

        let fv = FeatureValue {
            feature_name: feature_name.to_string(),
            value,
            updated_at: Utc::now(),
            source,
            version: def.version,
        };

        self.user_features
            .entry(user_id.to_string())
            .or_default()
            .insert(feature_name.to_string(), fv);

        Ok(())
    }

    /// Get the full feature vector for a user, including staleness checks.
    pub fn get_features(&self, user_id: &str) -> UserFeatureVector {
        let now = Utc::now();
        let mut features = Vec::new();
        let mut stale = Vec::new();

        if let Some(user_map) = self.user_features.get(user_id) {
            for entry in user_map.iter() {
                let fv = entry.value().clone();

                // Check staleness
                if let Some(def) = self.definitions.get(&fv.feature_name) {
                    let age = (now - fv.updated_at).num_seconds().max(0) as u64;
                    if age > def.ttl_seconds {
                        stale.push(fv.feature_name.clone());
                    }
                }

                features.push(fv);
            }
        }

        UserFeatureVector {
            user_id: user_id.to_string(),
            features,
            fetched_at: now,
            stale_features: stale,
        }
    }

    /// Get a single feature value for a user.
    pub fn get_feature(&self, user_id: &str, feature_name: &str) -> Option<FeatureValue> {
        self.user_features
            .get(user_id)
            .and_then(|m| m.get(feature_name).map(|v| v.clone()))
    }

    /// Check staleness alerts across all users for a specific feature.
    pub fn check_staleness(&self, feature_name: &str) -> Vec<StalenessAlert> {
        let now = Utc::now();
        let def = match self.definitions.get(feature_name) {
            Some(d) => d.clone(),
            None => return Vec::new(),
        };

        let mut alerts = Vec::new();

        for user_entry in self.user_features.iter() {
            let user_id = user_entry.key().clone();
            if let Some(fv) = user_entry.value().get(feature_name) {
                let age = (now - fv.updated_at).num_seconds().max(0) as u64;
                if age > def.ttl_seconds {
                    let severity = if age > def.ttl_seconds * 3 {
                        StalenessSeverity::Critical
                    } else {
                        StalenessSeverity::Warning
                    };

                    warn!(
                        user_id = %user_id,
                        feature = feature_name,
                        age_seconds = age,
                        "Stale feature detected"
                    );

                    alerts.push(StalenessAlert {
                        user_id,
                        feature_name: feature_name.to_string(),
                        severity,
                        age_seconds: age,
                        ttl_seconds: def.ttl_seconds,
                        last_updated: fv.updated_at,
                        detected_at: now,
                    });
                }
            }
        }

        alerts
    }

    /// Compute a derived feature value.
    pub fn compute_feature(&self, user_id: &str, computed_name: &str) -> Option<serde_json::Value> {
        let computed = self.computed_features.get(computed_name)?;

        match &computed.computation {
            ComputationType::DaysSince { date_feature } => {
                let fv = self.get_feature(user_id, date_feature)?;
                let date_str = fv.value.as_str()?;
                let date = DateTime::parse_from_rfc3339(date_str).ok()?;
                let days = (Utc::now() - date.with_timezone(&Utc)).num_days();
                Some(serde_json::json!(days))
            }
            ComputationType::Ratio {
                numerator,
                denominator,
            } => {
                let num = self
                    .get_feature(user_id, numerator)?
                    .value
                    .as_f64()
                    .unwrap_or(0.0);
                let den = self
                    .get_feature(user_id, denominator)?
                    .value
                    .as_f64()
                    .unwrap_or(1.0);
                if den == 0.0 {
                    return Some(serde_json::json!(0.0));
                }
                Some(serde_json::json!(num / den))
            }
            ComputationType::Threshold { feature, threshold } => {
                let val = self
                    .get_feature(user_id, feature)?
                    .value
                    .as_f64()
                    .unwrap_or(0.0);
                Some(serde_json::json!(val >= *threshold))
            }
            ComputationType::Sum { features } => {
                let total: f64 = features
                    .iter()
                    .filter_map(|f| self.get_feature(user_id, f))
                    .filter_map(|fv| fv.value.as_f64())
                    .sum();
                Some(serde_json::json!(total))
            }
            ComputationType::Average { features } => {
                let vals: Vec<f64> = features
                    .iter()
                    .filter_map(|f| self.get_feature(user_id, f))
                    .filter_map(|fv| fv.value.as_f64())
                    .collect();
                if vals.is_empty() {
                    return Some(serde_json::json!(0.0));
                }
                let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                Some(serde_json::json!(avg))
            }
        }
    }

    /// List all feature definitions.
    pub fn list_definitions(&self) -> Vec<FeatureDefinition> {
        self.definitions.iter().map(|e| e.value().clone()).collect()
    }

    /// Get a feature store health summary.
    pub fn health_summary(&self) -> serde_json::Value {
        let now = Utc::now();
        let total_users = self.user_features.len();
        let total_defs = self.definitions.len();
        let total_computed = self.computed_features.len();

        // Sample staleness across first 100 users
        let mut stale_count = 0u64;
        let mut total_features_checked = 0u64;

        for (i, user_entry) in self.user_features.iter().enumerate() {
            if i >= 100 {
                break;
            }
            for feat_entry in user_entry.value().iter() {
                total_features_checked += 1;
                if let Some(def) = self.definitions.get(feat_entry.key()) {
                    let age = (now - feat_entry.value().updated_at).num_seconds().max(0) as u64;
                    if age > def.ttl_seconds {
                        stale_count += 1;
                    }
                }
            }
        }

        let staleness_rate = if total_features_checked > 0 {
            stale_count as f64 / total_features_checked as f64 * 100.0
        } else {
            0.0
        };

        serde_json::json!({
            "total_users": total_users,
            "total_definitions": total_defs,
            "total_computed": total_computed,
            "sampled_features": total_features_checked,
            "stale_features": stale_count,
            "staleness_rate_percent": staleness_rate,
        })
    }
}

impl Default for FeatureStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_feature_update_and_get() {
        let store = FeatureStore::new();

        store
            .update_feature(
                "user_1",
                "recency_score",
                serde_json::json!(0.85),
                FeatureSource::RealTimeEvent,
            )
            .unwrap();

        let fv = store.get_feature("user_1", "recency_score").unwrap();
        assert_eq!(fv.value, serde_json::json!(0.85));
    }

    #[test]
    fn test_unknown_feature_rejected() {
        let store = FeatureStore::new();
        let result = store.update_feature(
            "user_1",
            "nonexistent_feature",
            serde_json::json!(42),
            FeatureSource::ManualEntry,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_feature_vector_with_staleness() {
        let store = FeatureStore::new();

        // Insert a feature with a very old timestamp to simulate staleness
        let user_map = DashMap::new();
        user_map.insert(
            "recency_score".to_string(),
            FeatureValue {
                feature_name: "recency_score".to_string(),
                value: serde_json::json!(0.5),
                updated_at: Utc::now() - Duration::hours(2), // TTL is 3600s = 1h
                source: FeatureSource::RealTimeEvent,
                version: 1,
            },
        );
        store
            .user_features
            .insert("stale_user".to_string(), user_map);

        let vector = store.get_features("stale_user");
        assert_eq!(vector.features.len(), 1);
        assert!(vector.stale_features.contains(&"recency_score".to_string()));
    }

    #[test]
    fn test_staleness_alerts() {
        let store = FeatureStore::new();

        let user_map = DashMap::new();
        user_map.insert(
            "recency_score".to_string(),
            FeatureValue {
                feature_name: "recency_score".to_string(),
                value: serde_json::json!(0.3),
                updated_at: Utc::now() - Duration::hours(5),
                source: FeatureSource::RealTimeEvent,
                version: 1,
            },
        );
        store
            .user_features
            .insert("alert_user".to_string(), user_map);

        let alerts = store.check_staleness("recency_score");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].user_id, "alert_user");
        // 5 hours > 3 * 1 hour TTL → critical
        assert_eq!(alerts[0].severity, StalenessSeverity::Critical);
    }

    #[test]
    fn test_computed_feature_ratio() {
        let store = FeatureStore::new();

        // Need to define the input features first
        store.define_feature(FeatureDefinition {
            name: "visit_count_30d".to_string(),
            description: "Visit count".to_string(),
            value_type: FeatureValueType::Integer,
            category: FeatureCategory::Behavioral,
            ttl_seconds: 3600,
            source: FeatureSource::RealTimeEvent,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        store
            .update_feature(
                "user_1",
                "purchase_count_30d",
                serde_json::json!(10.0),
                FeatureSource::RealTimeEvent,
            )
            .unwrap();
        store
            .update_feature(
                "user_1",
                "visit_count_30d",
                serde_json::json!(100.0),
                FeatureSource::RealTimeEvent,
            )
            .unwrap();

        let result = store.compute_feature("user_1", "conversion_rate");
        assert!(result.is_some());
        let val = result.unwrap().as_f64().unwrap();
        assert!((val - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_health_summary() {
        let store = FeatureStore::new();

        store
            .update_feature(
                "user_1",
                "recency_score",
                serde_json::json!(0.9),
                FeatureSource::RealTimeEvent,
            )
            .unwrap();

        let health = store.health_summary();
        assert_eq!(health["total_users"], 1);
        assert!(health["total_definitions"].as_u64().unwrap() >= 6);
    }

    #[test]
    fn test_feature_definitions_seeded() {
        let store = FeatureStore::new();
        let defs = store.list_definitions();
        assert!(defs.len() >= 6);
        assert!(defs.iter().any(|d| d.name == "recency_score"));
        assert!(defs.iter().any(|d| d.name == "lifetime_value"));
    }
}
