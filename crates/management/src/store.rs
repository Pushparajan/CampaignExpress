//! In-memory management store backed by DashMap.
//!
//! Production: replace with PostgreSQL (sqlx) or similar ACID store.
//! This provides the same API surface for development and testing.

use crate::models::*;
use chrono::Utc;
use dashmap::DashMap;
use tracing::info;
use uuid::Uuid;

/// Thread-safe in-memory store for campaigns, creatives, journeys, DCO, CDP, experiments,
/// platform (tenants, roles, compliance, privacy), users, billing, ops, and audit log.
pub struct ManagementStore {
    campaigns: DashMap<Uuid, Campaign>,
    creatives: DashMap<Uuid, Creative>,
    journeys: DashMap<Uuid, serde_json::Value>,
    dco_templates: DashMap<Uuid, serde_json::Value>,
    cdp_platforms: DashMap<String, serde_json::Value>,
    cdp_sync_history: DashMap<Uuid, serde_json::Value>,
    experiments: DashMap<Uuid, serde_json::Value>,
    audit_log: DashMap<Uuid, AuditLogEntry>,
    // Platform
    tenants: DashMap<Uuid, serde_json::Value>,
    roles: DashMap<Uuid, serde_json::Value>,
    compliance: DashMap<String, serde_json::Value>,
    dsrs: DashMap<Uuid, serde_json::Value>,
    // Users
    users: DashMap<Uuid, serde_json::Value>,
    invitations: DashMap<Uuid, serde_json::Value>,
    // Billing
    plans: DashMap<Uuid, serde_json::Value>,
    subscriptions: DashMap<Uuid, serde_json::Value>,
    invoices: DashMap<Uuid, serde_json::Value>,
    // Ops
    status_components: DashMap<Uuid, serde_json::Value>,
    incidents: DashMap<Uuid, serde_json::Value>,
    sla_targets: DashMap<String, serde_json::Value>,
    backup_schedules: DashMap<Uuid, serde_json::Value>,
}

impl ManagementStore {
    pub fn new() -> Self {
        info!("Management store initialized (in-memory, development mode)");
        let store = Self {
            campaigns: DashMap::new(),
            creatives: DashMap::new(),
            journeys: DashMap::new(),
            dco_templates: DashMap::new(),
            cdp_platforms: DashMap::new(),
            cdp_sync_history: DashMap::new(),
            experiments: DashMap::new(),
            audit_log: DashMap::new(),
            users: DashMap::new(),
            invitations: DashMap::new(),
            tenants: DashMap::new(),
            roles: DashMap::new(),
            compliance: DashMap::new(),
            dsrs: DashMap::new(),
            plans: DashMap::new(),
            subscriptions: DashMap::new(),
            invoices: DashMap::new(),
            status_components: DashMap::new(),
            incidents: DashMap::new(),
            sla_targets: DashMap::new(),
            backup_schedules: DashMap::new(),
        };
        store.seed_demo_data();
        store.seed_journey_data();
        store.seed_dco_data();
        store.seed_cdp_data();
        store.seed_experiment_data();
        store.seed_platform_data();
        store.seed_billing_data();
        store.seed_ops_data();
        store.seed_user_data();
        store
    }

    // ─── Campaigns ─────────────────────────────────────────────────────────

    pub fn list_campaigns(&self) -> Vec<Campaign> {
        let mut campaigns: Vec<Campaign> =
            self.campaigns.iter().map(|r| r.value().clone()).collect();
        campaigns.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        campaigns
    }

    pub fn get_campaign(&self, id: Uuid) -> Option<Campaign> {
        self.campaigns.get(&id).map(|r| r.value().clone())
    }

    pub fn create_campaign(
        &self,
        req: CreateCampaignRequest,
        user: &str,
    ) -> Result<Campaign, String> {
        // Validate budget
        if req.budget < 0.0 {
            return Err("Budget cannot be negative".into());
        }
        if req.daily_budget < 0.0 {
            return Err("Daily budget cannot be negative".into());
        }
        if req.daily_budget > req.budget && req.budget > 0.0 {
            return Err("Daily budget cannot exceed total budget".into());
        }
        // Validate schedule
        if let (Some(start), Some(end)) = (req.schedule_start, req.schedule_end) {
            if end <= start {
                return Err("Schedule end must be after schedule start".into());
            }
        }
        let now = Utc::now();
        let campaign = Campaign {
            id: Uuid::new_v4(),
            name: req.name,
            status: CampaignStatus::Draft,
            budget: req.budget,
            daily_budget: req.daily_budget,
            pacing: req.pacing,
            targeting: req.targeting,
            schedule_start: req.schedule_start,
            schedule_end: req.schedule_end,
            created_at: now,
            updated_at: now,
            stats: CampaignStats::default(),
        };
        let id = campaign.id;
        self.campaigns.insert(id, campaign.clone());
        self.log_audit(
            user,
            AuditAction::Create,
            "campaign",
            &id.to_string(),
            serde_json::json!({"name": &campaign.name}),
        );
        Ok(campaign)
    }

    pub fn update_campaign(
        &self,
        id: Uuid,
        req: UpdateCampaignRequest,
        user: &str,
    ) -> Option<Campaign> {
        self.campaigns.get_mut(&id).map(|mut entry| {
            let c = entry.value_mut();
            if let Some(name) = req.name {
                c.name = name;
            }
            if let Some(budget) = req.budget {
                c.budget = budget;
            }
            if let Some(daily_budget) = req.daily_budget {
                c.daily_budget = daily_budget;
            }
            if let Some(pacing) = req.pacing {
                c.pacing = pacing;
            }
            if let Some(targeting) = req.targeting {
                c.targeting = targeting;
            }
            if let Some(start) = req.schedule_start {
                c.schedule_start = Some(start);
            }
            if let Some(end) = req.schedule_end {
                c.schedule_end = Some(end);
            }
            c.updated_at = Utc::now();
            self.log_audit(
                user,
                AuditAction::Update,
                "campaign",
                &id.to_string(),
                serde_json::json!({}),
            );
            c.clone()
        })
    }

    pub fn delete_campaign(&self, id: Uuid, user: &str) -> bool {
        let removed = self.campaigns.remove(&id).is_some();
        if removed {
            // Also remove associated creatives
            let creative_ids: Vec<Uuid> = self
                .creatives
                .iter()
                .filter(|r| r.value().campaign_id == id)
                .map(|r| *r.key())
                .collect();
            for cid in creative_ids {
                self.creatives.remove(&cid);
            }
            self.log_audit(
                user,
                AuditAction::Delete,
                "campaign",
                &id.to_string(),
                serde_json::json!({}),
            );
        }
        removed
    }

    pub fn pause_campaign(&self, id: Uuid, user: &str) -> Option<Campaign> {
        self.campaigns.get_mut(&id).map(|mut entry| {
            entry.value_mut().status = CampaignStatus::Paused;
            entry.value_mut().updated_at = Utc::now();
            self.log_audit(
                user,
                AuditAction::Pause,
                "campaign",
                &id.to_string(),
                serde_json::json!({}),
            );
            entry.value().clone()
        })
    }

    pub fn resume_campaign(&self, id: Uuid, user: &str) -> Option<Campaign> {
        self.campaigns.get_mut(&id).map(|mut entry| {
            entry.value_mut().status = CampaignStatus::Active;
            entry.value_mut().updated_at = Utc::now();
            self.log_audit(
                user,
                AuditAction::Resume,
                "campaign",
                &id.to_string(),
                serde_json::json!({}),
            );
            entry.value().clone()
        })
    }

    // ─── Creatives ─────────────────────────────────────────────────────────

    pub fn list_creatives(&self) -> Vec<Creative> {
        let mut creatives: Vec<Creative> =
            self.creatives.iter().map(|r| r.value().clone()).collect();
        creatives.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        creatives
    }

    pub fn get_creative(&self, id: Uuid) -> Option<Creative> {
        self.creatives.get(&id).map(|r| r.value().clone())
    }

    pub fn create_creative(
        &self,
        req: CreateCreativeRequest,
        user: &str,
    ) -> Result<Creative, String> {
        if req.width == 0 || req.height == 0 {
            return Err("Creative dimensions must be greater than zero".into());
        }
        let now = Utc::now();
        let creative = Creative {
            id: Uuid::new_v4(),
            campaign_id: req.campaign_id,
            name: req.name,
            format: req.format,
            asset_url: req.asset_url,
            width: req.width,
            height: req.height,
            status: CreativeStatus::Draft,
            metadata: req.metadata,
            created_at: now,
            updated_at: now,
        };
        let id = creative.id;
        self.creatives.insert(id, creative.clone());
        self.log_audit(
            user,
            AuditAction::Create,
            "creative",
            &id.to_string(),
            serde_json::json!({"name": &creative.name}),
        );
        Ok(creative)
    }

    pub fn update_creative(
        &self,
        id: Uuid,
        req: UpdateCreativeRequest,
        user: &str,
    ) -> Option<Creative> {
        self.creatives.get_mut(&id).map(|mut entry| {
            let c = entry.value_mut();
            if let Some(name) = req.name {
                c.name = name;
            }
            if let Some(format) = req.format {
                c.format = format;
            }
            if let Some(url) = req.asset_url {
                c.asset_url = url;
            }
            if let Some(w) = req.width {
                c.width = w;
            }
            if let Some(h) = req.height {
                c.height = h;
            }
            if let Some(status) = req.status {
                c.status = status;
            }
            if let Some(meta) = req.metadata {
                c.metadata = meta;
            }
            c.updated_at = Utc::now();
            self.log_audit(
                user,
                AuditAction::Update,
                "creative",
                &id.to_string(),
                serde_json::json!({}),
            );
            c.clone()
        })
    }

    pub fn delete_creative(&self, id: Uuid, user: &str) -> bool {
        let removed = self.creatives.remove(&id).is_some();
        if removed {
            self.log_audit(
                user,
                AuditAction::Delete,
                "creative",
                &id.to_string(),
                serde_json::json!({}),
            );
        }
        removed
    }

    // ─── Monitoring ────────────────────────────────────────────────────────

    pub fn get_monitoring_overview(&self) -> MonitoringOverview {
        let total = self.campaigns.len() as u64;
        let active = self
            .campaigns
            .iter()
            .filter(|r| r.value().status == CampaignStatus::Active)
            .count() as u64;
        let total_impressions: u64 = self
            .campaigns
            .iter()
            .map(|r| r.value().stats.impressions)
            .sum();
        let total_clicks: u64 = self.campaigns.iter().map(|r| r.value().stats.clicks).sum();
        let total_spend: f64 = self.campaigns.iter().map(|r| r.value().stats.spend).sum();
        let avg_ctr = if total_impressions > 0 {
            (total_clicks as f64 / total_impressions as f64).min(1.0)
        } else {
            0.0
        };

        MonitoringOverview {
            total_campaigns: total,
            active_campaigns: active,
            total_impressions,
            total_clicks,
            total_spend,
            avg_ctr,
            avg_latency_us: 2500.0,
            active_pods: 20,
            offers_per_hour: 50_000_000,
            cache_hit_rate: 0.94,
            no_bid_rate: 0.12,
            error_rate: 0.001,
        }
    }

    pub fn get_campaign_stats(&self, campaign_id: Uuid) -> Option<CampaignStats> {
        self.campaigns
            .get(&campaign_id)
            .map(|r| r.value().stats.clone())
    }

    // ─── Audit Log ─────────────────────────────────────────────────────────

    pub fn get_audit_log(&self) -> Vec<AuditLogEntry> {
        let mut entries: Vec<AuditLogEntry> =
            self.audit_log.iter().map(|r| r.value().clone()).collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    fn log_audit(
        &self,
        user: &str,
        action: AuditAction,
        resource_type: &str,
        resource_id: &str,
        details: serde_json::Value,
    ) {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            user: user.to_string(),
            action,
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            details,
            timestamp: Utc::now(),
        };
        self.audit_log.insert(entry.id, entry);
    }

    // ─── Journeys ────────────────────────────────────────────────────────

    pub fn list_journeys(&self) -> Vec<serde_json::Value> {
        let mut journeys: Vec<serde_json::Value> =
            self.journeys.iter().map(|r| r.value().clone()).collect();
        journeys.sort_by(|a, b| {
            let a_date = a.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            let b_date = b.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            b_date.cmp(a_date)
        });
        journeys
    }

    pub fn get_journey(&self, id: Uuid) -> Option<serde_json::Value> {
        self.journeys.get(&id).map(|r| r.value().clone())
    }

    pub fn create_journey(&self, mut req: serde_json::Value, user: &str) -> serde_json::Value {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        req["id"] = serde_json::json!(id);
        req["created_at"] = serde_json::json!(now);
        req["updated_at"] = serde_json::json!(now);
        if req.get("status").is_none() {
            req["status"] = serde_json::json!("draft");
        }
        if req.get("version").is_none() {
            req["version"] = serde_json::json!(1);
        }
        self.journeys.insert(id, req.clone());
        self.log_audit(
            user,
            AuditAction::Create,
            "journey",
            &id.to_string(),
            serde_json::json!({}),
        );
        req
    }

    pub fn delete_journey(&self, id: Uuid, user: &str) -> bool {
        let removed = self.journeys.remove(&id).is_some();
        if removed {
            self.log_audit(
                user,
                AuditAction::Delete,
                "journey",
                &id.to_string(),
                serde_json::json!({}),
            );
        }
        removed
    }

    pub fn get_journey_stats(&self, id: Uuid) -> serde_json::Value {
        serde_json::json!({
            "journey_id": id,
            "total_entered": 15420,
            "active": 3200,
            "completed": 10800,
            "exited": 1120,
            "error": 300,
            "avg_completion_time_secs": 86400.0,
            "step_conversion_rates": {
                "step_1": 0.95,
                "step_2": 0.72,
                "step_3": 0.48,
            }
        })
    }

    // ─── DCO Templates ──────────────────────────────────────────────────

    pub fn list_dco_templates(&self) -> Vec<serde_json::Value> {
        self.dco_templates
            .iter()
            .map(|r| r.value().clone())
            .collect()
    }

    pub fn get_dco_template(&self, id: Uuid) -> Option<serde_json::Value> {
        self.dco_templates.get(&id).map(|r| r.value().clone())
    }

    pub fn create_dco_template(&self, mut req: serde_json::Value, user: &str) -> serde_json::Value {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        req["id"] = serde_json::json!(id);
        req["created_at"] = serde_json::json!(now);
        req["updated_at"] = serde_json::json!(now);
        if req.get("status").is_none() {
            req["status"] = serde_json::json!("draft");
        }
        self.dco_templates.insert(id, req.clone());
        self.log_audit(
            user,
            AuditAction::Create,
            "dco_template",
            &id.to_string(),
            serde_json::json!({}),
        );
        req
    }

    pub fn delete_dco_template(&self, id: Uuid, user: &str) -> bool {
        let removed = self.dco_templates.remove(&id).is_some();
        if removed {
            self.log_audit(
                user,
                AuditAction::Delete,
                "dco_template",
                &id.to_string(),
                serde_json::json!({}),
            );
        }
        removed
    }

    // ─── CDP Platforms ──────────────────────────────────────────────────

    pub fn list_cdp_platforms(&self) -> Vec<serde_json::Value> {
        self.cdp_platforms
            .iter()
            .map(|r| r.value().clone())
            .collect()
    }

    pub fn get_cdp_sync_history(&self) -> Vec<serde_json::Value> {
        let mut history: Vec<serde_json::Value> = self
            .cdp_sync_history
            .iter()
            .map(|r| r.value().clone())
            .collect();
        history.sort_by(|a, b| {
            let a_date = a.get("started_at").and_then(|v| v.as_str()).unwrap_or("");
            let b_date = b.get("started_at").and_then(|v| v.as_str()).unwrap_or("");
            b_date.cmp(a_date)
        });
        history
    }

    // ─── Experiments ────────────────────────────────────────────────────

    pub fn list_experiments(&self) -> Vec<serde_json::Value> {
        self.experiments.iter().map(|r| r.value().clone()).collect()
    }

    pub fn get_experiment(&self, id: Uuid) -> Option<serde_json::Value> {
        self.experiments.get(&id).map(|r| r.value().clone())
    }

    pub fn create_experiment(&self, mut req: serde_json::Value, user: &str) -> serde_json::Value {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        req["id"] = serde_json::json!(id);
        req["created_at"] = serde_json::json!(now);
        req["updated_at"] = serde_json::json!(now);
        if req.get("status").is_none() {
            req["status"] = serde_json::json!("draft");
        }
        self.experiments.insert(id, req.clone());
        self.log_audit(
            user,
            AuditAction::Create,
            "experiment",
            &id.to_string(),
            serde_json::json!({}),
        );
        req
    }

    // ─── Demo Data ─────────────────────────────────────────────────────────

    fn seed_demo_data(&self) {
        use chrono::Duration;
        let now = Utc::now();

        // Demo campaigns
        let campaigns = vec![
            (
                "Holiday Season Push",
                CampaignStatus::Active,
                50000.0,
                2500.0,
                1_250_000,
                37_500,
                625,
                18_750.0,
            ),
            (
                "Back to School",
                CampaignStatus::Active,
                25000.0,
                1200.0,
                890_000,
                26_700,
                445,
                12_450.0,
            ),
            (
                "Summer Clearance",
                CampaignStatus::Completed,
                15000.0,
                750.0,
                2_100_000,
                63_000,
                1050,
                14_800.0,
            ),
            (
                "New User Acquisition",
                CampaignStatus::Active,
                75000.0,
                3500.0,
                3_400_000,
                85_000,
                1700,
                42_500.0,
            ),
            (
                "VIP Loyalty Rewards",
                CampaignStatus::Active,
                10000.0,
                500.0,
                450_000,
                22_500,
                900,
                6_750.0,
            ),
            (
                "Flash Sale Weekend",
                CampaignStatus::Paused,
                8000.0,
                4000.0,
                320_000,
                12_800,
                384,
                4_200.0,
            ),
            (
                "Brand Awareness Q1",
                CampaignStatus::Draft,
                30000.0,
                1500.0,
                0,
                0,
                0,
                0.0,
            ),
        ];

        for (name, status, budget, daily, imps, clicks, convs, spend) in campaigns {
            let id = Uuid::new_v4();
            let ctr = if imps > 0 {
                clicks as f64 / imps as f64
            } else {
                0.0
            };
            let hourly: Vec<HourlyDataPoint> = (0..24)
                .map(|h| HourlyDataPoint {
                    hour: now - Duration::hours(24 - h),
                    impressions: if status == CampaignStatus::Active {
                        imps / 24 + (h as u64 * 100)
                    } else {
                        0
                    },
                    clicks: if status == CampaignStatus::Active {
                        clicks / 24 + (h as u64 * 3)
                    } else {
                        0
                    },
                    spend: if status == CampaignStatus::Active {
                        spend / 24.0
                    } else {
                        0.0
                    },
                })
                .collect();

            self.campaigns.insert(
                id,
                Campaign {
                    id,
                    name: name.to_string(),
                    status,
                    budget,
                    daily_budget: daily,
                    pacing: PacingStrategy::Even,
                    targeting: TargetingConfig {
                        geo_regions: vec!["US".into(), "CA".into()],
                        segments: vec![100, 200, 300],
                        devices: vec!["mobile".into(), "desktop".into()],
                        floor_price: 0.50,
                        max_bid: Some(5.0),
                        frequency_cap_hourly: Some(10),
                        frequency_cap_daily: Some(50),
                        loyalty_tiers: vec!["gold".into(), "reserve".into()],
                        dsp_platforms: vec!["google_dv360".into(), "the_trade_desk".into()],
                    },
                    schedule_start: Some(now - Duration::days(30)),
                    schedule_end: Some(now + Duration::days(30)),
                    created_at: now - Duration::days(30),
                    updated_at: now,
                    stats: CampaignStats {
                        impressions: imps,
                        clicks,
                        conversions: convs,
                        spend,
                        ctr,
                        win_rate: 0.32,
                        avg_bid: 2.50,
                        avg_win_price: 1.85,
                        hourly_data: hourly,
                    },
                },
            );

            // Add creatives for active campaigns
            if status == CampaignStatus::Active {
                for (i, format) in [
                    (CreativeFormat::Banner, 300, 250),
                    (CreativeFormat::Banner, 728, 90),
                    (CreativeFormat::Native, 600, 400),
                ]
                .iter()
                .enumerate()
                {
                    let cid = Uuid::new_v4();
                    self.creatives.insert(
                        cid,
                        Creative {
                            id: cid,
                            campaign_id: id,
                            name: format!("{} - Creative {}", name, i + 1),
                            format: format.0,
                            asset_url: format!(
                                "https://cdn.campaignexpress.io/creatives/{}/{}.png",
                                id, cid
                            ),
                            width: format.1,
                            height: format.2,
                            status: CreativeStatus::Active,
                            metadata: serde_json::json!({"variant": format!("v{}", i + 1)}),
                            created_at: now - Duration::days(28),
                            updated_at: now,
                        },
                    );
                }
            }
        }
    }

    fn seed_journey_data(&self) {
        use chrono::Duration;
        let now = Utc::now();
        let journeys = vec![
            (
                "Welcome Series",
                "active",
                "event_based",
                "Multi-step email welcome flow for new users",
                5,
            ),
            (
                "Cart Abandonment Recovery",
                "active",
                "event_based",
                "Push + email for users who abandon cart",
                4,
            ),
            (
                "Loyalty Re-engagement",
                "active",
                "segment_entry",
                "Multi-channel re-engage for dormant loyalty members",
                6,
            ),
            (
                "VIP Birthday Reward",
                "paused",
                "schedule_based",
                "Personalized birthday offers for VIP tier",
                3,
            ),
            (
                "Post-Purchase Upsell",
                "draft",
                "event_based",
                "Cross-sell journey triggered after purchase",
                4,
            ),
        ];
        for (name, status, trigger_type, desc, steps) in journeys {
            let id = Uuid::new_v4();
            let step_list: Vec<serde_json::Value> = (0..steps).map(|i| {
                serde_json::json!({
                    "id": Uuid::new_v4(),
                    "step_type": if i == 0 { "action" } else if i % 2 == 0 { "wait" } else { "action" },
                    "config": {},
                    "position": i,
                    "next_steps": []
                })
            }).collect();
            self.journeys.insert(
                id,
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "description": desc,
                    "status": status,
                    "trigger": { "type": trigger_type, "config": {} },
                    "steps": step_list,
                    "version": 1,
                    "created_at": (now - Duration::days(15)).to_rfc3339(),
                    "updated_at": now.to_rfc3339(),
                }),
            );
        }
    }

    fn seed_dco_data(&self) {
        let now = Utc::now();
        let templates = vec![
            (
                "Holiday Banner DCO",
                "Dynamic holiday banner with personalized offers",
                "active",
            ),
            (
                "Product Recommendation",
                "AI-selected product images with dynamic pricing",
                "active",
            ),
            (
                "Retargeting Creative",
                "Personalized retargeting ads with last-viewed items",
                "draft",
            ),
        ];
        for (name, desc, status) in templates {
            let id = Uuid::new_v4();
            let components: Vec<serde_json::Value> = vec![
                serde_json::json!({
                    "id": Uuid::new_v4(), "component_type": "headline",
                    "required": true,
                    "variants": [
                        {"id": Uuid::new_v4(), "name": "Urgency", "content": "Limited Time Offer!", "performance": {"impressions": 45000, "clicks": 2700, "conversions": 135, "ctr": 0.06, "cvr": 0.003, "revenue": 8100.0}},
                        {"id": Uuid::new_v4(), "name": "Value", "content": "Save Up To 50%", "performance": {"impressions": 42000, "clicks": 3360, "conversions": 168, "ctr": 0.08, "cvr": 0.004, "revenue": 10080.0}},
                        {"id": Uuid::new_v4(), "name": "Personal", "content": "Picked Just For You", "performance": {"impressions": 38000, "clicks": 2660, "conversions": 152, "ctr": 0.07, "cvr": 0.004, "revenue": 9120.0}},
                    ]
                }),
                serde_json::json!({
                    "id": Uuid::new_v4(), "component_type": "hero_image",
                    "required": true,
                    "variants": [
                        {"id": Uuid::new_v4(), "name": "Lifestyle", "content": "lifestyle_hero.jpg", "asset_url": "https://cdn.example.com/lifestyle.jpg", "performance": {"impressions": 50000, "clicks": 3000, "conversions": 150, "ctr": 0.06, "cvr": 0.003, "revenue": 9000.0}},
                        {"id": Uuid::new_v4(), "name": "Product Focus", "content": "product_hero.jpg", "asset_url": "https://cdn.example.com/product.jpg", "performance": {"impressions": 48000, "clicks": 3840, "conversions": 192, "ctr": 0.08, "cvr": 0.004, "revenue": 11520.0}},
                    ]
                }),
                serde_json::json!({
                    "id": Uuid::new_v4(), "component_type": "cta",
                    "required": true,
                    "variants": [
                        {"id": Uuid::new_v4(), "name": "Shop Now", "content": "Shop Now", "performance": {"impressions": 60000, "clicks": 4200, "conversions": 210, "ctr": 0.07, "cvr": 0.0035, "revenue": 12600.0}},
                        {"id": Uuid::new_v4(), "name": "Learn More", "content": "Learn More", "performance": {"impressions": 55000, "clicks": 3300, "conversions": 165, "ctr": 0.06, "cvr": 0.003, "revenue": 9900.0}},
                    ]
                }),
            ];
            self.dco_templates.insert(
                id,
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "description": desc,
                    "status": status,
                    "components": components,
                    "rules": [],
                    "created_at": now.to_rfc3339(),
                    "updated_at": now.to_rfc3339(),
                }),
            );
        }
    }

    fn seed_cdp_data(&self) {
        use chrono::Duration;
        let now = Utc::now();
        let platforms = vec![
            (
                "salesforce",
                "salesforce_data_cloud",
                "https://api.salesforce.com/cdp/v1",
                true,
            ),
            (
                "adobe",
                "adobe_real_time_cdp",
                "https://platform.adobe.io/data/core",
                true,
            ),
            (
                "segment",
                "twilio_segment",
                "https://api.segment.io/v1",
                true,
            ),
            (
                "tealium",
                "tealium",
                "https://collect.tealiumiq.com/event",
                false,
            ),
            (
                "hightouch",
                "hightouch",
                "https://api.hightouch.com/v1",
                false,
            ),
        ];
        for (name, platform, endpoint, enabled) in &platforms {
            self.cdp_platforms.insert(
                name.to_string(),
                serde_json::json!({
                    "platform": platform,
                    "api_endpoint": endpoint,
                    "api_key": format!("cdp_{}_{}", name, "demo_key_xxx"),
                    "enabled": enabled,
                    "sync_interval_secs": 300,
                    "batch_size": 10000,
                    "field_mappings": {
                        "user_id": "external_id",
                        "email": "email_address",
                        "segments": "audience_segments",
                    }
                }),
            );
        }
        // Seed sync history
        for (i, (name, platform, _, _)) in platforms.iter().enumerate() {
            let sync_id = Uuid::new_v4();
            let status = if i % 3 == 2 { "failed" } else { "completed" };
            self.cdp_sync_history.insert(sync_id, serde_json::json!({
                "id": sync_id,
                "platform": platform,
                "platform_name": name,
                "direction": if i % 2 == 0 { "inbound" } else { "outbound" },
                "record_count": 5000 + i * 2500,
                "status": status,
                "started_at": (now - Duration::hours(i as i64 * 2)).to_rfc3339(),
                "completed_at": if status == "completed" { Some((now - Duration::hours(i as i64 * 2) + Duration::minutes(5)).to_rfc3339()) } else { None },
                "error": if status == "failed" { Some("Connection timeout after 30s") } else { None },
            }));
        }
    }

    fn seed_experiment_data(&self) {
        let now = Utc::now();
        let experiments = vec![
            (
                "Headline A/B Test",
                "running",
                "ctr",
                0.5,
                vec![
                    (
                        "Control - Standard",
                        true,
                        0.5,
                        12500,
                        375,
                        3750.0,
                        0.03,
                        0.0,
                        0.0,
                    ),
                    (
                        "Variant A - Urgency",
                        false,
                        0.5,
                        12500,
                        500,
                        5000.0,
                        0.04,
                        0.85,
                        33.3,
                    ),
                ],
            ),
            (
                "Bid Strategy Test",
                "running",
                "roi",
                0.3,
                vec![
                    (
                        "Control - Even Pacing",
                        true,
                        0.33,
                        8000,
                        240,
                        4800.0,
                        0.03,
                        0.0,
                        0.0,
                    ),
                    (
                        "Accelerated Pacing",
                        false,
                        0.33,
                        8000,
                        280,
                        5600.0,
                        0.035,
                        0.72,
                        16.7,
                    ),
                    (
                        "ML-Optimized Pacing",
                        false,
                        0.34,
                        8000,
                        320,
                        6400.0,
                        0.04,
                        0.94,
                        33.3,
                    ),
                ],
            ),
            (
                "DCO vs Static Creative",
                "completed",
                "conversions",
                1.0,
                vec![
                    (
                        "Control - Static",
                        true,
                        0.5,
                        25000,
                        750,
                        15000.0,
                        0.03,
                        0.0,
                        0.0,
                    ),
                    (
                        "DCO Dynamic",
                        false,
                        0.5,
                        25000,
                        1000,
                        20000.0,
                        0.04,
                        0.98,
                        33.3,
                    ),
                ],
            ),
            (
                "Channel Priority Test",
                "draft",
                "engagement",
                0.2,
                vec![
                    ("Control - Email First", true, 0.5, 0, 0, 0.0, 0.0, 0.0, 0.0),
                    ("Push First", false, 0.5, 0, 0, 0.0, 0.0, 0.0, 0.0),
                ],
            ),
        ];
        for (name, status, metric, traffic, variants) in experiments {
            let id = Uuid::new_v4();
            let variant_list: Vec<serde_json::Value> = variants
                .iter()
                .map(
                    |(vname, is_control, weight, samples, convs, rev, cvr, conf, lift)| {
                        serde_json::json!({
                            "id": Uuid::new_v4(),
                            "name": vname,
                            "is_control": is_control,
                            "weight": weight,
                            "config": {},
                            "results": {
                                "sample_size": samples,
                                "conversions": convs,
                                "revenue": rev,
                                "conversion_rate": cvr,
                                "confidence": conf,
                                "lift": lift,
                            }
                        })
                    },
                )
                .collect();
            self.experiments.insert(
                id,
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "description": format!("Experiment testing {} optimization", metric),
                    "status": status,
                    "metric": metric,
                    "traffic_allocation": traffic,
                    "min_sample_size": 10000,
                    "variants": variant_list,
                    "created_at": now.to_rfc3339(),
                    "updated_at": now.to_rfc3339(),
                }),
            );
        }
    }

    // ─── Users ─────────────────────────────────────────────────────────────

    pub fn list_users(&self) -> Vec<serde_json::Value> {
        let mut users: Vec<serde_json::Value> =
            self.users.iter().map(|r| r.value().clone()).collect();
        users.sort_by(|a, b| {
            let a_name = a.get("display_name").and_then(|v| v.as_str()).unwrap_or("");
            let b_name = b.get("display_name").and_then(|v| v.as_str()).unwrap_or("");
            a_name.cmp(b_name)
        });
        users
    }

    pub fn get_user(&self, id: Uuid) -> Option<serde_json::Value> {
        self.users.get(&id).map(|r| r.value().clone())
    }

    pub fn create_user(&self, mut req: serde_json::Value, actor: &str) -> serde_json::Value {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        req["id"] = serde_json::json!(id);
        req["created_at"] = serde_json::json!(now);
        if req.get("status").is_none() {
            req["status"] = serde_json::json!("active");
        }
        if req.get("auth_provider").is_none() {
            req["auth_provider"] = serde_json::json!("local");
        }
        self.users.insert(id, req.clone());
        self.log_audit(
            actor,
            AuditAction::Create,
            "user",
            &id.to_string(),
            serde_json::json!({"email": req.get("email")}),
        );
        req
    }

    pub fn disable_user(&self, id: Uuid, actor: &str) -> Option<serde_json::Value> {
        self.users.get_mut(&id).map(|mut entry| {
            entry.value_mut()["status"] = serde_json::json!("disabled");
            self.log_audit(
                actor,
                AuditAction::Update,
                "user",
                &id.to_string(),
                serde_json::json!({"action": "disable"}),
            );
            entry.value().clone()
        })
    }

    pub fn enable_user(&self, id: Uuid, actor: &str) -> Option<serde_json::Value> {
        self.users.get_mut(&id).map(|mut entry| {
            entry.value_mut()["status"] = serde_json::json!("active");
            self.log_audit(
                actor,
                AuditAction::Update,
                "user",
                &id.to_string(),
                serde_json::json!({"action": "enable"}),
            );
            entry.value().clone()
        })
    }

    pub fn delete_user(&self, id: Uuid, actor: &str) -> bool {
        let removed = self.users.remove(&id).is_some();
        if removed {
            self.log_audit(
                actor,
                AuditAction::Delete,
                "user",
                &id.to_string(),
                serde_json::json!({}),
            );
        }
        removed
    }

    pub fn update_user_role(&self, id: Uuid, role: &str, actor: &str) -> Option<serde_json::Value> {
        self.users.get_mut(&id).map(|mut entry| {
            entry.value_mut()["role"] = serde_json::json!(role);
            self.log_audit(
                actor,
                AuditAction::Update,
                "user",
                &id.to_string(),
                serde_json::json!({"action": "role_change", "role": role}),
            );
            entry.value().clone()
        })
    }

    // ─── Invitations ──────────────────────────────────────────────────────

    pub fn list_invitations(&self) -> Vec<serde_json::Value> {
        let mut invitations: Vec<serde_json::Value> =
            self.invitations.iter().map(|r| r.value().clone()).collect();
        invitations.sort_by(|a, b| {
            let a_date = a.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            let b_date = b.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            b_date.cmp(a_date)
        });
        invitations
    }

    pub fn create_invitation(&self, mut req: serde_json::Value, actor: &str) -> serde_json::Value {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let expires = now + chrono::Duration::days(7);
        req["id"] = serde_json::json!(id);
        req["status"] = serde_json::json!("pending");
        req["created_at"] = serde_json::json!(now.to_rfc3339());
        req["expires_at"] = serde_json::json!(expires.to_rfc3339());
        self.invitations.insert(id, req.clone());
        self.log_audit(
            actor,
            AuditAction::Create,
            "invitation",
            &id.to_string(),
            serde_json::json!({"email": req.get("email")}),
        );
        req
    }

    pub fn revoke_invitation(&self, id: Uuid, actor: &str) -> bool {
        let updated = self.invitations.get_mut(&id).map(|mut entry| {
            entry.value_mut()["status"] = serde_json::json!("revoked");
        });
        if updated.is_some() {
            self.log_audit(
                actor,
                AuditAction::Delete,
                "invitation",
                &id.to_string(),
                serde_json::json!({}),
            );
        }
        updated.is_some()
    }

    // ─── Platform: Tenants ────────────────────────────────────────────────

    pub fn list_tenants(&self) -> Vec<serde_json::Value> {
        self.tenants.iter().map(|r| r.value().clone()).collect()
    }

    pub fn list_roles(&self) -> Vec<serde_json::Value> {
        self.roles.iter().map(|r| r.value().clone()).collect()
    }

    pub fn get_compliance_status(&self) -> Vec<serde_json::Value> {
        self.compliance.iter().map(|r| r.value().clone()).collect()
    }

    pub fn list_dsrs(&self) -> Vec<serde_json::Value> {
        self.dsrs.iter().map(|r| r.value().clone()).collect()
    }

    // ─── Billing ────────────────────────────────────────────────────────────

    pub fn list_plans(&self) -> Vec<serde_json::Value> {
        let mut plans: Vec<serde_json::Value> =
            self.plans.iter().map(|r| r.value().clone()).collect();
        plans.sort_by(|a, b| {
            let pa = a["monthly_price"].as_f64().unwrap_or(0.0);
            let pb = b["monthly_price"].as_f64().unwrap_or(0.0);
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        });
        plans
    }

    pub fn get_subscription(&self, tenant_id: Uuid) -> Option<serde_json::Value> {
        self.subscriptions
            .iter()
            .find(|r| r.value()["tenant_id"].as_str() == Some(&tenant_id.to_string()))
            .map(|r| r.value().clone())
    }

    pub fn list_invoices(&self) -> Vec<serde_json::Value> {
        self.invoices.iter().map(|r| r.value().clone()).collect()
    }

    pub fn get_usage_summary(&self, tenant_id: Uuid) -> serde_json::Value {
        serde_json::json!({
            "tenant_id": tenant_id,
            "period": "2026-02",
            "meters": [
                {"meter_type": "offers_served", "total_quantity": 2_450_000u64, "unit_price": 0.00001, "line_total": 24.50, "quota": 5_000_000u64, "usage_percent": 49.0},
                {"meter_type": "api_calls", "total_quantity": 125_000u64, "unit_price": 0.000005, "line_total": 0.63, "quota": 500_000u64, "usage_percent": 25.0},
                {"meter_type": "campaigns_active", "total_quantity": 12u64, "unit_price": 0.0, "line_total": 0.0, "quota": 100u64, "usage_percent": 12.0},
            ],
            "total_cost": 25.13
        })
    }

    pub fn get_onboarding_progress(&self, tenant_id: Uuid) -> serde_json::Value {
        serde_json::json!({
            "tenant_id": tenant_id,
            "steps": [
                {"id": "account_setup", "title": "Account Setup", "description": "Configure organization details", "status": "completed", "order": 1, "required": true},
                {"id": "team_invite", "title": "Invite Team Members", "description": "Add team members and assign roles", "status": "completed", "order": 2, "required": false},
                {"id": "first_campaign", "title": "Create First Campaign", "description": "Launch your first campaign", "status": "in_progress", "order": 3, "required": true},
                {"id": "connect_dsp", "title": "Connect DSP", "description": "Integrate with demand-side platforms", "status": "not_started", "order": 4, "required": false},
                {"id": "configure_channels", "title": "Configure Channels", "description": "Set up email, push, SMS channels", "status": "not_started", "order": 5, "required": true},
                {"id": "install_pixel", "title": "Install Tracking Pixel", "description": "Add tracking to your website", "status": "not_started", "order": 6, "required": false},
                {"id": "launch_campaign", "title": "Launch Campaign", "description": "Go live with your first campaign", "status": "not_started", "order": 7, "required": true},
            ],
            "started_at": "2026-02-01T00:00:00Z",
            "completed_at": null,
            "completion_percent": 28.6
        })
    }

    // ─── Ops ────────────────────────────────────────────────────────────────

    pub fn get_status_page(&self) -> serde_json::Value {
        let components: Vec<serde_json::Value> = self
            .status_components
            .iter()
            .map(|r| r.value().clone())
            .collect();
        serde_json::json!({
            "overall_status": "operational",
            "components": components,
            "updated_at": Utc::now().to_rfc3339()
        })
    }

    pub fn list_incidents(&self) -> Vec<serde_json::Value> {
        self.incidents.iter().map(|r| r.value().clone()).collect()
    }

    pub fn get_sla_report(&self) -> serde_json::Value {
        let targets: Vec<serde_json::Value> =
            self.sla_targets.iter().map(|r| r.value().clone()).collect();
        serde_json::json!({
            "report_period": "2026-02",
            "targets": targets,
            "overall_uptime": 99.97
        })
    }

    pub fn list_backups(&self) -> Vec<serde_json::Value> {
        self.backup_schedules
            .iter()
            .map(|r| r.value().clone())
            .collect()
    }

    // ─── Seed: Platform ─────────────────────────────────────────────────────

    fn seed_platform_data(&self) {
        let now = Utc::now();
        // Tenants
        let tiers = vec![
            ("Acme Corp", "acme-corp", "active", "professional"),
            ("StartupXYZ", "startupxyz", "trial", "starter"),
            (
                "Enterprise Global",
                "enterprise-global",
                "active",
                "enterprise",
            ),
        ];
        for (name, slug, status, tier) in tiers {
            let id = Uuid::new_v4();
            self.tenants.insert(
                id,
                serde_json::json!({
                    "id": id, "name": name, "slug": slug, "status": status,
                    "pricing_tier": tier, "owner_id": Uuid::new_v4(),
                    "settings": {
                        "max_campaigns": if tier == "enterprise" { 0 } else { 100 },
                        "max_users": if tier == "enterprise" { 0 } else { 25 },
                        "max_offers_per_hour": if tier == "enterprise" { 0 } else { 5_000_000u64 },
                        "max_api_calls_per_day": if tier == "enterprise" { 0 } else { 500_000u64 },
                        "features_enabled": ["journeys", "dco", "experiments"],
                        "data_retention_days": 365
                    },
                    "usage": {
                        "campaigns_active": 12, "users_count": 8,
                        "offers_served_today": 1_250_000u64,
                        "api_calls_today": 45_000u64, "storage_bytes": 2_500_000_000u64
                    },
                    "created_at": now.to_rfc3339(), "updated_at": now.to_rfc3339()
                }),
            );
        }

        // Roles
        let roles_data = vec![
            ("Admin", "Full access to all features", vec!["*"], true),
            (
                "Campaign Manager",
                "Create, edit, and manage campaigns and creatives",
                vec![
                    "campaign_read",
                    "campaign_write",
                    "campaign_delete",
                    "creative_read",
                    "creative_write",
                    "journey_read",
                    "journey_write",
                    "analytics_read",
                ],
                true,
            ),
            (
                "Analyst",
                "View-only access to analytics and experiments",
                vec!["analytics_read", "experiment_read", "campaign_read"],
                true,
            ),
            (
                "Viewer",
                "Read-only access to all resources",
                vec![
                    "campaign_read",
                    "creative_read",
                    "journey_read",
                    "experiment_read",
                    "dco_read",
                    "cdp_read",
                    "analytics_read",
                ],
                true,
            ),
        ];
        for (name, desc, perms, is_system) in roles_data {
            let id = Uuid::new_v4();
            self.roles.insert(
                id,
                serde_json::json!({
                    "id": id, "name": name, "description": desc,
                    "permissions": perms, "is_system": is_system,
                    "created_at": now.to_rfc3339()
                }),
            );
        }

        // Compliance
        let compliance_data = vec![
            ("gdpr", "compliant", "2025-11-15"),
            ("ccpa", "compliant", "2025-12-01"),
            ("soc2", "in_progress", ""),
            ("iso27001", "planned", ""),
        ];
        for (framework, status, last_audit) in compliance_data {
            self.compliance.insert(
                framework.to_string(),
                serde_json::json!({
                    "framework": framework, "status": status,
                    "last_audit": if last_audit.is_empty() { serde_json::Value::Null } else { serde_json::json!(format!("{}T00:00:00Z", last_audit)) },
                    "next_audit": null,
                    "findings": []
                }),
            );
        }

        // DSRs
        let dsr_types = vec![
            ("user-1234@email.com", "erasure", "completed"),
            ("user-5678@email.com", "access", "completed"),
            ("user-9012@email.com", "erasure", "pending"),
        ];
        for (user, req_type, status) in dsr_types {
            let id = Uuid::new_v4();
            self.dsrs.insert(
                id,
                serde_json::json!({
                    "id": id, "tenant_id": Uuid::new_v4(),
                    "user_identifier": user, "request_type": req_type,
                    "status": status, "requested_at": now.to_rfc3339(),
                    "completed_at": if status == "completed" { serde_json::json!(now.to_rfc3339()) } else { serde_json::Value::Null }
                }),
            );
        }
    }

    // ─── Seed: Billing ──────────────────────────────────────────────────────

    fn seed_billing_data(&self) {
        let now = Utc::now();
        // Plans
        let plans_data = vec![
            (
                "Free",
                "free",
                0.0,
                0.0,
                1000u64,
                10_000u64,
                vec!["Basic campaigns", "Email channel"],
            ),
            (
                "Starter",
                "starter",
                99.0,
                990.0,
                100_000,
                100_000,
                vec!["Multi-channel", "Basic journeys", "5 users"],
            ),
            (
                "Professional",
                "professional",
                499.0,
                4990.0,
                5_000_000,
                500_000,
                vec![
                    "All channels",
                    "Advanced journeys",
                    "DCO",
                    "Experiments",
                    "25 users",
                ],
            ),
            (
                "Enterprise",
                "enterprise",
                1999.0,
                19990.0,
                0,
                0,
                vec![
                    "Unlimited everything",
                    "Dedicated support",
                    "SLA guarantee",
                    "Custom integrations",
                    "SSO/SAML",
                ],
            ),
        ];
        for (name, tier, monthly, annual, offers, api_calls, features) in plans_data {
            let id = Uuid::new_v4();
            self.plans.insert(
                id,
                serde_json::json!({
                    "id": id, "name": name, "tier": tier,
                    "monthly_price": monthly, "annual_price": annual,
                    "included_offers": offers, "included_api_calls": api_calls,
                    "features": features
                }),
            );
        }

        // Invoices
        let invoice_data = vec![
            (499.0, "paid", 3),
            (499.0, "paid", 2),
            (523.50, "pending", 4),
        ];
        for (amount, status, items) in invoice_data {
            let id = Uuid::new_v4();
            let line_items: Vec<serde_json::Value> = (0..items)
                .map(|i| {
                    serde_json::json!({
                        "description": format!("Line item {}", i + 1),
                        "quantity": 1, "unit_price": amount / items as f64,
                        "amount": amount / items as f64
                    })
                })
                .collect();
            self.invoices.insert(
                id,
                serde_json::json!({
                    "id": id, "tenant_id": Uuid::new_v4(),
                    "subscription_id": Uuid::new_v4(),
                    "amount": amount, "currency": "USD", "status": status,
                    "line_items": line_items,
                    "issued_at": now.to_rfc3339(),
                    "due_at": (now + chrono::Duration::days(30)).to_rfc3339(),
                    "paid_at": if status == "paid" { serde_json::json!(now.to_rfc3339()) } else { serde_json::Value::Null }
                }),
            );
        }
    }

    // ─── Seed: Ops ──────────────────────────────────────────────────────────

    fn seed_ops_data(&self) {
        let now = Utc::now();
        // Status components
        let components = vec![
            (
                "API Gateway",
                "All endpoints responding normally",
                "operational",
                "Core",
            ),
            (
                "Bidding Engine",
                "Processing bids at target throughput",
                "operational",
                "Core",
            ),
            (
                "NATS Cluster",
                "Message queue healthy",
                "operational",
                "Infrastructure",
            ),
            (
                "Redis Cluster",
                "Cache layer operational",
                "operational",
                "Infrastructure",
            ),
            (
                "ClickHouse",
                "Analytics DB accepting writes",
                "operational",
                "Infrastructure",
            ),
            (
                "NPU Engine",
                "ML inference operational",
                "operational",
                "Core",
            ),
            (
                "Management UI",
                "Dashboard accessible",
                "operational",
                "Frontend",
            ),
        ];
        for (name, desc, status, group) in components {
            let id = Uuid::new_v4();
            self.status_components.insert(
                id,
                serde_json::json!({
                    "id": id, "name": name, "description": desc,
                    "status": status, "group": group,
                    "updated_at": now.to_rfc3339()
                }),
            );
        }

        // SLA targets
        let sla_data = vec![
            ("API Availability", 99.9, 99.97),
            ("Bid Latency p99 < 10ms", 99.5, 99.82),
            ("Data Pipeline", 99.9, 99.95),
        ];
        for (name, target, current) in sla_data {
            self.sla_targets.insert(
                name.to_string(),
                serde_json::json!({
                    "name": name, "target_percent": target,
                    "current_percent": current,
                    "measurement_window": "30 days",
                    "last_incident": null
                }),
            );
        }

        // Backup schedules
        let backups = vec![
            ("redis", "0 */6 * * *", 7, true),
            ("clickhouse", "0 2 * * *", 30, true),
            ("configs", "0 0 * * *", 90, true),
            ("models", "0 0 * * 0", 60, true),
        ];
        for (target, cron, retention, enabled) in backups {
            let id = Uuid::new_v4();
            self.backup_schedules.insert(
                id,
                serde_json::json!({
                    "id": id, "target": target,
                    "cron_expression": cron, "retention_days": retention,
                    "enabled": enabled,
                    "last_run": now.to_rfc3339(),
                    "next_run": (now + chrono::Duration::hours(6)).to_rfc3339()
                }),
            );
        }

        // Sample resolved incident
        let inc_id = Uuid::new_v4();
        self.incidents.insert(
            inc_id,
            serde_json::json!({
                "id": inc_id,
                "title": "Elevated bid latency on nodes 12-15",
                "description": "NPU inference latency exceeded 10ms threshold",
                "severity": "minor", "status": "resolved",
                "affected_components": ["Bidding Engine", "NPU Engine"],
                "created_at": (now - chrono::Duration::days(3)).to_rfc3339(),
                "resolved_at": (now - chrono::Duration::days(3) + chrono::Duration::hours(2)).to_rfc3339()
            }),
        );
    }

    // ─── Seed: Users ─────────────────────────────────────────────────────

    fn seed_user_data(&self) {
        let now = Utc::now();
        let users_data = vec![
            (
                "admin@campaignexpress.io",
                "Platform Admin",
                "active",
                "Admin",
                "local",
            ),
            (
                "sarah.chen@acme.com",
                "Sarah Chen",
                "active",
                "Campaign Manager",
                "oauth2",
            ),
            (
                "mike.johnson@acme.com",
                "Mike Johnson",
                "active",
                "Analyst",
                "oauth2",
            ),
            (
                "emily.davis@acme.com",
                "Emily Davis",
                "active",
                "Campaign Manager",
                "saml",
            ),
            (
                "james.wilson@acme.com",
                "James Wilson",
                "active",
                "Viewer",
                "local",
            ),
            (
                "lisa.park@acme.com",
                "Lisa Park",
                "disabled",
                "Analyst",
                "oauth2",
            ),
        ];
        for (email, name, status, role, provider) in users_data {
            let id = Uuid::new_v4();
            self.users.insert(
                id,
                serde_json::json!({
                    "id": id,
                    "email": email,
                    "display_name": name,
                    "status": status,
                    "role": role,
                    "auth_provider": provider,
                    "created_at": (now - chrono::Duration::days(30)).to_rfc3339(),
                    "last_login": if status == "active" { serde_json::json!((now - chrono::Duration::hours(2)).to_rfc3339()) } else { serde_json::Value::Null },
                }),
            );
        }

        // Pending invitation
        let inv_id = Uuid::new_v4();
        self.invitations.insert(
            inv_id,
            serde_json::json!({
                "id": inv_id,
                "email": "new.hire@acme.com",
                "role": "Campaign Manager",
                "status": "pending",
                "invited_by": "admin@campaignexpress.io",
                "created_at": (now - chrono::Duration::days(1)).to_rfc3339(),
                "expires_at": (now + chrono::Duration::days(6)).to_rfc3339(),
            }),
        );
    }
}

impl Default for ManagementStore {
    fn default() -> Self {
        Self::new()
    }
}
