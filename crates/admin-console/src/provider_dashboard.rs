//! Provider dashboard — cross-tenant SaaS overview for the platform operator.
//! Aggregates tenants, billing, licensing, and ops data into a single view.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use campaign_billing::billing::BillingEngine;
use campaign_licensing::dashboard::DashboardEngine;
use campaign_platform::tenancy::{PricingTier, TenantManager, TenantStatus};

/// Top-level provider dashboard snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOverview {
    pub total_tenants: u64,
    pub active_tenants: u64,
    pub suspended_tenants: u64,
    pub trial_tenants: u64,
    pub cancelled_tenants: u64,
    pub tenants_by_tier: TierBreakdown,
    pub billing_summary: BillingSummary,
    pub usage_summary: UsageSummary,
    pub generated_at: DateTime<Utc>,
}

/// Tenant count by pricing tier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TierBreakdown {
    pub free: u64,
    pub starter: u64,
    pub professional: u64,
    pub enterprise: u64,
    pub custom: u64,
}

/// Aggregate billing summary across all tenants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingSummary {
    pub total_mrr: f64,
    pub total_arr: f64,
    pub active_subscriptions: u64,
    pub open_invoices: u64,
    pub open_invoice_amount: f64,
    pub paid_invoices: u64,
    pub total_revenue: f64,
}

/// Aggregate usage across all tenants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_offers_today: u64,
    pub total_api_calls_today: u64,
    pub total_campaigns_active: u64,
    pub total_users: u64,
    pub total_storage_bytes: u64,
}

/// Per-tenant row for the admin table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRow {
    pub tenant_id: Uuid,
    pub name: String,
    pub tier: PricingTier,
    pub status: TenantStatus,
    pub users: u32,
    pub campaigns: u32,
    pub offers_today: u64,
    pub api_calls_today: u64,
    pub mrr: f64,
    pub created_at: DateTime<Utc>,
}

/// Provider dashboard builder composing multiple subsystems.
pub struct ProviderDashboard<'a> {
    tenants: &'a TenantManager,
    billing: &'a BillingEngine,
    licensing: Option<&'a DashboardEngine>,
}

impl<'a> ProviderDashboard<'a> {
    pub fn new(tenants: &'a TenantManager, billing: &'a BillingEngine) -> Self {
        Self {
            tenants,
            billing,
            licensing: None,
        }
    }

    /// Optionally attach the licensing dashboard for installation data.
    pub fn with_licensing(mut self, dashboard: &'a DashboardEngine) -> Self {
        self.licensing = Some(dashboard);
        self
    }

    /// Build the full provider overview.
    pub fn overview(&self) -> ProviderOverview {
        let all_tenants = self.tenants.list_tenants();
        let total = all_tenants.len() as u64;

        let mut active = 0u64;
        let mut suspended = 0u64;
        let mut trial = 0u64;
        let mut cancelled = 0u64;
        let mut tiers = TierBreakdown::default();
        let mut usage = UsageSummary::default();

        for t in &all_tenants {
            match t.status {
                TenantStatus::Active => active += 1,
                TenantStatus::Suspended => suspended += 1,
                TenantStatus::Trial => trial += 1,
                TenantStatus::Cancelled => cancelled += 1,
            }
            match t.pricing_tier {
                PricingTier::Free => tiers.free += 1,
                PricingTier::Starter => tiers.starter += 1,
                PricingTier::Professional => tiers.professional += 1,
                PricingTier::Enterprise => tiers.enterprise += 1,
                PricingTier::Custom => tiers.custom += 1,
            }
            usage.total_offers_today += t.usage.offers_served_today;
            usage.total_api_calls_today += t.usage.api_calls_today;
            usage.total_campaigns_active += t.usage.campaigns_active as u64;
            usage.total_users += t.usage.users_count as u64;
            usage.total_storage_bytes += t.usage.storage_bytes;
        }

        // Billing aggregation
        let plans = self.billing.list_plans();
        let mut billing_summary = BillingSummary::default();

        for t in &all_tenants {
            if let Some(sub) = self.billing.get_subscription(t.id) {
                billing_summary.active_subscriptions += 1;
                if let Some(plan) = plans.iter().find(|p| p.id == sub.plan_id) {
                    billing_summary.total_mrr += plan.monthly_price;
                }

                let invoices = self.billing.list_invoices(t.id);
                for inv in &invoices {
                    if inv.paid_at.is_some() {
                        billing_summary.paid_invoices += 1;
                        billing_summary.total_revenue += inv.amount;
                    } else {
                        billing_summary.open_invoices += 1;
                        billing_summary.open_invoice_amount += inv.amount;
                    }
                }
            }
        }
        billing_summary.total_arr = billing_summary.total_mrr * 12.0;

        ProviderOverview {
            total_tenants: total,
            active_tenants: active,
            suspended_tenants: suspended,
            trial_tenants: trial,
            cancelled_tenants: cancelled,
            tenants_by_tier: tiers,
            billing_summary,
            usage_summary: usage,
            generated_at: Utc::now(),
        }
    }

    /// Get a per-tenant table for the admin console.
    pub fn tenant_table(&self) -> Vec<TenantRow> {
        let all_tenants = self.tenants.list_tenants();
        let plans = self.billing.list_plans();

        let mut rows: Vec<TenantRow> = all_tenants
            .iter()
            .map(|t| {
                let mrr = self
                    .billing
                    .get_subscription(t.id)
                    .and_then(|sub| plans.iter().find(|p| p.id == sub.plan_id))
                    .map(|p| p.monthly_price)
                    .unwrap_or(0.0);

                TenantRow {
                    tenant_id: t.id,
                    name: t.name.clone(),
                    tier: t.pricing_tier,
                    status: t.status,
                    users: t.usage.users_count,
                    campaigns: t.usage.campaigns_active,
                    offers_today: t.usage.offers_served_today,
                    api_calls_today: t.usage.api_calls_today,
                    mrr,
                    created_at: t.created_at,
                }
            })
            .collect();

        rows.sort_by(|a, b| a.name.cmp(&b.name));
        rows
    }

    /// Get the licensing fleet view if licensing is attached.
    pub fn fleet_overview(&self) -> Option<campaign_licensing::dashboard::FleetOverview> {
        self.licensing.map(|d| d.fleet_overview())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_overview() {
        let tenants = TenantManager::new();
        tenants.seed_demo_tenants();
        let billing = BillingEngine::new();
        billing.seed_demo_data();

        let dashboard = ProviderDashboard::new(&tenants, &billing);
        let overview = dashboard.overview();

        assert_eq!(overview.total_tenants, 3);
        assert_eq!(overview.active_tenants, 3);
        assert!(overview.tenants_by_tier.enterprise >= 1);
        assert!(overview.tenants_by_tier.starter >= 1);
        assert!(overview.tenants_by_tier.free >= 1);
    }

    #[test]
    fn test_tenant_table() {
        let tenants = TenantManager::new();
        tenants.seed_demo_tenants();
        let billing = BillingEngine::new();

        let dashboard = ProviderDashboard::new(&tenants, &billing);
        let rows = dashboard.tenant_table();

        assert_eq!(rows.len(), 3);
        // Sorted by name
        assert!(rows[0].name <= rows[1].name);
    }
}
