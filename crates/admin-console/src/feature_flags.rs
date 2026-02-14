//! Feature flag management — per-tenant and global feature toggles for
//! gradual rollout, A/B testing, and module gating.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// Rollout strategy for a feature flag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStrategy {
    /// Enabled for all tenants.
    Global,
    /// Enabled only for specific tenant IDs.
    AllowList,
    /// Percentage-based gradual rollout.
    Percentage,
    /// Disabled for everyone.
    Disabled,
}

/// A feature flag definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub key: String,
    pub description: String,
    pub strategy: RolloutStrategy,
    pub percentage: Option<u8>,
    pub allowed_tenants: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Feature flag evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagEvaluation {
    pub flag_key: String,
    pub enabled: bool,
    pub strategy: RolloutStrategy,
    pub reason: String,
}

/// Feature flag manager.
pub struct FeatureFlagManager {
    flags: DashMap<String, FeatureFlag>,
}

impl Default for FeatureFlagManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureFlagManager {
    pub fn new() -> Self {
        Self {
            flags: DashMap::new(),
        }
    }

    /// Create a new feature flag.
    pub fn create_flag(
        &self,
        key: impl Into<String>,
        description: impl Into<String>,
        strategy: RolloutStrategy,
    ) -> FeatureFlag {
        let key = key.into();
        let now = Utc::now();
        let flag = FeatureFlag {
            id: Uuid::new_v4(),
            key: key.clone(),
            description: description.into(),
            strategy,
            percentage: None,
            allowed_tenants: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        info!(flag_key = %key, "Feature flag created");
        self.flags.insert(key, flag.clone());
        flag
    }

    /// Update a flag's rollout strategy.
    pub fn update_strategy(
        &self,
        key: &str,
        strategy: RolloutStrategy,
        percentage: Option<u8>,
    ) -> anyhow::Result<FeatureFlag> {
        let mut entry = self
            .flags
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("Flag not found: {key}"))?;
        entry.strategy = strategy;
        entry.percentage = percentage;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Add a tenant to the allow-list for a flag.
    pub fn add_tenant(&self, key: &str, tenant_id: Uuid) -> anyhow::Result<()> {
        let mut entry = self
            .flags
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("Flag not found: {key}"))?;
        if !entry.allowed_tenants.contains(&tenant_id) {
            entry.allowed_tenants.push(tenant_id);
            entry.updated_at = Utc::now();
        }
        Ok(())
    }

    /// Remove a tenant from the allow-list for a flag.
    pub fn remove_tenant(&self, key: &str, tenant_id: Uuid) -> anyhow::Result<()> {
        let mut entry = self
            .flags
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("Flag not found: {key}"))?;
        entry.allowed_tenants.retain(|id| *id != tenant_id);
        entry.updated_at = Utc::now();
        Ok(())
    }

    /// Evaluate whether a feature is enabled for a specific tenant.
    pub fn evaluate(&self, key: &str, tenant_id: Uuid) -> FlagEvaluation {
        let Some(flag) = self.flags.get(key) else {
            return FlagEvaluation {
                flag_key: key.to_string(),
                enabled: false,
                strategy: RolloutStrategy::Disabled,
                reason: "Flag not found".into(),
            };
        };

        let (enabled, reason) = match &flag.strategy {
            RolloutStrategy::Global => (true, "Global rollout".into()),
            RolloutStrategy::Disabled => (false, "Flag disabled".into()),
            RolloutStrategy::AllowList => {
                if flag.allowed_tenants.contains(&tenant_id) {
                    (true, "Tenant in allow-list".into())
                } else {
                    (false, "Tenant not in allow-list".into())
                }
            }
            RolloutStrategy::Percentage => {
                let pct = flag.percentage.unwrap_or(0);
                // Deterministic hash from tenant_id to get stable percentage bucket
                let hash = tenant_id.as_u128() % 100;
                if hash < pct as u128 {
                    (true, format!("Tenant in {pct}% rollout"))
                } else {
                    (false, format!("Tenant outside {pct}% rollout"))
                }
            }
        };

        FlagEvaluation {
            flag_key: key.to_string(),
            enabled,
            strategy: flag.strategy.clone(),
            reason,
        }
    }

    /// List all feature flags.
    pub fn list_flags(&self) -> Vec<FeatureFlag> {
        let mut flags: Vec<_> = self.flags.iter().map(|e| e.value().clone()).collect();
        flags.sort_by(|a, b| a.key.cmp(&b.key));
        flags
    }

    /// Delete a feature flag.
    pub fn delete_flag(&self, key: &str) -> bool {
        self.flags.remove(key).is_some()
    }

    /// Seed common feature flags for a SaaS platform.
    pub fn seed_defaults(&self) {
        self.create_flag(
            "npu_acceleration",
            "Enable AMD XDNA NPU hardware acceleration",
            RolloutStrategy::AllowList,
        );

        let mut beta = self.create_flag(
            "journey_builder_v2",
            "New journey builder UI with drag-and-drop",
            RolloutStrategy::Percentage,
        );
        beta.percentage = Some(25);
        self.flags.insert(beta.key.clone(), beta);

        self.create_flag(
            "realtime_analytics",
            "Real-time analytics dashboard with live updates",
            RolloutStrategy::Global,
        );

        self.create_flag(
            "advanced_segmentation",
            "ML-powered audience segmentation",
            RolloutStrategy::AllowList,
        );

        self.create_flag(
            "webhook_v2",
            "Next-gen webhook system with retry and DLQ",
            RolloutStrategy::Disabled,
        );

        info!("Default feature flags seeded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_flag() {
        let mgr = FeatureFlagManager::new();
        mgr.create_flag("test_global", "Global flag", RolloutStrategy::Global);

        let eval = mgr.evaluate("test_global", Uuid::new_v4());
        assert!(eval.enabled);
        assert_eq!(eval.strategy, RolloutStrategy::Global);
    }

    #[test]
    fn test_disabled_flag() {
        let mgr = FeatureFlagManager::new();
        mgr.create_flag("disabled_feat", "Disabled", RolloutStrategy::Disabled);

        let eval = mgr.evaluate("disabled_feat", Uuid::new_v4());
        assert!(!eval.enabled);
    }

    #[test]
    fn test_allowlist_flag() {
        let mgr = FeatureFlagManager::new();
        mgr.create_flag("beta", "Beta feature", RolloutStrategy::AllowList);

        let allowed = Uuid::new_v4();
        let denied = Uuid::new_v4();
        mgr.add_tenant("beta", allowed).unwrap();

        assert!(mgr.evaluate("beta", allowed).enabled);
        assert!(!mgr.evaluate("beta", denied).enabled);

        mgr.remove_tenant("beta", allowed).unwrap();
        assert!(!mgr.evaluate("beta", allowed).enabled);
    }

    #[test]
    fn test_percentage_flag() {
        let mgr = FeatureFlagManager::new();
        mgr.create_flag("gradual", "Gradual rollout", RolloutStrategy::Percentage);
        mgr.update_strategy("gradual", RolloutStrategy::Percentage, Some(50))
            .unwrap();

        // Evaluate many tenants and check roughly 50% are enabled
        let enabled_count = (0..100)
            .map(|_| mgr.evaluate("gradual", Uuid::new_v4()))
            .filter(|e| e.enabled)
            .count();

        // Should be roughly 50%, allow wide margin for random UUIDs
        assert!(
            enabled_count > 20,
            "Expected >20% enabled, got {enabled_count}"
        );
        assert!(
            enabled_count < 80,
            "Expected <80% enabled, got {enabled_count}"
        );
    }

    #[test]
    fn test_unknown_flag() {
        let mgr = FeatureFlagManager::new();
        let eval = mgr.evaluate("nonexistent", Uuid::new_v4());
        assert!(!eval.enabled);
    }

    #[test]
    fn test_list_and_delete() {
        let mgr = FeatureFlagManager::new();
        mgr.create_flag("a", "First", RolloutStrategy::Global);
        mgr.create_flag("b", "Second", RolloutStrategy::Disabled);

        assert_eq!(mgr.list_flags().len(), 2);
        assert!(mgr.delete_flag("a"));
        assert_eq!(mgr.list_flags().len(), 1);
        assert!(!mgr.delete_flag("a")); // already deleted
    }
}
