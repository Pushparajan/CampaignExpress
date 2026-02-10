//! Bid processing pipeline: receives OpenRTB requests, fetches user profiles,
//! runs NPU inference, selects the winning offer, and returns a bid response.

use campaign_analytics::AnalyticsLogger;
use campaign_cache::RedisCache;
use campaign_core::loyalty::LoyaltyTier;
use campaign_core::openrtb::{Bid, BidRequest, BidResponse, SeatBid};
use campaign_core::types::{BidDecision, EventType};
use campaign_npu::NpuEngine;
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// Processes a single bid request through the full pipeline.
pub struct BidProcessor {
    npu: Arc<NpuEngine>,
    cache: Arc<RedisCache>,
    analytics: Arc<AnalyticsLogger>,
    node_id: String,
}

impl BidProcessor {
    pub fn new(
        npu: Arc<NpuEngine>,
        cache: Arc<RedisCache>,
        analytics: Arc<AnalyticsLogger>,
        node_id: String,
    ) -> Self {
        Self {
            npu,
            cache,
            analytics,
            node_id,
        }
    }

    /// Process a bid request and return a bid response.
    pub async fn process(
        &self,
        request: &BidRequest,
        agent_id: &str,
    ) -> anyhow::Result<BidResponse> {
        let start = std::time::Instant::now();
        let request_id = &request.id;

        metrics::counter!("bids.requests").increment(1);

        // Extract user ID from request
        let user_id = request
            .user
            .as_ref()
            .and_then(|u| u.id.clone())
            .or_else(|| {
                request
                    .user
                    .as_ref()
                    .and_then(|u| u.buyeruid.clone())
            })
            .unwrap_or_else(|| "anonymous".to_string());

        // Fetch user profile from cache
        let profile = match self.cache.get_profile(&user_id).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                debug!(user_id = %user_id, "Cache miss, using default profile");
                RedisCache::default_profile(&user_id)
            }
            Err(e) => {
                warn!(error = %e, "Cache error, using default profile");
                RedisCache::default_profile(&user_id)
            }
        };

        // Check frequency cap
        if profile.frequency_cap.impressions_1h >= profile.frequency_cap.max_per_hour {
            metrics::counter!("bids.frequency_capped").increment(1);
            self.analytics
                .log_event(
                    EventType::NoBid,
                    request_id.clone(),
                    agent_id.to_string(),
                    None,
                    Some(user_id),
                    None,
                    None,
                    None,
                    None,
                    Some(start.elapsed().as_micros() as u64),
                )
                .await;
            return Ok(BidResponse::no_bid(request_id.clone()));
        }

        // Generate candidate offer IDs (in production, these come from campaign targeting)
        let offer_ids: Vec<String> = (0..self.npu.config().batch_size.min(request.imp.len().max(4)))
            .map(|i| format!("offer-{:04}", i))
            .collect();

        // Run NPU inference to score offers (loyalty features are baked into the feature vector)
        let inference_start = std::time::Instant::now();
        let mut results = self.npu.score_offers(&profile, &offer_ids)?;
        let inference_latency_us = inference_start.elapsed().as_micros() as u64;

        metrics::histogram!("inference.latency_us").record(inference_latency_us as f64);

        // Apply loyalty tier bid boost: higher tiers get higher bid willingness
        let tier_boost = match profile.loyalty.as_ref().map(|l| l.tier) {
            Some(LoyaltyTier::Reserve) => 1.3,
            Some(LoyaltyTier::Gold) => 1.15,
            _ => 1.0,
        };
        if tier_boost > 1.0 {
            for r in &mut results {
                r.recommended_bid *= tier_boost;
            }
            metrics::counter!("bids.loyalty_boosted").increment(1);
        }

        // Select the winning offer (highest score above bid floor)
        let mut seat_bids = Vec::new();

        for imp in &request.imp {
            let best = results
                .iter()
                .filter(|r| r.recommended_bid >= imp.bidfloor)
                .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

            if let Some(winner) = best {
                let bid_id = Uuid::new_v4().to_string();
                let decision = BidDecision {
                    request_id: request_id.clone(),
                    impression_id: imp.id.clone(),
                    offer_id: winner.offer_id.clone(),
                    bid_price: winner.recommended_bid,
                    creative_url: format!("https://cdn.campaignexpress.io/creative/{}", winner.offer_id),
                    landing_url: format!("https://campaignexpress.io/click/{}", winner.offer_id),
                    agent_id: agent_id.to_string(),
                    node_id: self.node_id.clone(),
                    inference_latency_us,
                    total_latency_us: start.elapsed().as_micros() as u64,
                    timestamp: Utc::now(),
                };

                let bid = Bid {
                    id: bid_id,
                    impid: imp.id.clone(),
                    price: decision.bid_price,
                    adid: Some(decision.offer_id.clone()),
                    nurl: Some(format!(
                        "https://campaignexpress.io/win/{}/{}",
                        request_id, imp.id
                    )),
                    adm: Some(format!(
                        "<img src=\"{}\" />",
                        decision.creative_url
                    )),
                    crid: Some(decision.offer_id.clone()),
                    w: imp.banner.as_ref().and_then(|b| b.w).unwrap_or(300),
                    h: imp.banner.as_ref().and_then(|b| b.h).unwrap_or(250),
                    ext: None,
                };

                seat_bids.push(SeatBid {
                    bid: vec![bid],
                    seat: Some("campaign-express".to_string()),
                    group: 0,
                });

                // Log bid response event
                self.analytics
                    .log_event(
                        EventType::BidResponse,
                        request_id.clone(),
                        agent_id.to_string(),
                        Some(imp.id.clone()),
                        Some(user_id.clone()),
                        Some(decision.offer_id),
                        Some(decision.bid_price),
                        None,
                        Some(inference_latency_us),
                        Some(decision.total_latency_us),
                    )
                    .await;
            }
        }

        let total_latency_us = start.elapsed().as_micros() as u64;
        metrics::histogram!("bids.total_latency_us").record(total_latency_us as f64);

        if seat_bids.is_empty() {
            metrics::counter!("bids.no_bid").increment(1);
        } else {
            metrics::counter!("bids.responded").increment(1);
        }

        Ok(BidResponse {
            id: request_id.clone(),
            seatbid: seat_bids,
            bidid: Some(Uuid::new_v4().to_string()),
            cur: "USD".to_string(),
            ext: None,
        })
    }
}
