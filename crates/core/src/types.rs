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
