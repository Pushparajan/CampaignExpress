//! DSP (Demand-Side Platform) integration types.
//!
//! Supports programmatic bid routing to major DSPs:
//! - Google Display & Video 360 (DV360)
//! - Amazon DSP
//! - The Trade Desk
//! - Meta Ads (Facebook/Instagram)
//!
//! Each DSP integration translates our internal bid decisions into
//! platform-specific bid request/response formats and handles
//! authentication, rate limiting, and reporting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported DSP platforms.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DspPlatform {
    GoogleDv360,
    AmazonDsp,
    TheTradeDesk,
    MetaAds,
}

impl DspPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            DspPlatform::GoogleDv360 => "Google Display & Video 360",
            DspPlatform::AmazonDsp => "Amazon DSP",
            DspPlatform::TheTradeDesk => "The Trade Desk",
            DspPlatform::MetaAds => "Meta Ads",
        }
    }

    /// OpenRTB seat ID convention per DSP.
    pub fn seat_id(&self) -> &'static str {
        match self {
            DspPlatform::GoogleDv360 => "google-dv360",
            DspPlatform::AmazonDsp => "amazon-dsp",
            DspPlatform::TheTradeDesk => "thetradedesk",
            DspPlatform::MetaAds => "meta-ads",
        }
    }
}

/// Configuration for a single DSP connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspConfig {
    pub platform: DspPlatform,
    pub enabled: bool,
    pub endpoint_url: String,
    /// API key or OAuth token for authentication.
    pub api_key: String,
    /// Optional advertiser/seat ID on the DSP side.
    pub advertiser_id: Option<String>,
    /// Max QPS to this DSP.
    pub rate_limit_qps: u32,
    /// Timeout for DSP API calls in milliseconds.
    pub timeout_ms: u64,
    /// Minimum bid floor override per DSP.
    pub min_bid_floor: Option<f64>,
    /// Budget cap per hour in USD.
    pub hourly_budget_usd: Option<f64>,
}

impl Default for DspConfig {
    fn default() -> Self {
        Self {
            platform: DspPlatform::GoogleDv360,
            enabled: false,
            endpoint_url: String::new(),
            api_key: String::new(),
            advertiser_id: None,
            rate_limit_qps: 1000,
            timeout_ms: 200,
            min_bid_floor: None,
            hourly_budget_usd: None,
        }
    }
}

/// A bid request forwarded to a DSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspBidRequest {
    pub request_id: String,
    pub platform: DspPlatform,
    /// OpenRTB JSON payload (platform-adapted).
    pub openrtb_payload: String,
    pub impression_ids: Vec<String>,
    pub user_id: Option<String>,
    pub timeout_ms: u64,
    pub sent_at: DateTime<Utc>,
}

/// A bid response received from a DSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspBidResponse {
    pub request_id: String,
    pub platform: DspPlatform,
    pub bids: Vec<DspBid>,
    pub no_bid: bool,
    pub latency_ms: u64,
    pub received_at: DateTime<Utc>,
}

/// Individual bid from a DSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspBid {
    pub bid_id: String,
    pub impression_id: String,
    pub price: f64,
    pub creative_id: Option<String>,
    pub creative_url: Option<String>,
    pub adomain: Vec<String>,
    pub deal_id: Option<String>,
}

/// DSP spend tracking for budget management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspSpendRecord {
    pub platform: DspPlatform,
    pub hour: DateTime<Utc>,
    pub impressions: u64,
    pub spend_usd: f64,
    pub wins: u64,
    pub avg_win_price: f64,
}

/// Aggregated DSP performance metrics for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspPerformanceMetrics {
    pub platform: DspPlatform,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: u64,
    pub total_bids: u64,
    pub total_wins: u64,
    pub total_spend_usd: f64,
    pub avg_bid_price: f64,
    pub avg_win_price: f64,
    pub win_rate: f64,
    pub avg_latency_ms: f64,
    pub timeout_rate: f64,
}

/// DSP routing decision — which DSPs to send a bid request to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspRoutingDecision {
    pub request_id: String,
    pub selected_dsps: Vec<DspPlatform>,
    /// Reason for each selection/exclusion.
    pub routing_reasons: Vec<DspRoutingReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspRoutingReason {
    pub platform: DspPlatform,
    pub selected: bool,
    pub reason: String,
}

/// DSP-specific analytics event types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DspEventType {
    BidSent,
    BidReceived,
    BidWon,
    BidLost,
    BidTimeout,
    BidError,
    BudgetExhausted,
    RateLimited,
}
