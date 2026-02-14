//! Multi-tenancy: tenant lifecycle, pricing tiers, quotas, and usage tracking.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// Tenant lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Trial,
    Cancelled,
}

/// SaaS pricing tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingTier {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom,
}

/// Per-tenant configuration limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub max_campaigns: u32,
    pub max_users: u32,
    pub max_offers_per_hour: u64,
    pub max_api_calls_per_day: u64,
    pub features_enabled: Vec<String>,
    pub custom_domain: Option<String>,
    pub data_retention_days: u32,
}

/// Real-time usage counters for a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsage {
    pub campaigns_active: u32,
    pub users_count: u32,
    pub offers_served_today: u64,
    pub api_calls_today: u64,
    pub storage_bytes: u64,
    pub last_reset: DateTime<Utc>,
}

/// A single tenant in the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: TenantStatus,
    pub pricing_tier: PricingTier,
    pub owner_id: Uuid,
    pub settings: TenantSettings,
    pub usage: TenantUsage,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Multi-tenant manager backed by DashMap.
pub struct TenantManager {
    tenants: DashMap<Uuid, Tenant>,
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self {
            tenants: DashMap::new(),
        }
    }

    /// Create a new tenant with tier-appropriate settings.
    pub fn create_tenant(&self, name: String, owner_id: Uuid, tier: PricingTier) -> Tenant {
        let now = Utc::now();
        let slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();

        let settings = Self::tier_limits(&tier);
        let usage = TenantUsage {
            campaigns_active: 0,
            users_count: 1,
            offers_served_today: 0,
            api_calls_today: 0,
            storage_bytes: 0,
            last_reset: now,
        };

        let tenant = Tenant {
            id: Uuid::new_v4(),
            name,
            slug,
            status: TenantStatus::Active,
            pricing_tier: tier,
            owner_id,
            settings,
            usage,
            created_at: now,
            updated_at: now,
        };

        info!(tenant_id = %tenant.id, tenant_name = %tenant.name, "Tenant created");
        self.tenants.insert(tenant.id, tenant.clone());
        tenant
    }

    /// Look up a tenant by id.
    pub fn get_tenant(&self, id: Uuid) -> Option<Tenant> {
        self.tenants.get(&id).map(|e| e.value().clone())
    }

    /// List all tenants.
    pub fn list_tenants(&self) -> Vec<Tenant> {
        self.tenants.iter().map(|e| e.value().clone()).collect()
    }

    /// Change a tenant's pricing tier (updates settings accordingly).
    pub fn update_tier(&self, id: Uuid, tier: PricingTier) -> Option<Tenant> {
        if let Some(mut entry) = self.tenants.get_mut(&id) {
            entry.pricing_tier = tier;
            entry.settings = Self::tier_limits(&tier);
            entry.updated_at = Utc::now();
            info!(tenant_id = %id, new_tier = ?tier, "Tier updated");
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Suspend a tenant.
    pub fn suspend_tenant(&self, id: Uuid) -> Option<Tenant> {
        if let Some(mut entry) = self.tenants.get_mut(&id) {
            entry.status = TenantStatus::Suspended;
            entry.updated_at = Utc::now();
            info!(tenant_id = %id, "Tenant suspended");
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Reactivate a suspended or cancelled tenant.
    pub fn reactivate_tenant(&self, id: Uuid) -> Option<Tenant> {
        if let Some(mut entry) = self.tenants.get_mut(&id) {
            entry.status = TenantStatus::Active;
            entry.updated_at = Utc::now();
            info!(tenant_id = %id, "Tenant reactivated");
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Reset daily usage counters for a tenant.
    pub fn reset_daily_usage(&self, id: Uuid) -> Option<()> {
        if let Some(mut entry) = self.tenants.get_mut(&id) {
            entry.usage.offers_served_today = 0;
            entry.usage.api_calls_today = 0;
            entry.usage.last_reset = Utc::now();
            entry.updated_at = Utc::now();
            info!(tenant_id = %id, "Daily usage counters reset");
            Some(())
        } else {
            None
        }
    }

    /// Set a custom domain for a tenant.
    pub fn set_custom_domain(&self, id: Uuid, domain: Option<String>) -> Option<()> {
        if let Some(mut entry) = self.tenants.get_mut(&id) {
            entry.settings.custom_domain = domain;
            entry.updated_at = Utc::now();
            Some(())
        } else {
            None
        }
    }

    /// Check whether a tenant is within its quota for the given resource.
    pub fn check_quota(&self, tenant_id: Uuid, resource: &str) -> Result<bool, anyhow::Error> {
        let tenant = self
            .tenants
            .get(&tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {tenant_id}"))?;

        let within = match resource {
            "campaigns" => tenant.usage.campaigns_active < tenant.settings.max_campaigns,
            "offers" => tenant.usage.offers_served_today < tenant.settings.max_offers_per_hour,
            "api_calls" => tenant.usage.api_calls_today < tenant.settings.max_api_calls_per_day,
            "users" => tenant.usage.users_count < tenant.settings.max_users,
            _ => true,
        };
        Ok(within)
    }

    /// Increment a usage counter for the given resource.
    pub fn increment_usage(&self, tenant_id: Uuid, resource: &str, amount: u64) {
        if let Some(mut entry) = self.tenants.get_mut(&tenant_id) {
            match resource {
                "campaigns" => entry.usage.campaigns_active += amount as u32,
                "offers" => entry.usage.offers_served_today += amount,
                "api_calls" => entry.usage.api_calls_today += amount,
                "users" => entry.usage.users_count += amount as u32,
                "storage" => entry.usage.storage_bytes += amount,
                _ => {}
            }
        }
    }

    /// Return the default settings for a given pricing tier.
    pub fn tier_limits(tier: &PricingTier) -> TenantSettings {
        match tier {
            PricingTier::Free => TenantSettings {
                max_campaigns: 5,
                max_users: 2,
                max_offers_per_hour: 1_000,
                max_api_calls_per_day: 10_000,
                features_enabled: vec!["basic_targeting".into()],
                custom_domain: None,
                data_retention_days: 30,
            },
            PricingTier::Starter => TenantSettings {
                max_campaigns: 25,
                max_users: 10,
                max_offers_per_hour: 100_000,
                max_api_calls_per_day: 100_000,
                features_enabled: vec![
                    "basic_targeting".into(),
                    "ab_testing".into(),
                    "analytics".into(),
                ],
                custom_domain: None,
                data_retention_days: 90,
            },
            PricingTier::Professional => TenantSettings {
                max_campaigns: 100,
                max_users: 50,
                max_offers_per_hour: 5_000_000,
                max_api_calls_per_day: 1_000_000,
                features_enabled: vec![
                    "basic_targeting".into(),
                    "ab_testing".into(),
                    "analytics".into(),
                    "journey_builder".into(),
                    "dco".into(),
                ],
                custom_domain: None,
                data_retention_days: 365,
            },
            PricingTier::Enterprise | PricingTier::Custom => TenantSettings {
                max_campaigns: u32::MAX,
                max_users: u32::MAX,
                max_offers_per_hour: u64::MAX,
                max_api_calls_per_day: u64::MAX,
                features_enabled: vec![
                    "basic_targeting".into(),
                    "ab_testing".into(),
                    "analytics".into(),
                    "journey_builder".into(),
                    "dco".into(),
                    "cdp".into(),
                    "loyalty".into(),
                    "custom_models".into(),
                ],
                custom_domain: None,
                data_retention_days: 730,
            },
        }
    }

    /// Seed three demo tenants in different pricing tiers.
    pub fn seed_demo_tenants(&self) {
        let owner = Uuid::new_v4();
        self.create_tenant("Acme Corp".into(), owner, PricingTier::Enterprise);
        self.create_tenant("Startup Inc".into(), Uuid::new_v4(), PricingTier::Starter);
        self.create_tenant("Hobby Shop".into(), Uuid::new_v4(), PricingTier::Free);
        info!("Demo tenants seeded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tenant() {
        let mgr = TenantManager::new();
        let owner = Uuid::new_v4();
        let tenant = mgr.create_tenant("My Company".into(), owner, PricingTier::Professional);

        assert_eq!(tenant.name, "My Company");
        assert_eq!(tenant.slug, "my-company");
        assert_eq!(tenant.status, TenantStatus::Active);
        assert_eq!(tenant.pricing_tier, PricingTier::Professional);
        assert_eq!(tenant.settings.max_campaigns, 100);

        let fetched = mgr.get_tenant(tenant.id).unwrap();
        assert_eq!(fetched.id, tenant.id);
    }

    #[test]
    fn test_quota_check() {
        let mgr = TenantManager::new();
        let owner = Uuid::new_v4();
        let tenant = mgr.create_tenant("Free Org".into(), owner, PricingTier::Free);

        // Initially within quota.
        assert!(mgr.check_quota(tenant.id, "campaigns").unwrap());

        // Use up campaigns (limit is 5).
        mgr.increment_usage(tenant.id, "campaigns", 5);
        assert!(!mgr.check_quota(tenant.id, "campaigns").unwrap());

        // Unknown resource always returns true.
        assert!(mgr.check_quota(tenant.id, "widgets").unwrap());
    }
}
