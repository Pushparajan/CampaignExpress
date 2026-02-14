//! Tenant lifecycle operations — suspend, reactivate, offboard, usage reset,
//! tier migration, and domain configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use campaign_platform::tenancy::{PricingTier, Tenant, TenantManager, TenantStatus};

/// Reason for a lifecycle action — stored in audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReason {
    pub actor_id: Uuid,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

impl ActionReason {
    pub fn new(actor_id: Uuid, reason: impl Into<String>) -> Self {
        Self {
            actor_id,
            reason: reason.into(),
            timestamp: Utc::now(),
        }
    }
}

/// Result of a tenant operation with before/after state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantOpResult {
    pub tenant_id: Uuid,
    pub action: String,
    pub previous_status: TenantStatus,
    pub new_status: TenantStatus,
    pub previous_tier: PricingTier,
    pub new_tier: PricingTier,
    pub performed_at: DateTime<Utc>,
    pub reason: ActionReason,
}

/// Tenant lifecycle manager composing the platform TenantManager.
pub struct TenantOps<'a> {
    manager: &'a TenantManager,
}

impl<'a> TenantOps<'a> {
    pub fn new(manager: &'a TenantManager) -> Self {
        Self { manager }
    }

    /// Reactivate a suspended or cancelled tenant.
    pub fn reactivate(
        &self,
        tenant_id: Uuid,
        reason: ActionReason,
    ) -> anyhow::Result<TenantOpResult> {
        let tenant = self
            .manager
            .get_tenant(tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {tenant_id}"))?;

        let previous_status = tenant.status;
        let tier = tenant.pricing_tier;

        // Only suspended or cancelled tenants can be reactivated
        if tenant.status != TenantStatus::Suspended && tenant.status != TenantStatus::Cancelled {
            return Err(anyhow::anyhow!(
                "Tenant is {:?}, cannot reactivate",
                tenant.status
            ));
        }

        self.manager.reactivate_tenant(tenant_id);

        Ok(TenantOpResult {
            tenant_id,
            action: "reactivate".into(),
            previous_status,
            new_status: TenantStatus::Active,
            previous_tier: tier,
            new_tier: tier,
            performed_at: Utc::now(),
            reason,
        })
    }

    /// Suspend a tenant (disable access but retain data).
    pub fn suspend(&self, tenant_id: Uuid, reason: ActionReason) -> anyhow::Result<TenantOpResult> {
        let tenant = self
            .manager
            .get_tenant(tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {tenant_id}"))?;

        let previous_status = tenant.status;
        let tier = tenant.pricing_tier;

        if tenant.status == TenantStatus::Suspended {
            return Err(anyhow::anyhow!("Tenant is already suspended"));
        }

        self.manager.suspend_tenant(tenant_id);

        Ok(TenantOpResult {
            tenant_id,
            action: "suspend".into(),
            previous_status,
            new_status: TenantStatus::Suspended,
            previous_tier: tier,
            new_tier: tier,
            performed_at: Utc::now(),
            reason,
        })
    }

    /// Change a tenant's pricing tier.
    pub fn change_tier(
        &self,
        tenant_id: Uuid,
        new_tier: PricingTier,
        reason: ActionReason,
    ) -> anyhow::Result<TenantOpResult> {
        let tenant = self
            .manager
            .get_tenant(tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {tenant_id}"))?;

        let previous_status = tenant.status;
        let previous_tier = tenant.pricing_tier;

        if previous_tier == new_tier {
            return Err(anyhow::anyhow!("Tenant is already on {:?} tier", new_tier));
        }

        self.manager.update_tier(tenant_id, new_tier);

        Ok(TenantOpResult {
            tenant_id,
            action: "change_tier".into(),
            previous_status,
            new_status: previous_status,
            previous_tier,
            new_tier,
            performed_at: Utc::now(),
            reason,
        })
    }

    /// Reset daily usage counters for a tenant.
    pub fn reset_usage(&self, tenant_id: Uuid) -> anyhow::Result<()> {
        let _tenant = self
            .manager
            .get_tenant(tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {tenant_id}"))?;
        self.manager.reset_daily_usage(tenant_id);
        Ok(())
    }

    /// Set a custom domain for a tenant.
    pub fn set_custom_domain(&self, tenant_id: Uuid, domain: Option<String>) -> anyhow::Result<()> {
        self.manager
            .set_custom_domain(tenant_id, domain)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {tenant_id}"))?;
        Ok(())
    }

    /// Get all tenants with their current status, sorted by name.
    pub fn list_all(&self) -> Vec<Tenant> {
        let mut tenants = self.manager.list_tenants();
        tenants.sort_by(|a, b| a.name.cmp(&b.name));
        tenants
    }

    /// Get tenants filtered by status.
    pub fn list_by_status(&self, status: TenantStatus) -> Vec<Tenant> {
        let mut tenants: Vec<_> = self
            .manager
            .list_tenants()
            .into_iter()
            .filter(|t| t.status == status)
            .collect();
        tenants.sort_by(|a, b| a.name.cmp(&b.name));
        tenants
    }

    /// Get tenants that are approaching or exceeding their quotas.
    pub fn tenants_near_quota(&self, threshold_pct: f64) -> Vec<TenantQuotaAlert> {
        if !(0.0..=100.0).contains(&threshold_pct) {
            return Vec::new();
        }
        self.manager
            .list_tenants()
            .into_iter()
            .filter_map(|t| {
                let mut alerts = Vec::new();

                let offers_pct = if t.settings.max_offers_per_hour > 0 {
                    t.usage.offers_served_today as f64 / t.settings.max_offers_per_hour as f64
                        * 100.0
                } else {
                    0.0
                };
                if offers_pct >= threshold_pct {
                    alerts.push(("offers".into(), offers_pct));
                }

                let api_pct = if t.settings.max_api_calls_per_day > 0 {
                    t.usage.api_calls_today as f64 / t.settings.max_api_calls_per_day as f64 * 100.0
                } else {
                    0.0
                };
                if api_pct >= threshold_pct {
                    alerts.push(("api_calls".into(), api_pct));
                }

                let campaigns_pct = if t.settings.max_campaigns > 0 {
                    t.usage.campaigns_active as f64 / t.settings.max_campaigns as f64 * 100.0
                } else {
                    0.0
                };
                if campaigns_pct >= threshold_pct {
                    alerts.push(("campaigns".into(), campaigns_pct));
                }

                if alerts.is_empty() {
                    None
                } else {
                    Some(TenantQuotaAlert {
                        tenant_id: t.id,
                        tenant_name: t.name.clone(),
                        tier: t.pricing_tier,
                        alerts,
                    })
                }
            })
            .collect()
    }
}

/// A tenant approaching or exceeding quota limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotaAlert {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub tier: PricingTier,
    pub alerts: Vec<(String, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TenantManager {
        let mgr = TenantManager::new();
        mgr.seed_demo_tenants();
        mgr
    }

    #[test]
    fn test_suspend_and_reactivate() {
        let mgr = setup();
        let ops = TenantOps::new(&mgr);
        let tenants = ops.list_all();
        let tenant = &tenants[0];

        let reason = ActionReason::new(Uuid::new_v4(), "Non-payment");
        let result = ops.suspend(tenant.id, reason).unwrap();
        assert_eq!(result.new_status, TenantStatus::Suspended);
        assert_eq!(result.previous_status, TenantStatus::Active);

        let reason = ActionReason::new(Uuid::new_v4(), "Payment received");
        let result = ops.reactivate(tenant.id, reason).unwrap();
        assert_eq!(result.new_status, TenantStatus::Active);
    }

    #[test]
    fn test_double_suspend_fails() {
        let mgr = setup();
        let ops = TenantOps::new(&mgr);
        let tenants = ops.list_all();
        let tenant = &tenants[0];

        let reason = ActionReason::new(Uuid::new_v4(), "Test");
        ops.suspend(tenant.id, reason.clone()).unwrap();
        assert!(ops.suspend(tenant.id, reason).is_err());
    }

    #[test]
    fn test_change_tier() {
        let mgr = setup();
        let ops = TenantOps::new(&mgr);
        let tenants = ops.list_all();
        // Find the Starter tenant
        let starter = tenants
            .iter()
            .find(|t| t.pricing_tier == PricingTier::Starter)
            .unwrap();

        let reason = ActionReason::new(Uuid::new_v4(), "Upgrade");
        let result = ops
            .change_tier(starter.id, PricingTier::Professional, reason)
            .unwrap();
        assert_eq!(result.previous_tier, PricingTier::Starter);
        assert_eq!(result.new_tier, PricingTier::Professional);

        let updated = mgr.get_tenant(starter.id).unwrap();
        assert_eq!(updated.pricing_tier, PricingTier::Professional);
    }

    #[test]
    fn test_list_by_status() {
        let mgr = setup();
        let ops = TenantOps::new(&mgr);
        let active = ops.list_by_status(TenantStatus::Active);
        assert_eq!(active.len(), 3);
        let suspended = ops.list_by_status(TenantStatus::Suspended);
        assert!(suspended.is_empty());
    }

    #[test]
    fn test_quota_alerts() {
        let mgr = TenantManager::new();
        let owner = Uuid::new_v4();
        let tenant = mgr.create_tenant("Test Corp".into(), owner, PricingTier::Free);
        // Free tier: 1000 max_offers_per_hour
        mgr.increment_usage(tenant.id, "offers", 900);

        let ops = TenantOps::new(&mgr);
        let alerts = ops.tenants_near_quota(80.0);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].tenant_name, "Test Corp");
    }
}
