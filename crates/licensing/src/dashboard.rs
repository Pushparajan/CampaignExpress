//! Billing dashboard — tracks all tenant installations, module usage,
//! billing records, and payment status across the Campaign Express fleet.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{License, LicenseTier, LicenseType, LicensedModule};

// ---------------------------------------------------------------------------
// Installation status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationStatus {
    Active,
    Suspended,
    Expired,
    PendingActivation,
    Decommissioned,
}

impl std::fmt::Display for InstallationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Suspended => write!(f, "Suspended"),
            Self::Expired => write!(f, "Expired"),
            Self::PendingActivation => write!(f, "Pending Activation"),
            Self::Decommissioned => write!(f, "Decommissioned"),
        }
    }
}

// ---------------------------------------------------------------------------
// Installation record
// ---------------------------------------------------------------------------

/// A deployed Campaign Express installation (one per tenant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub installation_id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub license: License,
    pub status: InstallationStatus,
    pub node_count: u32,
    pub region: String,
    pub environment: String,
    pub activated_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Usage snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageMeter {
    OffersServed,
    ApiCalls,
    ActiveCampaigns,
    StorageGb,
    BandwidthGb,
    JourneyExecutions,
    DcoRenders,
    CdpSyncs,
    SmsMessages,
    PushNotifications,
}

impl UsageMeter {
    pub const ALL: &'static [UsageMeter] = &[
        Self::OffersServed,
        Self::ApiCalls,
        Self::ActiveCampaigns,
        Self::StorageGb,
        Self::BandwidthGb,
        Self::JourneyExecutions,
        Self::DcoRenders,
        Self::CdpSyncs,
        Self::SmsMessages,
        Self::PushNotifications,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OffersServed => "offers_served",
            Self::ApiCalls => "api_calls",
            Self::ActiveCampaigns => "active_campaigns",
            Self::StorageGb => "storage_gb",
            Self::BandwidthGb => "bandwidth_gb",
            Self::JourneyExecutions => "journey_executions",
            Self::DcoRenders => "dco_renders",
            Self::CdpSyncs => "cdp_syncs",
            Self::SmsMessages => "sms_messages",
            Self::PushNotifications => "push_notifications",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            Self::OffersServed => "offers",
            Self::ApiCalls => "calls",
            Self::ActiveCampaigns => "campaigns",
            Self::StorageGb => "GB",
            Self::BandwidthGb => "GB",
            Self::JourneyExecutions => "executions",
            Self::DcoRenders => "renders",
            Self::CdpSyncs => "syncs",
            Self::SmsMessages => "messages",
            Self::PushNotifications => "notifications",
        }
    }
}

impl std::fmt::Display for UsageMeter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Per-meter usage for one installation in one billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub meter: UsageMeter,
    pub quantity: u64,
    pub quota: Option<u64>,
    pub unit_price_cents: u64,
}

impl UsageEntry {
    pub fn usage_percent(&self) -> f64 {
        match self.quota {
            Some(q) if q > 0 => (self.quantity as f64 / q as f64) * 100.0,
            _ => 0.0,
        }
    }

    pub fn line_total_cents(&self) -> u64 {
        self.quantity * self.unit_price_cents
    }
}

/// Full usage snapshot for one installation in one billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub installation_id: Uuid,
    pub period: String,
    pub entries: Vec<UsageEntry>,
    pub captured_at: DateTime<Utc>,
}

impl UsageSnapshot {
    pub fn total_cost_cents(&self) -> u64 {
        self.entries.iter().map(|e| e.line_total_cents()).sum()
    }
}

// ---------------------------------------------------------------------------
// Billing & Invoices
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    PastDue,
    Void,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Open => write!(f, "Open"),
            Self::Paid => write!(f, "Paid"),
            Self::PastDue => write!(f, "Past Due"),
            Self::Void => write!(f, "Void"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: u64,
    pub unit_price_cents: u64,
    pub total_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInvoice {
    pub invoice_id: Uuid,
    pub installation_id: Uuid,
    pub tenant_name: String,
    pub period: String,
    pub line_items: Vec<LineItem>,
    pub subtotal_cents: u64,
    pub tax_cents: u64,
    pub total_cents: u64,
    pub currency: String,
    pub status: InvoiceStatus,
    pub issued_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Payment records
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Refunded => write!(f, "Refunded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub payment_id: Uuid,
    pub invoice_id: Uuid,
    pub installation_id: Uuid,
    pub tenant_name: String,
    pub amount_cents: u64,
    pub currency: String,
    pub method: String,
    pub status: PaymentStatus,
    pub reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Dashboard summary types
// ---------------------------------------------------------------------------

/// Top-level fleet overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetOverview {
    pub total_installations: usize,
    pub active_installations: usize,
    pub suspended_installations: usize,
    pub expired_installations: usize,
    pub total_nodes: u32,
    pub total_revenue_cents: u64,
    pub pending_payments_cents: u64,
    pub overdue_invoices: usize,
    pub module_adoption: Vec<(String, usize)>,
}

/// Per-installation billing summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationSummary {
    pub installation: Installation,
    pub current_usage: Option<UsageSnapshot>,
    pub invoices: Vec<BillingInvoice>,
    pub payments: Vec<PaymentRecord>,
    pub total_billed_cents: u64,
    pub total_paid_cents: u64,
    pub outstanding_cents: u64,
}

// ---------------------------------------------------------------------------
// Dashboard engine
// ---------------------------------------------------------------------------

/// In-memory billing dashboard engine.
pub struct DashboardEngine {
    installations: Arc<DashMap<Uuid, Installation>>,
    usage: Arc<DashMap<Uuid, Vec<UsageSnapshot>>>,
    invoices: Arc<DashMap<Uuid, BillingInvoice>>,
    payments: Arc<DashMap<Uuid, PaymentRecord>>,
}

impl Default for DashboardEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardEngine {
    pub fn new() -> Self {
        Self {
            installations: Arc::new(DashMap::new()),
            usage: Arc::new(DashMap::new()),
            invoices: Arc::new(DashMap::new()),
            payments: Arc::new(DashMap::new()),
        }
    }

    // -- installations --

    pub fn register_installation(&self, inst: Installation) {
        self.installations.insert(inst.installation_id, inst);
    }

    pub fn get_installation(&self, id: Uuid) -> Option<Installation> {
        self.installations.get(&id).map(|e| e.value().clone())
    }

    pub fn list_installations(&self) -> Vec<Installation> {
        let mut list: Vec<_> = self
            .installations
            .iter()
            .map(|e| e.value().clone())
            .collect();
        list.sort_by(|a, b| a.tenant_name.cmp(&b.tenant_name));
        list
    }

    pub fn update_status(&self, id: Uuid, status: InstallationStatus) -> Option<Installation> {
        self.installations.get_mut(&id).map(|mut e| {
            e.status = status;
            e.clone()
        })
    }

    // -- usage --

    pub fn record_usage(&self, snapshot: UsageSnapshot) {
        self.usage
            .entry(snapshot.installation_id)
            .or_default()
            .push(snapshot);
    }

    pub fn get_latest_usage(&self, installation_id: Uuid) -> Option<UsageSnapshot> {
        self.usage
            .get(&installation_id)
            .and_then(|snaps| snaps.iter().max_by_key(|s| s.captured_at).cloned())
    }

    pub fn get_usage_history(&self, installation_id: Uuid) -> Vec<UsageSnapshot> {
        self.usage
            .get(&installation_id)
            .map(|s| s.value().clone())
            .unwrap_or_default()
    }

    // -- invoices --

    pub fn create_invoice(&self, invoice: BillingInvoice) {
        self.invoices.insert(invoice.invoice_id, invoice);
    }

    pub fn get_invoices_for_installation(&self, installation_id: Uuid) -> Vec<BillingInvoice> {
        let mut list: Vec<_> = self
            .invoices
            .iter()
            .filter(|e| e.value().installation_id == installation_id)
            .map(|e| e.value().clone())
            .collect();
        list.sort_by(|a, b| b.issued_at.cmp(&a.issued_at));
        list
    }

    pub fn get_overdue_invoices(&self) -> Vec<BillingInvoice> {
        let now = Utc::now();
        self.invoices
            .iter()
            .filter(|e| e.value().status == InvoiceStatus::Open && e.value().due_at < now)
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn get_pending_invoices(&self) -> Vec<BillingInvoice> {
        self.invoices
            .iter()
            .filter(|e| {
                matches!(
                    e.value().status,
                    InvoiceStatus::Open | InvoiceStatus::PastDue
                )
            })
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn mark_invoice_paid(&self, invoice_id: Uuid) -> Option<BillingInvoice> {
        self.invoices.get_mut(&invoice_id).map(|mut e| {
            e.status = InvoiceStatus::Paid;
            e.paid_at = Some(Utc::now());
            e.clone()
        })
    }

    // -- payments --

    pub fn record_payment(&self, payment: PaymentRecord) {
        self.payments.insert(payment.payment_id, payment);
    }

    pub fn get_payments_for_installation(&self, installation_id: Uuid) -> Vec<PaymentRecord> {
        let mut list: Vec<_> = self
            .payments
            .iter()
            .filter(|e| e.value().installation_id == installation_id)
            .map(|e| e.value().clone())
            .collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub fn get_pending_payments(&self) -> Vec<PaymentRecord> {
        self.payments
            .iter()
            .filter(|e| e.value().status == PaymentStatus::Pending)
            .map(|e| e.value().clone())
            .collect()
    }

    // -- summaries --

    /// Full fleet overview across all installations.
    pub fn fleet_overview(&self) -> FleetOverview {
        let installations = self.list_installations();
        let active = installations
            .iter()
            .filter(|i| i.status == InstallationStatus::Active)
            .count();
        let suspended = installations
            .iter()
            .filter(|i| i.status == InstallationStatus::Suspended)
            .count();
        let expired = installations
            .iter()
            .filter(|i| i.status == InstallationStatus::Expired)
            .count();
        let total_nodes: u32 = installations.iter().map(|i| i.node_count).sum();

        let total_revenue_cents: u64 = self
            .invoices
            .iter()
            .filter(|e| e.value().status == InvoiceStatus::Paid)
            .map(|e| e.value().total_cents)
            .sum();

        let pending_payments_cents: u64 = self
            .invoices
            .iter()
            .filter(|e| {
                matches!(
                    e.value().status,
                    InvoiceStatus::Open | InvoiceStatus::PastDue
                )
            })
            .map(|e| e.value().total_cents)
            .sum();

        let overdue_invoices = self.get_overdue_invoices().len();

        // Module adoption: count how many installations have each module
        let mut module_counts: Vec<(String, usize)> = LicensedModule::ALL
            .iter()
            .map(|m| {
                let count = installations
                    .iter()
                    .filter(|i| i.license.has_module(*m))
                    .count();
                (m.as_str().to_string(), count)
            })
            .collect();
        module_counts.sort_by(|a, b| b.1.cmp(&a.1));

        FleetOverview {
            total_installations: installations.len(),
            active_installations: active,
            suspended_installations: suspended,
            expired_installations: expired,
            total_nodes,
            total_revenue_cents,
            pending_payments_cents,
            overdue_invoices,
            module_adoption: module_counts,
        }
    }

    /// Detailed summary for one installation.
    pub fn installation_summary(&self, installation_id: Uuid) -> Option<InstallationSummary> {
        let installation = self.get_installation(installation_id)?;
        let current_usage = self.get_latest_usage(installation_id);
        let invoices = self.get_invoices_for_installation(installation_id);
        let payments = self.get_payments_for_installation(installation_id);

        let total_billed_cents: u64 = invoices.iter().map(|i| i.total_cents).sum();
        let total_paid_cents: u64 = invoices
            .iter()
            .filter(|i| i.status == InvoiceStatus::Paid)
            .map(|i| i.total_cents)
            .sum();
        let outstanding_cents = total_billed_cents - total_paid_cents;

        Some(InstallationSummary {
            installation,
            current_usage,
            invoices,
            payments,
            total_billed_cents,
            total_paid_cents,
            outstanding_cents,
        })
    }

    // -- demo data --

    /// Seed realistic demo data: 4 installations with usage, invoices, and payments.
    pub fn seed_demo_data(&self) {
        let now = Utc::now();

        // --- Installation 1: Acme Corp (Enterprise, active) ---
        let acme_id = Uuid::parse_str("10000000-0000-0000-0000-000000000001").unwrap();
        let acme_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let acme_license = License {
            license_id: Uuid::new_v4(),
            tenant_id: acme_tenant,
            tenant_name: "Acme Corp".into(),
            license_type: LicenseType::Commercial,
            tier: LicenseTier::Enterprise,
            modules: LicensedModule::ALL.to_vec(),
            max_nodes: 20,
            max_offers_per_hour: 50_000_000,
            issued_at: now - Duration::days(180),
            expires_at: now + Duration::days(185),
            issued_by: "license-admin".into(),
        };
        self.register_installation(Installation {
            installation_id: acme_id,
            tenant_id: acme_tenant,
            tenant_name: "Acme Corp".into(),
            license: acme_license,
            status: InstallationStatus::Active,
            node_count: 20,
            region: "eastus".into(),
            environment: "production".into(),
            activated_at: now - Duration::days(180),
            last_heartbeat: now - Duration::minutes(5),
        });

        // --- Installation 2: Globex Inc (Professional, active) ---
        let globex_id = Uuid::parse_str("10000000-0000-0000-0000-000000000002").unwrap();
        let globex_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let globex_license = License {
            license_id: Uuid::new_v4(),
            tenant_id: globex_tenant,
            tenant_name: "Globex Inc".into(),
            license_type: LicenseType::Commercial,
            tier: LicenseTier::Professional,
            modules: LicenseTier::Professional.default_modules(),
            max_nodes: 10,
            max_offers_per_hour: 10_000_000,
            issued_at: now - Duration::days(90),
            expires_at: now + Duration::days(275),
            issued_by: "license-admin".into(),
        };
        self.register_installation(Installation {
            installation_id: globex_id,
            tenant_id: globex_tenant,
            tenant_name: "Globex Inc".into(),
            license: globex_license,
            status: InstallationStatus::Active,
            node_count: 8,
            region: "westeurope".into(),
            environment: "production".into(),
            activated_at: now - Duration::days(90),
            last_heartbeat: now - Duration::minutes(12),
        });

        // --- Installation 3: Initech (Starter, active) ---
        let initech_id = Uuid::parse_str("10000000-0000-0000-0000-000000000003").unwrap();
        let initech_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        let initech_license = License {
            license_id: Uuid::new_v4(),
            tenant_id: initech_tenant,
            tenant_name: "Initech".into(),
            license_type: LicenseType::Commercial,
            tier: LicenseTier::Starter,
            modules: LicenseTier::Starter.default_modules(),
            max_nodes: 3,
            max_offers_per_hour: 1_000_000,
            issued_at: now - Duration::days(30),
            expires_at: now + Duration::days(335),
            issued_by: "license-admin".into(),
        };
        self.register_installation(Installation {
            installation_id: initech_id,
            tenant_id: initech_tenant,
            tenant_name: "Initech".into(),
            license: initech_license,
            status: InstallationStatus::Active,
            node_count: 2,
            region: "southeastasia".into(),
            environment: "production".into(),
            activated_at: now - Duration::days(30),
            last_heartbeat: now - Duration::minutes(2),
        });

        // --- Installation 4: Umbrella Corp (Trial, expired) ---
        let umbrella_id = Uuid::parse_str("10000000-0000-0000-0000-000000000004").unwrap();
        let umbrella_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();
        let umbrella_license = License {
            license_id: Uuid::new_v4(),
            tenant_id: umbrella_tenant,
            tenant_name: "Umbrella Corp".into(),
            license_type: LicenseType::Trial,
            tier: LicenseTier::Professional,
            modules: LicenseTier::Professional.default_modules(),
            max_nodes: 5,
            max_offers_per_hour: 5_000_000,
            issued_at: now - Duration::days(44),
            expires_at: now - Duration::days(14),
            issued_by: "license-admin".into(),
        };
        self.register_installation(Installation {
            installation_id: umbrella_id,
            tenant_id: umbrella_tenant,
            tenant_name: "Umbrella Corp".into(),
            license: umbrella_license,
            status: InstallationStatus::Expired,
            node_count: 5,
            region: "eastus".into(),
            environment: "staging".into(),
            activated_at: now - Duration::days(44),
            last_heartbeat: now - Duration::days(14),
        });

        // --- Usage snapshots ---
        self.record_usage(UsageSnapshot {
            installation_id: acme_id,
            period: "2026-02".into(),
            entries: vec![
                UsageEntry {
                    meter: UsageMeter::OffersServed,
                    quantity: 38_500_000,
                    quota: Some(50_000_000),
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::ApiCalls,
                    quantity: 12_400_000,
                    quota: Some(20_000_000),
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::ActiveCampaigns,
                    quantity: 47,
                    quota: None,
                    unit_price_cents: 500,
                },
                UsageEntry {
                    meter: UsageMeter::StorageGb,
                    quantity: 128,
                    quota: Some(500),
                    unit_price_cents: 10,
                },
                UsageEntry {
                    meter: UsageMeter::JourneyExecutions,
                    quantity: 2_300_000,
                    quota: None,
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::DcoRenders,
                    quantity: 8_700_000,
                    quota: None,
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::SmsMessages,
                    quantity: 450_000,
                    quota: Some(1_000_000),
                    unit_price_cents: 1,
                },
            ],
            captured_at: now,
        });

        self.record_usage(UsageSnapshot {
            installation_id: globex_id,
            period: "2026-02".into(),
            entries: vec![
                UsageEntry {
                    meter: UsageMeter::OffersServed,
                    quantity: 7_200_000,
                    quota: Some(10_000_000),
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::ApiCalls,
                    quantity: 3_100_000,
                    quota: Some(5_000_000),
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::ActiveCampaigns,
                    quantity: 18,
                    quota: None,
                    unit_price_cents: 500,
                },
                UsageEntry {
                    meter: UsageMeter::StorageGb,
                    quantity: 42,
                    quota: Some(200),
                    unit_price_cents: 10,
                },
                UsageEntry {
                    meter: UsageMeter::CdpSyncs,
                    quantity: 85_000,
                    quota: None,
                    unit_price_cents: 0,
                },
            ],
            captured_at: now,
        });

        self.record_usage(UsageSnapshot {
            installation_id: initech_id,
            period: "2026-02".into(),
            entries: vec![
                UsageEntry {
                    meter: UsageMeter::OffersServed,
                    quantity: 620_000,
                    quota: Some(1_000_000),
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::ApiCalls,
                    quantity: 180_000,
                    quota: Some(500_000),
                    unit_price_cents: 0,
                },
                UsageEntry {
                    meter: UsageMeter::ActiveCampaigns,
                    quantity: 5,
                    quota: None,
                    unit_price_cents: 500,
                },
            ],
            captured_at: now,
        });

        // --- Invoices ---
        // Acme: 3 months of invoices
        for month_offset in [2, 1, 0] {
            let issued = now - Duration::days(month_offset * 30);
            let status = if month_offset > 0 {
                InvoiceStatus::Paid
            } else {
                InvoiceStatus::Open
            };
            let paid_at = if month_offset > 0 {
                Some(issued + Duration::days(15))
            } else {
                None
            };
            self.create_invoice(BillingInvoice {
                invoice_id: Uuid::new_v4(),
                installation_id: acme_id,
                tenant_name: "Acme Corp".into(),
                period: format!(
                    "2026-{:02}",
                    if month_offset == 0 {
                        2
                    } else {
                        2 - month_offset
                    }
                ),
                line_items: vec![
                    LineItem {
                        description: "Enterprise License".into(),
                        quantity: 1,
                        unit_price_cents: 199_900,
                        total_cents: 199_900,
                    },
                    LineItem {
                        description: "NPU Node Pack (20 nodes)".into(),
                        quantity: 20,
                        unit_price_cents: 5_000,
                        total_cents: 100_000,
                    },
                    LineItem {
                        description: "SMS Messages".into(),
                        quantity: 450_000,
                        unit_price_cents: 1,
                        total_cents: 450_000,
                    },
                ],
                subtotal_cents: 749_900,
                tax_cents: 67_491,
                total_cents: 817_391,
                currency: "USD".into(),
                status,
                issued_at: issued,
                due_at: issued + Duration::days(30),
                paid_at,
            });
        }

        // Globex: 2 months, one overdue
        let globex_inv1 = BillingInvoice {
            invoice_id: Uuid::new_v4(),
            installation_id: globex_id,
            tenant_name: "Globex Inc".into(),
            period: "2025-12".into(),
            line_items: vec![
                LineItem {
                    description: "Professional License".into(),
                    quantity: 1,
                    unit_price_cents: 49_900,
                    total_cents: 49_900,
                },
                LineItem {
                    description: "Node Pack (8 nodes)".into(),
                    quantity: 8,
                    unit_price_cents: 5_000,
                    total_cents: 40_000,
                },
            ],
            subtotal_cents: 89_900,
            tax_cents: 8_091,
            total_cents: 97_991,
            currency: "USD".into(),
            status: InvoiceStatus::Paid,
            issued_at: now - Duration::days(60),
            due_at: now - Duration::days(30),
            paid_at: Some(now - Duration::days(45)),
        };
        self.create_invoice(globex_inv1);

        let globex_inv2 = BillingInvoice {
            invoice_id: Uuid::new_v4(),
            installation_id: globex_id,
            tenant_name: "Globex Inc".into(),
            period: "2026-01".into(),
            line_items: vec![
                LineItem {
                    description: "Professional License".into(),
                    quantity: 1,
                    unit_price_cents: 49_900,
                    total_cents: 49_900,
                },
                LineItem {
                    description: "Node Pack (8 nodes)".into(),
                    quantity: 8,
                    unit_price_cents: 5_000,
                    total_cents: 40_000,
                },
            ],
            subtotal_cents: 89_900,
            tax_cents: 8_091,
            total_cents: 97_991,
            currency: "USD".into(),
            status: InvoiceStatus::Open,
            issued_at: now - Duration::days(30),
            due_at: now - Duration::days(1),
            paid_at: None,
        };
        self.create_invoice(globex_inv2);

        // Initech: 1 invoice paid
        self.create_invoice(BillingInvoice {
            invoice_id: Uuid::new_v4(),
            installation_id: initech_id,
            tenant_name: "Initech".into(),
            period: "2026-01".into(),
            line_items: vec![
                LineItem {
                    description: "Starter License".into(),
                    quantity: 1,
                    unit_price_cents: 9_900,
                    total_cents: 9_900,
                },
                LineItem {
                    description: "Node Pack (2 nodes)".into(),
                    quantity: 2,
                    unit_price_cents: 5_000,
                    total_cents: 10_000,
                },
            ],
            subtotal_cents: 19_900,
            tax_cents: 1_791,
            total_cents: 21_691,
            currency: "USD".into(),
            status: InvoiceStatus::Paid,
            issued_at: now - Duration::days(30),
            due_at: now,
            paid_at: Some(now - Duration::days(10)),
        });

        // --- Payments ---
        // Acme: 2 completed payments
        for i in 0..2 {
            self.record_payment(PaymentRecord {
                payment_id: Uuid::new_v4(),
                invoice_id: Uuid::new_v4(), // simplified
                installation_id: acme_id,
                tenant_name: "Acme Corp".into(),
                amount_cents: 817_391,
                currency: "USD".into(),
                method: "card ending 4242".into(),
                status: PaymentStatus::Completed,
                reference: Some(format!("stripe_pi_acme_{}", i + 1)),
                created_at: now - Duration::days((2 - i) * 30),
                completed_at: Some(now - Duration::days((2 - i) * 30)),
            });
        }

        // Globex: 1 completed, 1 pending
        self.record_payment(PaymentRecord {
            payment_id: Uuid::new_v4(),
            invoice_id: Uuid::new_v4(),
            installation_id: globex_id,
            tenant_name: "Globex Inc".into(),
            amount_cents: 97_991,
            currency: "USD".into(),
            method: "card ending 8888".into(),
            status: PaymentStatus::Completed,
            reference: Some("stripe_pi_globex_1".into()),
            created_at: now - Duration::days(45),
            completed_at: Some(now - Duration::days(45)),
        });
        self.record_payment(PaymentRecord {
            payment_id: Uuid::new_v4(),
            invoice_id: Uuid::new_v4(),
            installation_id: globex_id,
            tenant_name: "Globex Inc".into(),
            amount_cents: 97_991,
            currency: "USD".into(),
            method: "card ending 8888".into(),
            status: PaymentStatus::Pending,
            reference: None,
            created_at: now - Duration::days(1),
            completed_at: None,
        });

        // Initech: 1 completed
        self.record_payment(PaymentRecord {
            payment_id: Uuid::new_v4(),
            invoice_id: Uuid::new_v4(),
            installation_id: initech_id,
            tenant_name: "Initech".into(),
            amount_cents: 21_691,
            currency: "USD".into(),
            method: "bank transfer".into(),
            status: PaymentStatus::Completed,
            reference: Some("wire_initech_001".into()),
            created_at: now - Duration::days(10),
            completed_at: Some(now - Duration::days(10)),
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_and_fleet_overview() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let overview = engine.fleet_overview();
        assert_eq!(overview.total_installations, 4);
        assert_eq!(overview.active_installations, 3);
        assert_eq!(overview.expired_installations, 1);
        assert!(overview.total_nodes >= 35);
        assert!(overview.total_revenue_cents > 0);
    }

    #[test]
    fn test_installation_summary() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let acme_id = Uuid::parse_str("10000000-0000-0000-0000-000000000001").unwrap();
        let summary = engine.installation_summary(acme_id).unwrap();

        assert_eq!(summary.installation.tenant_name, "Acme Corp");
        assert_eq!(summary.installation.status, InstallationStatus::Active);
        assert!(!summary.invoices.is_empty());
        assert!(summary.current_usage.is_some());
        assert!(summary.total_billed_cents > 0);
    }

    #[test]
    fn test_pending_payments() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let pending = engine.get_pending_payments();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].tenant_name, "Globex Inc");
    }

    #[test]
    fn test_overdue_invoices() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let overdue = engine.get_overdue_invoices();
        // Globex has one open invoice past due date
        assert!(overdue.iter().any(|i| i.tenant_name == "Globex Inc"));
    }

    #[test]
    fn test_usage_entry_calculations() {
        let entry = UsageEntry {
            meter: UsageMeter::OffersServed,
            quantity: 7_500_000,
            quota: Some(10_000_000),
            unit_price_cents: 0,
        };
        assert!((entry.usage_percent() - 75.0).abs() < 0.01);

        let entry2 = UsageEntry {
            meter: UsageMeter::ActiveCampaigns,
            quantity: 18,
            quota: None,
            unit_price_cents: 500,
        };
        assert_eq!(entry2.line_total_cents(), 9_000);
    }

    #[test]
    fn test_mark_invoice_paid() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let pending = engine.get_pending_invoices();
        assert!(!pending.is_empty());

        let invoice_id = pending[0].invoice_id;
        let paid = engine.mark_invoice_paid(invoice_id).unwrap();
        assert_eq!(paid.status, InvoiceStatus::Paid);
        assert!(paid.paid_at.is_some());
    }

    #[test]
    fn test_module_adoption_counts() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let overview = engine.fleet_overview();
        // "management" should be adopted by all 4 installations
        // (Enterprise has all, Pro has it, Starter has it, Trial/Pro has it)
        let management = overview
            .module_adoption
            .iter()
            .find(|(name, _)| name == "management")
            .unwrap();
        assert_eq!(management.1, 4);
    }

    #[test]
    fn test_update_installation_status() {
        let engine = DashboardEngine::new();
        engine.seed_demo_data();

        let initech_id = Uuid::parse_str("10000000-0000-0000-0000-000000000003").unwrap();
        let updated = engine
            .update_status(initech_id, InstallationStatus::Suspended)
            .unwrap();
        assert_eq!(updated.status, InstallationStatus::Suspended);
    }
}
