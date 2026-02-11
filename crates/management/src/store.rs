//! In-memory management store backed by DashMap.
//!
//! Production: replace with PostgreSQL (sqlx) or similar ACID store.
//! This provides the same API surface for development and testing.

use crate::models::*;
use chrono::Utc;
use dashmap::DashMap;
use tracing::info;
use uuid::Uuid;

/// Thread-safe in-memory store for campaigns, creatives, journeys, DCO, CDP, experiments, and audit log.
pub struct ManagementStore {
    campaigns: DashMap<Uuid, Campaign>,
    creatives: DashMap<Uuid, Creative>,
    journeys: DashMap<Uuid, serde_json::Value>,
    dco_templates: DashMap<Uuid, serde_json::Value>,
    cdp_platforms: DashMap<String, serde_json::Value>,
    cdp_sync_history: DashMap<Uuid, serde_json::Value>,
    experiments: DashMap<Uuid, serde_json::Value>,
    audit_log: DashMap<Uuid, AuditLogEntry>,
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
        };
        store.seed_demo_data();
        store.seed_journey_data();
        store.seed_dco_data();
        store.seed_cdp_data();
        store.seed_experiment_data();
        store
    }

    // ─── Campaigns ─────────────────────────────────────────────────────────

    pub fn list_campaigns(&self) -> Vec<Campaign> {
        let mut campaigns: Vec<Campaign> = self.campaigns.iter().map(|r| r.value().clone()).collect();
        campaigns.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        campaigns
    }

    pub fn get_campaign(&self, id: Uuid) -> Option<Campaign> {
        self.campaigns.get(&id).map(|r| r.value().clone())
    }

    pub fn create_campaign(&self, req: CreateCampaignRequest, user: &str) -> Campaign {
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
        self.log_audit(user, AuditAction::Create, "campaign", &id.to_string(), serde_json::json!({"name": &campaign.name}));
        campaign
    }

    pub fn update_campaign(&self, id: Uuid, req: UpdateCampaignRequest, user: &str) -> Option<Campaign> {
        self.campaigns.get_mut(&id).map(|mut entry| {
            let c = entry.value_mut();
            if let Some(name) = req.name { c.name = name; }
            if let Some(budget) = req.budget { c.budget = budget; }
            if let Some(daily_budget) = req.daily_budget { c.daily_budget = daily_budget; }
            if let Some(pacing) = req.pacing { c.pacing = pacing; }
            if let Some(targeting) = req.targeting { c.targeting = targeting; }
            if let Some(start) = req.schedule_start { c.schedule_start = Some(start); }
            if let Some(end) = req.schedule_end { c.schedule_end = Some(end); }
            c.updated_at = Utc::now();
            self.log_audit(user, AuditAction::Update, "campaign", &id.to_string(), serde_json::json!({}));
            c.clone()
        })
    }

    pub fn delete_campaign(&self, id: Uuid, user: &str) -> bool {
        let removed = self.campaigns.remove(&id).is_some();
        if removed {
            // Also remove associated creatives
            let creative_ids: Vec<Uuid> = self.creatives.iter()
                .filter(|r| r.value().campaign_id == id)
                .map(|r| r.key().clone())
                .collect();
            for cid in creative_ids {
                self.creatives.remove(&cid);
            }
            self.log_audit(user, AuditAction::Delete, "campaign", &id.to_string(), serde_json::json!({}));
        }
        removed
    }

    pub fn pause_campaign(&self, id: Uuid, user: &str) -> Option<Campaign> {
        self.campaigns.get_mut(&id).map(|mut entry| {
            entry.value_mut().status = CampaignStatus::Paused;
            entry.value_mut().updated_at = Utc::now();
            self.log_audit(user, AuditAction::Pause, "campaign", &id.to_string(), serde_json::json!({}));
            entry.value().clone()
        })
    }

    pub fn resume_campaign(&self, id: Uuid, user: &str) -> Option<Campaign> {
        self.campaigns.get_mut(&id).map(|mut entry| {
            entry.value_mut().status = CampaignStatus::Active;
            entry.value_mut().updated_at = Utc::now();
            self.log_audit(user, AuditAction::Resume, "campaign", &id.to_string(), serde_json::json!({}));
            entry.value().clone()
        })
    }

    // ─── Creatives ─────────────────────────────────────────────────────────

    pub fn list_creatives(&self) -> Vec<Creative> {
        let mut creatives: Vec<Creative> = self.creatives.iter().map(|r| r.value().clone()).collect();
        creatives.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        creatives
    }

    pub fn get_creative(&self, id: Uuid) -> Option<Creative> {
        self.creatives.get(&id).map(|r| r.value().clone())
    }

    pub fn create_creative(&self, req: CreateCreativeRequest, user: &str) -> Creative {
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
        self.log_audit(user, AuditAction::Create, "creative", &id.to_string(), serde_json::json!({"name": &creative.name}));
        creative
    }

    pub fn update_creative(&self, id: Uuid, req: UpdateCreativeRequest, user: &str) -> Option<Creative> {
        self.creatives.get_mut(&id).map(|mut entry| {
            let c = entry.value_mut();
            if let Some(name) = req.name { c.name = name; }
            if let Some(format) = req.format { c.format = format; }
            if let Some(url) = req.asset_url { c.asset_url = url; }
            if let Some(w) = req.width { c.width = w; }
            if let Some(h) = req.height { c.height = h; }
            if let Some(status) = req.status { c.status = status; }
            if let Some(meta) = req.metadata { c.metadata = meta; }
            c.updated_at = Utc::now();
            self.log_audit(user, AuditAction::Update, "creative", &id.to_string(), serde_json::json!({}));
            c.clone()
        })
    }

    pub fn delete_creative(&self, id: Uuid, user: &str) -> bool {
        let removed = self.creatives.remove(&id).is_some();
        if removed {
            self.log_audit(user, AuditAction::Delete, "creative", &id.to_string(), serde_json::json!({}));
        }
        removed
    }

    // ─── Monitoring ────────────────────────────────────────────────────────

    pub fn get_monitoring_overview(&self) -> MonitoringOverview {
        let total = self.campaigns.len() as u64;
        let active = self.campaigns.iter()
            .filter(|r| r.value().status == CampaignStatus::Active)
            .count() as u64;
        let total_impressions: u64 = self.campaigns.iter().map(|r| r.value().stats.impressions).sum();
        let total_clicks: u64 = self.campaigns.iter().map(|r| r.value().stats.clicks).sum();
        let total_spend: f64 = self.campaigns.iter().map(|r| r.value().stats.spend).sum();
        let avg_ctr = if total_impressions > 0 {
            total_clicks as f64 / total_impressions as f64
        } else { 0.0 };

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
        self.campaigns.get(&campaign_id).map(|r| r.value().stats.clone())
    }

    // ─── Audit Log ─────────────────────────────────────────────────────────

    pub fn get_audit_log(&self) -> Vec<AuditLogEntry> {
        let mut entries: Vec<AuditLogEntry> = self.audit_log.iter().map(|r| r.value().clone()).collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    fn log_audit(&self, user: &str, action: AuditAction, resource_type: &str, resource_id: &str, details: serde_json::Value) {
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
        let mut journeys: Vec<serde_json::Value> = self.journeys.iter().map(|r| r.value().clone()).collect();
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
        if req.get("status").is_none() { req["status"] = serde_json::json!("draft"); }
        if req.get("version").is_none() { req["version"] = serde_json::json!(1); }
        self.journeys.insert(id, req.clone());
        self.log_audit(user, AuditAction::Create, "journey", &id.to_string(), serde_json::json!({}));
        req
    }

    pub fn delete_journey(&self, id: Uuid, user: &str) -> bool {
        let removed = self.journeys.remove(&id).is_some();
        if removed {
            self.log_audit(user, AuditAction::Delete, "journey", &id.to_string(), serde_json::json!({}));
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
        self.dco_templates.iter().map(|r| r.value().clone()).collect()
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
        if req.get("status").is_none() { req["status"] = serde_json::json!("draft"); }
        self.dco_templates.insert(id, req.clone());
        self.log_audit(user, AuditAction::Create, "dco_template", &id.to_string(), serde_json::json!({}));
        req
    }

    pub fn delete_dco_template(&self, id: Uuid, user: &str) -> bool {
        let removed = self.dco_templates.remove(&id).is_some();
        if removed {
            self.log_audit(user, AuditAction::Delete, "dco_template", &id.to_string(), serde_json::json!({}));
        }
        removed
    }

    // ─── CDP Platforms ──────────────────────────────────────────────────

    pub fn list_cdp_platforms(&self) -> Vec<serde_json::Value> {
        self.cdp_platforms.iter().map(|r| r.value().clone()).collect()
    }

    pub fn get_cdp_sync_history(&self) -> Vec<serde_json::Value> {
        let mut history: Vec<serde_json::Value> = self.cdp_sync_history.iter().map(|r| r.value().clone()).collect();
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
        if req.get("status").is_none() { req["status"] = serde_json::json!("draft"); }
        self.experiments.insert(id, req.clone());
        self.log_audit(user, AuditAction::Create, "experiment", &id.to_string(), serde_json::json!({}));
        req
    }

    // ─── Demo Data ─────────────────────────────────────────────────────────

    fn seed_demo_data(&self) {
        use chrono::Duration;
        let now = Utc::now();

        // Demo campaigns
        let campaigns = vec![
            ("Holiday Season Push", CampaignStatus::Active, 50000.0, 2500.0, 1_250_000, 37_500, 625, 18_750.0),
            ("Back to School", CampaignStatus::Active, 25000.0, 1200.0, 890_000, 26_700, 445, 12_450.0),
            ("Summer Clearance", CampaignStatus::Completed, 15000.0, 750.0, 2_100_000, 63_000, 1050, 14_800.0),
            ("New User Acquisition", CampaignStatus::Active, 75000.0, 3500.0, 3_400_000, 85_000, 1700, 42_500.0),
            ("VIP Loyalty Rewards", CampaignStatus::Active, 10000.0, 500.0, 450_000, 22_500, 900, 6_750.0),
            ("Flash Sale Weekend", CampaignStatus::Paused, 8000.0, 4000.0, 320_000, 12_800, 384, 4_200.0),
            ("Brand Awareness Q1", CampaignStatus::Draft, 30000.0, 1500.0, 0, 0, 0, 0.0),
        ];

        for (name, status, budget, daily, imps, clicks, convs, spend) in campaigns {
            let id = Uuid::new_v4();
            let ctr = if imps > 0 { clicks as f64 / imps as f64 } else { 0.0 };
            let hourly: Vec<HourlyDataPoint> = (0..24).map(|h| {
                HourlyDataPoint {
                    hour: now - Duration::hours(24 - h),
                    impressions: if status == CampaignStatus::Active { imps / 24 + (h as u64 * 100) } else { 0 },
                    clicks: if status == CampaignStatus::Active { clicks / 24 + (h as u64 * 3) } else { 0 },
                    spend: if status == CampaignStatus::Active { spend / 24.0 } else { 0.0 },
                }
            }).collect();

            self.campaigns.insert(id, Campaign {
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
            });

            // Add creatives for active campaigns
            if status == CampaignStatus::Active {
                for (i, format) in [(CreativeFormat::Banner, 300, 250), (CreativeFormat::Banner, 728, 90), (CreativeFormat::Native, 600, 400)].iter().enumerate() {
                    let cid = Uuid::new_v4();
                    self.creatives.insert(cid, Creative {
                        id: cid,
                        campaign_id: id,
                        name: format!("{} - Creative {}", name, i + 1),
                        format: format.0,
                        asset_url: format!("https://cdn.campaignexpress.io/creatives/{}/{}.png", id, cid),
                        width: format.1,
                        height: format.2,
                        status: CreativeStatus::Active,
                        metadata: serde_json::json!({"variant": format!("v{}", i + 1)}),
                        created_at: now - Duration::days(28),
                        updated_at: now,
                    });
                }
            }
        }
    }

    fn seed_journey_data(&self) {
        use chrono::Duration;
        let now = Utc::now();
        let journeys = vec![
            ("Welcome Series", "active", "event_based", "Multi-step email welcome flow for new users", 5),
            ("Cart Abandonment Recovery", "active", "event_based", "Push + email for users who abandon cart", 4),
            ("Loyalty Re-engagement", "active", "segment_entry", "Multi-channel re-engage for dormant loyalty members", 6),
            ("VIP Birthday Reward", "paused", "schedule_based", "Personalized birthday offers for VIP tier", 3),
            ("Post-Purchase Upsell", "draft", "event_based", "Cross-sell journey triggered after purchase", 4),
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
            self.journeys.insert(id, serde_json::json!({
                "id": id,
                "name": name,
                "description": desc,
                "status": status,
                "trigger": { "type": trigger_type, "config": {} },
                "steps": step_list,
                "version": 1,
                "created_at": (now - Duration::days(15)).to_rfc3339(),
                "updated_at": now.to_rfc3339(),
            }));
        }
    }

    fn seed_dco_data(&self) {
        let now = Utc::now();
        let templates = vec![
            ("Holiday Banner DCO", "Dynamic holiday banner with personalized offers", "active"),
            ("Product Recommendation", "AI-selected product images with dynamic pricing", "active"),
            ("Retargeting Creative", "Personalized retargeting ads with last-viewed items", "draft"),
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
            self.dco_templates.insert(id, serde_json::json!({
                "id": id,
                "name": name,
                "description": desc,
                "status": status,
                "components": components,
                "rules": [],
                "created_at": now.to_rfc3339(),
                "updated_at": now.to_rfc3339(),
            }));
        }
    }

    fn seed_cdp_data(&self) {
        use chrono::Duration;
        let now = Utc::now();
        let platforms = vec![
            ("salesforce", "salesforce_data_cloud", "https://api.salesforce.com/cdp/v1", true),
            ("adobe", "adobe_real_time_cdp", "https://platform.adobe.io/data/core", true),
            ("segment", "twilio_segment", "https://api.segment.io/v1", true),
            ("tealium", "tealium", "https://collect.tealiumiq.com/event", false),
            ("hightouch", "hightouch", "https://api.hightouch.com/v1", false),
        ];
        for (name, platform, endpoint, enabled) in &platforms {
            self.cdp_platforms.insert(name.to_string(), serde_json::json!({
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
            }));
        }
        // Seed sync history
        for (i, (name, platform, _, _)) in platforms.iter().enumerate() {
            let sync_id = Uuid::new_v4();
            let status = if i % 3 == 0 { "completed" } else if i % 3 == 1 { "completed" } else { "failed" };
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
            ("Headline A/B Test", "running", "ctr", 0.5, vec![
                ("Control - Standard", true, 0.5, 12500, 375, 3750.0, 0.03, 0.0, 0.0),
                ("Variant A - Urgency", false, 0.5, 12500, 500, 5000.0, 0.04, 0.85, 33.3),
            ]),
            ("Bid Strategy Test", "running", "roi", 0.3, vec![
                ("Control - Even Pacing", true, 0.33, 8000, 240, 4800.0, 0.03, 0.0, 0.0),
                ("Accelerated Pacing", false, 0.33, 8000, 280, 5600.0, 0.035, 0.72, 16.7),
                ("ML-Optimized Pacing", false, 0.34, 8000, 320, 6400.0, 0.04, 0.94, 33.3),
            ]),
            ("DCO vs Static Creative", "completed", "conversions", 1.0, vec![
                ("Control - Static", true, 0.5, 25000, 750, 15000.0, 0.03, 0.0, 0.0),
                ("DCO Dynamic", false, 0.5, 25000, 1000, 20000.0, 0.04, 0.98, 33.3),
            ]),
            ("Channel Priority Test", "draft", "engagement", 0.2, vec![
                ("Control - Email First", true, 0.5, 0, 0, 0.0, 0.0, 0.0, 0.0),
                ("Push First", false, 0.5, 0, 0, 0.0, 0.0, 0.0, 0.0),
            ]),
        ];
        for (name, status, metric, traffic, variants) in experiments {
            let id = Uuid::new_v4();
            let variant_list: Vec<serde_json::Value> = variants.iter().map(|(vname, is_control, weight, samples, convs, rev, cvr, conf, lift)| {
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
            }).collect();
            self.experiments.insert(id, serde_json::json!({
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
            }));
        }
    }
}

impl Default for ManagementStore {
    fn default() -> Self {
        Self::new()
    }
}
