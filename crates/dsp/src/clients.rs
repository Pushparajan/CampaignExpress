//! DSP client implementations for each platform.
//! Each client translates our internal bid format to the platform-specific API.

use campaign_core::dsp::*;
use chrono::Utc;
use tracing::debug;

/// Trait for DSP platform clients.
pub trait DspClient: Send + Sync {
    fn platform(&self) -> DspPlatform;
    fn send_bid(&self, request: &DspBidRequest) -> Result<DspBidResponse, anyhow::Error>;
}

// ─── Google DV360 ───────────────────────────────────────────────────────────

pub struct GoogleDv360Client {
    _config: DspConfig,
}

impl GoogleDv360Client {
    pub fn new(config: DspConfig) -> Self {
        Self { _config: config }
    }
}

impl DspClient for GoogleDv360Client {
    fn platform(&self) -> DspPlatform {
        DspPlatform::GoogleDv360
    }

    fn send_bid(&self, request: &DspBidRequest) -> Result<DspBidResponse, anyhow::Error> {
        let start = std::time::Instant::now();
        debug!(
            platform = "google_dv360",
            request_id = %request.request_id,
            "Sending bid to Google DV360"
        );

        // In production: HTTP POST to DV360 Authorized Buyers RTB endpoint
        // For now: simulate a response
        Ok(DspBidResponse {
            request_id: request.request_id.clone(),
            platform: DspPlatform::GoogleDv360,
            bids: Vec::new(),
            no_bid: true,
            latency_ms: start.elapsed().as_millis() as u64,
            received_at: Utc::now(),
        })
    }
}

// ─── Amazon DSP ─────────────────────────────────────────────────────────────

pub struct AmazonDspClient {
    _config: DspConfig,
}

impl AmazonDspClient {
    pub fn new(config: DspConfig) -> Self {
        Self { _config: config }
    }
}

impl DspClient for AmazonDspClient {
    fn platform(&self) -> DspPlatform {
        DspPlatform::AmazonDsp
    }

    fn send_bid(&self, request: &DspBidRequest) -> Result<DspBidResponse, anyhow::Error> {
        let start = std::time::Instant::now();
        debug!(
            platform = "amazon_dsp",
            request_id = %request.request_id,
            "Sending bid to Amazon DSP"
        );

        // In production: HTTP POST to Amazon DSP API
        Ok(DspBidResponse {
            request_id: request.request_id.clone(),
            platform: DspPlatform::AmazonDsp,
            bids: Vec::new(),
            no_bid: true,
            latency_ms: start.elapsed().as_millis() as u64,
            received_at: Utc::now(),
        })
    }
}

// ─── The Trade Desk ─────────────────────────────────────────────────────────

pub struct TradeDeskClient {
    _config: DspConfig,
}

impl TradeDeskClient {
    pub fn new(config: DspConfig) -> Self {
        Self { _config: config }
    }
}

impl DspClient for TradeDeskClient {
    fn platform(&self) -> DspPlatform {
        DspPlatform::TheTradeDesk
    }

    fn send_bid(&self, request: &DspBidRequest) -> Result<DspBidResponse, anyhow::Error> {
        let start = std::time::Instant::now();
        debug!(
            platform = "the_trade_desk",
            request_id = %request.request_id,
            "Sending bid to The Trade Desk"
        );

        // In production: HTTP POST to TTD OpenPath API
        Ok(DspBidResponse {
            request_id: request.request_id.clone(),
            platform: DspPlatform::TheTradeDesk,
            bids: Vec::new(),
            no_bid: true,
            latency_ms: start.elapsed().as_millis() as u64,
            received_at: Utc::now(),
        })
    }
}

// ─── Meta Ads ───────────────────────────────────────────────────────────────

pub struct MetaAdsClient {
    _config: DspConfig,
}

impl MetaAdsClient {
    pub fn new(config: DspConfig) -> Self {
        Self { _config: config }
    }
}

impl DspClient for MetaAdsClient {
    fn platform(&self) -> DspPlatform {
        DspPlatform::MetaAds
    }

    fn send_bid(&self, request: &DspBidRequest) -> Result<DspBidResponse, anyhow::Error> {
        let start = std::time::Instant::now();
        debug!(
            platform = "meta_ads",
            request_id = %request.request_id,
            "Sending bid to Meta Ads"
        );

        // In production: HTTP POST to Meta Marketing API
        Ok(DspBidResponse {
            request_id: request.request_id.clone(),
            platform: DspPlatform::MetaAds,
            bids: Vec::new(),
            no_bid: true,
            latency_ms: start.elapsed().as_millis() as u64,
            received_at: Utc::now(),
        })
    }
}
