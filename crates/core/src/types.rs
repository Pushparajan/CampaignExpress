use crate::loyalty::LoyaltyProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User profile for personalization, stored in Redis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub segments: Vec<u32>,
    pub interests: Vec<f32>,
    pub geo_region: Option<String>,
    pub device_type: Option<DeviceType>,
    pub recency_score: f32,
    pub frequency_cap: FrequencyCap,
    pub last_seen: DateTime<Utc>,
    /// Loyalty program state (tier, stars, rewards).
    #[serde(default)]
    pub loyalty: Option<LoyaltyProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyCap {
    pub impressions_24h: u32,
    pub impressions_1h: u32,
    pub max_per_hour: u32,
    pub max_per_day: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Ctv,
}

/// An ad campaign/offer available for bidding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdOffer {
    pub offer_id: String,
    pub campaign_id: String,
    pub advertiser_id: String,
    pub creative_url: String,
    pub landing_url: String,
    pub bid_floor: f64,
    pub max_bid: f64,
    pub target_segments: Vec<u32>,
    pub priority: f32,
}

/// Result of SNN inference for a single user-offer pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub offer_id: String,
    pub score: f32,
    pub predicted_ctr: f32,
    pub recommended_bid: f64,
    pub latency_us: u64,
}

/// Final bid decision after agent processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidDecision {
    pub request_id: String,
    pub impression_id: String,
    pub offer_id: String,
    pub bid_price: f64,
    pub creative_url: String,
    pub landing_url: String,
    pub agent_id: String,
    pub node_id: String,
    pub inference_latency_us: u64,
    pub total_latency_us: u64,
    pub timestamp: DateTime<Utc>,
}

/// Analytics event logged to ClickHouse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub request_id: String,
    pub impression_id: Option<String>,
    pub user_id: Option<String>,
    pub offer_id: Option<String>,
    pub bid_price: Option<f64>,
    pub win_price: Option<f64>,
    pub agent_id: String,
    pub node_id: String,
    pub inference_latency_us: Option<u64>,
    pub total_latency_us: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    BidRequest,
    BidResponse,
    Impression,
    Click,
    Conversion,
    NoBid,
    Timeout,
    Error,
    // Loyalty events
    LoyaltyEarn,
    LoyaltyRedeem,
    LoyaltyTierUp,
    LoyaltyTierDown,
    LoyaltyOfferServed,
    LoyaltyOfferClicked,
    LoyaltyOfferRedeemed,
    // DSP events
    DspBidSent,
    DspBidWon,
    DspBidLost,
    DspBidTimeout,
    // Omnichannel ingest events
    ChannelIngest,
    // Activation events
    ActivationSent,
    ActivationDelivered,
    ActivationFailed,
    // Journey events
    JourneyEntered,
    JourneyStepCompleted,
    JourneyCompleted,
    JourneyExited,
    JourneySuppressedBid,
    // DCO events
    DcoAssembly,
    DcoImpression,
    DcoClick,
    DcoConversion,
    // CDP events
    CdpSyncInbound,
    CdpSyncOutbound,
    CdpWebhook,
    // Experimentation events
    ExperimentAssignment,
    ExperimentConversion,
    // Template events
    TemplateRendered,
    TemplateDelivered,
    // Web SDK events
    WebPageView,
    WebClick,
    WebFormSubmit,
    WebScroll,
    WebCustomEvent,
    WebSessionStart,
    WebSessionEnd,
}

/// Internal message envelope for NATS inter-agent communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub msg_type: AgentMessageType,
    pub payload: serde_json::Value,
    pub source_agent: String,
    pub source_node: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentMessageType {
    BidRequest,
    BidResponse,
    ModelUpdate,
    HealthCheck,
    Metrics,
}

impl Default for FrequencyCap {
    fn default() -> Self {
        Self {
            impressions_24h: 0,
            impressions_1h: 0,
            max_per_hour: 10,
            max_per_day: 50,
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            segments: Vec::new(),
            interests: Vec::new(),
            geo_region: None,
            device_type: None,
            recency_score: 0.0,
            frequency_cap: FrequencyCap::default(),
            last_seen: Utc::now(),
            loyalty: None,
        }
    }
}

// ─── Journey Events ─────────────────────────────────────────────────────
/// Extended event types for journey orchestration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JourneyEventType {
    JourneyEntered,
    StepCompleted,
    JourneyCompleted,
    JourneyExited,
    JourneyError,
    SuppressedBid,
}

// ─── Unified Profile Extensions ────────────────────────────────────────
/// Consent flags for GDPR/CCPA compliance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsentFlags {
    pub gdpr_consent: bool,
    pub ccpa_opt_out: bool,
    pub email_opt_in: bool,
    pub push_opt_in: bool,
    pub sms_opt_in: bool,
    pub personalization_consent: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Dynamic user segment with RFM-based criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicSegment {
    pub id: u32,
    pub name: String,
    pub criteria: SegmentCriteria,
    pub user_count: u64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SegmentCriteria {
    pub min_recency_days: Option<u32>,
    pub max_recency_days: Option<u32>,
    pub min_frequency: Option<u32>,
    pub max_frequency: Option<u32>,
    pub min_monetary: Option<f64>,
    pub max_monetary: Option<f64>,
    pub required_events: Vec<String>,
    pub excluded_events: Vec<String>,
    pub loyalty_tiers: Vec<String>,
}

// ─── Experimentation ────────────────────────────────────────────────────
/// A/B/n experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: ExperimentStatus,
    pub variants: Vec<ExperimentVariant>,
    pub traffic_allocation: f64,
    pub metric: String,
    pub min_sample_size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentVariant {
    pub id: Uuid,
    pub name: String,
    pub weight: f64,
    pub is_control: bool,
    pub config: serde_json::Value,
    pub results: VariantResults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VariantResults {
    pub sample_size: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub conversion_rate: f64,
    pub confidence: f64,
    pub lift: f64,
}

// ─── Attribution ────────────────────────────────────────────────────────
/// Cross-channel attribution touchpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionTouchpoint {
    pub id: Uuid,
    pub user_id: String,
    pub channel: String,
    pub campaign_id: Option<String>,
    pub journey_id: Option<Uuid>,
    pub creative_id: Option<Uuid>,
    pub event_type: String,
    pub revenue: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttributionModel {
    LastTouch,
    FirstTouch,
    Linear,
    TimeDecay,
    PositionBased,
    DataDriven,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionResult {
    pub conversion_id: Uuid,
    pub user_id: String,
    pub model: AttributionModel,
    pub touchpoints: Vec<AttributionCredit>,
    pub total_revenue: f64,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionCredit {
    pub touchpoint_id: Uuid,
    pub channel: String,
    pub credit: f64,
    pub revenue_attributed: f64,
}

// ─── Template Management ────────────────────────────────────────────────
/// Message template for owned channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub id: Uuid,
    pub name: String,
    pub channel: String,
    pub subject: Option<String>,
    pub body_template: String,
    pub variables: Vec<TemplateVariable>,
    pub status: TemplateStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: String,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateStatus {
    Draft,
    Active,
    Archived,
}
