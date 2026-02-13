//! Billing engine — subscription management, invoice generation, payment methods,
//! and pricing plan CRUD. Backed by DashMap for development; swap to Stripe /
//! Chargebee integration for production.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// External billing provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingProvider {
    Stripe,
    Chargebee,
    Manual,
}

/// Subscription lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Trialing,
    Paused,
}

/// A pricing plan available for subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingPlan {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub monthly_price: f64,
    pub annual_price: f64,
    pub included_offers: u64,
    pub included_api_calls: u64,
    pub features: Vec<String>,
}

/// A tenant's subscription to a pricing plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: Uuid,
    pub provider: BillingProvider,
    pub external_subscription_id: Option<String>,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub created_at: DateTime<Utc>,
}

/// An invoice issued to a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subscription_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub issued_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

/// A single line item on an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: u64,
    pub unit_price: f64,
    pub amount: f64,
}

/// A stored payment method for a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub provider: BillingProvider,
    pub method_type: String,
    pub last_four: Option<String>,
    pub expiry: Option<String>,
    pub is_default: bool,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// In-memory billing engine backed by `DashMap`.
pub struct BillingEngine {
    plans: Arc<DashMap<Uuid, PricingPlan>>,
    subscriptions: Arc<DashMap<Uuid, Subscription>>,
    invoices: Arc<DashMap<Uuid, Invoice>>,
    payment_methods: Arc<DashMap<Uuid, PaymentMethod>>,
}

impl Default for BillingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BillingEngine {
    /// Create a new empty billing engine.
    pub fn new() -> Self {
        info!("BillingEngine initialized");
        Self {
            plans: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            invoices: Arc::new(DashMap::new()),
            payment_methods: Arc::new(DashMap::new()),
        }
    }

    /// Create a new pricing plan and store it.
    #[allow(clippy::too_many_arguments)]
    pub fn create_plan(
        &self,
        name: String,
        tier: String,
        monthly: f64,
        annual: f64,
        offers: u64,
        api_calls: u64,
        features: Vec<String>,
    ) -> PricingPlan {
        let plan = PricingPlan {
            id: Uuid::new_v4(),
            name,
            tier,
            monthly_price: monthly,
            annual_price: annual,
            included_offers: offers,
            included_api_calls: api_calls,
            features,
        };
        self.plans.insert(plan.id, plan.clone());
        plan
    }

    /// Subscribe a tenant to the given plan.
    pub fn subscribe(
        &self,
        tenant_id: Uuid,
        plan_id: Uuid,
        provider: BillingProvider,
    ) -> Subscription {
        let now = Utc::now();
        let sub = Subscription {
            id: Uuid::new_v4(),
            tenant_id,
            plan_id,
            provider,
            external_subscription_id: None,
            status: SubscriptionStatus::Active,
            current_period_start: now,
            current_period_end: now + Duration::days(30),
            cancel_at_period_end: false,
            created_at: now,
        };
        self.subscriptions.insert(sub.id, sub.clone());
        sub
    }

    /// Mark a subscription to cancel at the end of the current period.
    pub fn cancel_subscription(&self, subscription_id: Uuid) -> Option<Subscription> {
        self.subscriptions.get_mut(&subscription_id).map(|mut sub| {
            sub.cancel_at_period_end = true;
            sub.clone()
        })
    }

    /// Generate an invoice for a tenant/subscription from line items.
    pub fn generate_invoice(
        &self,
        tenant_id: Uuid,
        subscription_id: Uuid,
        line_items: Vec<InvoiceLineItem>,
    ) -> Invoice {
        let amount: f64 = line_items.iter().map(|li| li.amount).sum();
        let now = Utc::now();
        let invoice = Invoice {
            id: Uuid::new_v4(),
            tenant_id,
            subscription_id,
            amount,
            currency: "USD".into(),
            status: "open".into(),
            line_items,
            issued_at: now,
            due_at: now + Duration::days(30),
            paid_at: None,
        };
        self.invoices.insert(invoice.id, invoice.clone());
        invoice
    }

    /// List all invoices for a tenant.
    pub fn list_invoices(&self, tenant_id: Uuid) -> Vec<Invoice> {
        self.invoices
            .iter()
            .filter(|e| e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Add a payment method for a tenant.
    pub fn add_payment_method(
        &self,
        tenant_id: Uuid,
        provider: BillingProvider,
        method_type: String,
        last_four: Option<String>,
        expiry: Option<String>,
    ) -> PaymentMethod {
        let pm = PaymentMethod {
            id: Uuid::new_v4(),
            tenant_id,
            provider,
            method_type,
            last_four,
            expiry,
            is_default: true,
        };
        self.payment_methods.insert(pm.id, pm.clone());
        pm
    }

    /// List all available pricing plans.
    pub fn list_plans(&self) -> Vec<PricingPlan> {
        self.plans.iter().map(|e| e.value().clone()).collect()
    }

    /// Get the first active subscription for a tenant (if any).
    pub fn get_subscription(&self, tenant_id: Uuid) -> Option<Subscription> {
        self.subscriptions
            .iter()
            .find(|e| e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
    }

    /// Seed demo data: 4 plans, 2 subscriptions, and sample invoices.
    pub fn seed_demo_data(&self) {
        let tenant_a = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let tenant_b = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        // Plans
        let _free = self.create_plan(
            "Free".into(),
            "free".into(),
            0.0,
            0.0,
            10_000,
            5_000,
            vec!["basic_analytics".into(), "single_campaign".into()],
        );

        let starter = self.create_plan(
            "Starter".into(),
            "starter".into(),
            99.0,
            990.0,
            500_000,
            100_000,
            vec![
                "advanced_analytics".into(),
                "5_campaigns".into(),
                "email_support".into(),
            ],
        );

        let pro = self.create_plan(
            "Pro".into(),
            "pro".into(),
            499.0,
            4990.0,
            5_000_000,
            1_000_000,
            vec![
                "full_analytics".into(),
                "unlimited_campaigns".into(),
                "priority_support".into(),
                "dco".into(),
                "journey_builder".into(),
            ],
        );

        let _enterprise = self.create_plan(
            "Enterprise".into(),
            "enterprise".into(),
            1999.0,
            19990.0,
            50_000_000,
            10_000_000,
            vec![
                "full_analytics".into(),
                "unlimited_campaigns".into(),
                "dedicated_support".into(),
                "sla_99_99".into(),
                "custom_integrations".into(),
                "npu_acceleration".into(),
            ],
        );

        // Subscriptions
        let sub_a = self.subscribe(tenant_a, pro.id, BillingProvider::Stripe);
        let sub_b = self.subscribe(tenant_b, starter.id, BillingProvider::Stripe);

        // Invoices
        self.generate_invoice(
            tenant_a,
            sub_a.id,
            vec![
                InvoiceLineItem {
                    description: "Pro Plan - Monthly".into(),
                    quantity: 1,
                    unit_price: 499.0,
                    amount: 499.0,
                },
                InvoiceLineItem {
                    description: "Overage: 250K offers".into(),
                    quantity: 250_000,
                    unit_price: 0.00001,
                    amount: 2.50,
                },
            ],
        );

        self.generate_invoice(
            tenant_b,
            sub_b.id,
            vec![InvoiceLineItem {
                description: "Starter Plan - Monthly".into(),
                quantity: 1,
                unit_price: 99.0,
                amount: 99.0,
            }],
        );

        // Payment methods
        self.add_payment_method(
            tenant_a,
            BillingProvider::Stripe,
            "card".into(),
            Some("4242".into()),
            Some("12/27".into()),
        );
        self.add_payment_method(
            tenant_b,
            BillingProvider::Stripe,
            "card".into(),
            Some("1234".into()),
            Some("06/28".into()),
        );

        info!("Seeded demo billing data: 4 plans, 2 subscriptions, 2 invoices");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_to_plan() {
        let engine = BillingEngine::new();
        let plan = engine.create_plan(
            "Test Plan".into(),
            "test".into(),
            49.0,
            490.0,
            100_000,
            50_000,
            vec!["feature_a".into()],
        );

        let tenant = Uuid::new_v4();
        let sub = engine.subscribe(tenant, plan.id, BillingProvider::Stripe);

        assert_eq!(sub.tenant_id, tenant);
        assert_eq!(sub.plan_id, plan.id);
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert!(!sub.cancel_at_period_end);

        // Cancel
        let cancelled = engine.cancel_subscription(sub.id).unwrap();
        assert!(cancelled.cancel_at_period_end);

        // Retrieve
        let fetched = engine.get_subscription(tenant).unwrap();
        assert!(fetched.cancel_at_period_end);
    }

    #[test]
    fn test_generate_invoice() {
        let engine = BillingEngine::new();
        let tenant = Uuid::new_v4();
        let plan = engine.create_plan(
            "Pro".into(),
            "pro".into(),
            499.0,
            4990.0,
            5_000_000,
            1_000_000,
            vec![],
        );
        let sub = engine.subscribe(tenant, plan.id, BillingProvider::Manual);

        let invoice = engine.generate_invoice(
            tenant,
            sub.id,
            vec![
                InvoiceLineItem {
                    description: "Pro Plan".into(),
                    quantity: 1,
                    unit_price: 499.0,
                    amount: 499.0,
                },
                InvoiceLineItem {
                    description: "Overage".into(),
                    quantity: 100_000,
                    unit_price: 0.00001,
                    amount: 1.0,
                },
            ],
        );

        assert_eq!(invoice.tenant_id, tenant);
        assert_eq!(invoice.currency, "USD");
        assert!((invoice.amount - 500.0).abs() < f64::EPSILON);
        assert_eq!(invoice.line_items.len(), 2);

        let invoices = engine.list_invoices(tenant);
        assert_eq!(invoices.len(), 1);
    }
}
