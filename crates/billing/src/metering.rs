//! Usage metering engine — tracks per-tenant consumption across multiple
//! meter types and computes cost summaries against a configurable pricing table.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The kind of resource being metered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterType {
    OffersServed,
    ApiCalls,
    CampaignsActive,
    StorageBytes,
    BandwidthBytes,
    JourneyExecutions,
    DcoRenders,
    CdpSyncs,
}

impl MeterType {
    /// Pricing-table key for this meter type.
    fn pricing_key(self) -> &'static str {
        match self {
            Self::OffersServed => "offers_served",
            Self::ApiCalls => "api_calls",
            Self::CampaignsActive => "campaigns_active",
            Self::StorageBytes => "storage_bytes",
            Self::BandwidthBytes => "bandwidth_bytes",
            Self::JourneyExecutions => "journey_executions",
            Self::DcoRenders => "dco_renders",
            Self::CdpSyncs => "cdp_syncs",
        }
    }
}

impl std::fmt::Display for MeterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pricing_key())
    }
}

/// A single recorded usage event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub meter_type: MeterType,
    pub quantity: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
}

/// Aggregated summary for one meter type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterSummary {
    pub meter_type: MeterType,
    pub total_quantity: u64,
    pub unit_price: f64,
    pub line_total: f64,
    pub quota: Option<u64>,
    pub usage_percent: f64,
}

/// Full usage summary for a tenant in a given period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub tenant_id: Uuid,
    pub period: String,
    pub meters: Vec<MeterSummary>,
    pub total_cost: f64,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// In-memory metering engine backed by `DashMap`.
pub struct MeteringEngine {
    records: Arc<DashMap<Uuid, Vec<UsageRecord>>>,
    pricing_table: Arc<DashMap<String, f64>>,
}

impl Default for MeteringEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MeteringEngine {
    /// Create a new engine with default pricing seeded.
    pub fn new() -> Self {
        let pricing_table = DashMap::new();
        // price per unit (or per 1 000 where noted)
        pricing_table.insert("offers_served".into(), 0.01 / 1000.0); // $0.01 per 1000
        pricing_table.insert("api_calls".into(), 0.005 / 1000.0); // $0.005 per 1000
        pricing_table.insert("campaigns_active".into(), 5.0); // $5 per active campaign
        pricing_table.insert("storage_bytes".into(), 0.10 / 1_073_741_824.0); // $0.10 per GB
        pricing_table.insert("bandwidth_bytes".into(), 0.08 / 1_073_741_824.0); // $0.08 per GB
        pricing_table.insert("journey_executions".into(), 0.02 / 1000.0); // $0.02 per 1000
        pricing_table.insert("dco_renders".into(), 0.015 / 1000.0); // $0.015 per 1000
        pricing_table.insert("cdp_syncs".into(), 0.03 / 1000.0); // $0.03 per 1000

        info!("MeteringEngine initialized with default pricing");

        Self {
            records: Arc::new(DashMap::new()),
            pricing_table: Arc::new(pricing_table),
        }
    }

    /// Record a usage event for a tenant.
    pub fn record_usage(&self, tenant_id: Uuid, meter_type: MeterType, quantity: u64) {
        let now = Utc::now();
        let record = UsageRecord {
            id: Uuid::new_v4(),
            tenant_id,
            meter_type,
            quantity,
            period_start: now,
            period_end: now,
            recorded_at: now,
        };
        self.records
            .entry(tenant_id)
            .or_default()
            .push(record);
    }

    /// Build a usage summary for the given tenant and period label.
    pub fn get_usage_summary(&self, tenant_id: Uuid, period: &str) -> UsageSummary {
        let meter_types = [
            MeterType::OffersServed,
            MeterType::ApiCalls,
            MeterType::CampaignsActive,
            MeterType::StorageBytes,
            MeterType::BandwidthBytes,
            MeterType::JourneyExecutions,
            MeterType::DcoRenders,
            MeterType::CdpSyncs,
        ];

        let mut meters = Vec::new();
        let mut total_cost = 0.0;

        for mt in &meter_types {
            let total_quantity = self.get_current_usage(tenant_id, mt);
            let unit_price = self
                .pricing_table
                .get(mt.pricing_key())
                .map(|v| *v)
                .unwrap_or(0.0);
            let line_total = total_quantity as f64 * unit_price;
            total_cost += line_total;

            meters.push(MeterSummary {
                meter_type: *mt,
                total_quantity,
                unit_price,
                line_total,
                quota: None,
                usage_percent: 0.0,
            });
        }

        UsageSummary {
            tenant_id,
            period: period.to_string(),
            meters,
            total_cost,
        }
    }

    /// Return the total quantity recorded for a specific meter type.
    pub fn get_current_usage(&self, tenant_id: Uuid, meter_type: &MeterType) -> u64 {
        self.records
            .get(&tenant_id)
            .map(|recs| {
                recs.iter()
                    .filter(|r| r.meter_type == *meter_type)
                    .map(|r| r.quantity)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Returns `true` if the tenant's current usage is strictly under the quota.
    pub fn check_quota(&self, tenant_id: Uuid, meter_type: &MeterType, quota: u64) -> bool {
        self.get_current_usage(tenant_id, meter_type) < quota
    }

    /// Seed demo usage data for two synthetic tenants.
    pub fn seed_demo_usage(&self) {
        let tenant_a = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let tenant_b = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        self.record_usage(tenant_a, MeterType::OffersServed, 1_250_000);
        self.record_usage(tenant_a, MeterType::ApiCalls, 340_000);
        self.record_usage(tenant_a, MeterType::CampaignsActive, 12);
        self.record_usage(tenant_a, MeterType::StorageBytes, 5_368_709_120); // ~5 GB
        self.record_usage(tenant_a, MeterType::JourneyExecutions, 78_000);

        self.record_usage(tenant_b, MeterType::OffersServed, 450_000);
        self.record_usage(tenant_b, MeterType::ApiCalls, 120_000);
        self.record_usage(tenant_b, MeterType::CampaignsActive, 4);
        self.record_usage(tenant_b, MeterType::DcoRenders, 95_000);
        self.record_usage(tenant_b, MeterType::CdpSyncs, 15_000);

        info!("Seeded demo usage for 2 tenants");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage() {
        let engine = MeteringEngine::new();
        let tenant = Uuid::new_v4();

        engine.record_usage(tenant, MeterType::OffersServed, 5000);
        engine.record_usage(tenant, MeterType::OffersServed, 3000);
        engine.record_usage(tenant, MeterType::ApiCalls, 1000);

        assert_eq!(
            engine.get_current_usage(tenant, &MeterType::OffersServed),
            8000
        );
        assert_eq!(
            engine.get_current_usage(tenant, &MeterType::ApiCalls),
            1000
        );
        assert!(engine.check_quota(tenant, &MeterType::OffersServed, 10_000));
        assert!(!engine.check_quota(tenant, &MeterType::OffersServed, 5000));
    }

    #[test]
    fn test_usage_summary() {
        let engine = MeteringEngine::new();
        let tenant = Uuid::new_v4();

        engine.record_usage(tenant, MeterType::OffersServed, 100_000);
        engine.record_usage(tenant, MeterType::ApiCalls, 50_000);

        let summary = engine.get_usage_summary(tenant, "2026-02");
        assert_eq!(summary.tenant_id, tenant);
        assert_eq!(summary.period, "2026-02");
        assert!(summary.total_cost > 0.0);

        let offers_meter = summary
            .meters
            .iter()
            .find(|m| m.meter_type == MeterType::OffersServed)
            .unwrap();
        assert_eq!(offers_meter.total_quantity, 100_000);
    }
}
