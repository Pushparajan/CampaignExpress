//! Management domain types — campaigns, creatives, targeting, audit log.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Campaign ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: Uuid,
    pub name: String,
    pub status: CampaignStatus,
    pub budget: f64,
    pub daily_budget: f64,
    pub pacing: PacingStrategy,
    pub targeting: TargetingConfig,
    pub schedule_start: Option<DateTime<Utc>>,
    pub schedule_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub stats: CampaignStats,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PacingStrategy {
    Even,
    Accelerated,
    Manual,
}

impl Default for PacingStrategy {
    fn default() -> Self {
        PacingStrategy::Even
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingConfig {
    #[serde(default)]
    pub geo_regions: Vec<String>,
    #[serde(default)]
    pub segments: Vec<u32>,
    #[serde(default)]
    pub devices: Vec<String>,
    #[serde(default = "default_floor_price")]
    pub floor_price: f64,
    #[serde(default)]
    pub max_bid: Option<f64>,
    #[serde(default)]
    pub frequency_cap_hourly: Option<u32>,
    #[serde(default)]
    pub frequency_cap_daily: Option<u32>,
    #[serde(default)]
    pub loyalty_tiers: Vec<String>,
    #[serde(default)]
    pub dsp_platforms: Vec<String>,
}

fn default_floor_price() -> f64 {
    0.50
}

impl Default for TargetingConfig {
    fn default() -> Self {
        Self {
            geo_regions: Vec::new(),
            segments: Vec::new(),
            devices: Vec::new(),
            floor_price: default_floor_price(),
            max_bid: None,
            frequency_cap_hourly: Some(10),
            frequency_cap_daily: Some(50),
            loyalty_tiers: Vec::new(),
            dsp_platforms: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CampaignStats {
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub spend: f64,
    pub ctr: f64,
    pub win_rate: f64,
    pub avg_bid: f64,
    pub avg_win_price: f64,
    #[serde(default)]
    pub hourly_data: Vec<HourlyDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyDataPoint {
    pub hour: DateTime<Utc>,
    pub impressions: u64,
    pub clicks: u64,
    pub spend: f64,
}

// ─── Creative ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creative {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub name: String,
    pub format: CreativeFormat,
    pub asset_url: String,
    pub width: u32,
    pub height: u32,
    pub status: CreativeStatus,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreativeFormat {
    Banner,
    Native,
    Video,
    Html5,
    Rich,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreativeStatus {
    Draft,
    Active,
    Paused,
    Rejected,
    Archived,
}

// ─── Monitoring ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringOverview {
    pub total_campaigns: u64,
    pub active_campaigns: u64,
    pub total_impressions: u64,
    pub total_clicks: u64,
    pub total_spend: f64,
    pub avg_ctr: f64,
    pub avg_latency_us: f64,
    pub active_pods: u32,
    pub offers_per_hour: u64,
    pub cache_hit_rate: f64,
    pub no_bid_rate: f64,
    pub error_rate: f64,
}

// ─── Audit Log ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user: String,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Pause,
    Resume,
    ModelReload,
    Login,
}

// ─── API Request/Response types ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    #[serde(default)]
    pub budget: f64,
    #[serde(default)]
    pub daily_budget: f64,
    #[serde(default)]
    pub pacing: PacingStrategy,
    #[serde(default)]
    pub targeting: TargetingConfig,
    pub schedule_start: Option<DateTime<Utc>>,
    pub schedule_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub budget: Option<f64>,
    pub daily_budget: Option<f64>,
    pub pacing: Option<PacingStrategy>,
    pub targeting: Option<TargetingConfig>,
    pub schedule_start: Option<DateTime<Utc>>,
    pub schedule_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCreativeRequest {
    pub campaign_id: Uuid,
    pub name: String,
    pub format: CreativeFormat,
    pub asset_url: String,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_width() -> u32 {
    300
}
fn default_height() -> u32 {
    250
}

#[derive(Debug, Deserialize)]
pub struct UpdateCreativeRequest {
    pub name: Option<String>,
    pub format: Option<CreativeFormat>,
    pub asset_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub status: Option<CreativeStatus>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
