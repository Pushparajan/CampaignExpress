//! DSP router — selects which DSPs to forward bids to and aggregates responses.

use crate::clients::*;
use campaign_core::config::DspIntegrationConfig;
use campaign_core::dsp::*;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Routes bid requests to configured DSP platforms and merges responses.
pub struct DspRouter {
    clients: Vec<Arc<dyn DspClient>>,
    spend_tracker: DashMap<DspPlatform, DspSpendRecord>,
    config: DspIntegrationConfig,
}

impl DspRouter {
    pub fn new(config: &DspIntegrationConfig, dsp_configs: Vec<DspConfig>) -> Self {
        let mut clients: Vec<Arc<dyn DspClient>> = Vec::new();

        for cfg in &dsp_configs {
            if !cfg.enabled {
                continue;
            }
            let client: Arc<dyn DspClient> = match cfg.platform {
                DspPlatform::GoogleDv360 => Arc::new(GoogleDv360Client::new(cfg.clone())),
                DspPlatform::AmazonDsp => Arc::new(AmazonDspClient::new(cfg.clone())),
                DspPlatform::TheTradeDesk => Arc::new(TradeDeskClient::new(cfg.clone())),
                DspPlatform::MetaAds => Arc::new(MetaAdsClient::new(cfg.clone())),
            };
            clients.push(client);
        }

        info!(
            dsp_count = clients.len(),
            "DSP router initialized"
        );

        Self {
            clients,
            spend_tracker: DashMap::new(),
            config: config.clone(),
        }
    }

    /// Route a bid request to all enabled DSPs and collect responses.
    pub fn route_bid(&self, request_id: &str, openrtb_json: &str, impression_ids: &[String]) -> Vec<DspBidResponse> {
        if !self.config.enabled || self.clients.is_empty() {
            return Vec::new();
        }

        let dsp_request = DspBidRequest {
            request_id: request_id.to_string(),
            platform: DspPlatform::GoogleDv360, // overridden per client
            openrtb_payload: openrtb_json.to_string(),
            impression_ids: impression_ids.to_vec(),
            user_id: None,
            timeout_ms: self.config.default_timeout_ms,
            sent_at: Utc::now(),
        };

        let mut responses = Vec::new();

        for client in &self.clients {
            let mut req = dsp_request.clone();
            req.platform = client.platform();

            metrics::counter!("dsp.requests", "platform" => client.platform().seat_id()).increment(1);

            match client.send_bid(&req) {
                Ok(resp) => {
                    metrics::histogram!(
                        "dsp.latency_ms",
                        "platform" => client.platform().seat_id()
                    )
                    .record(resp.latency_ms as f64);

                    if !resp.no_bid {
                        metrics::counter!("dsp.bids", "platform" => client.platform().seat_id())
                            .increment(1);
                    }
                    responses.push(resp);
                }
                Err(e) => {
                    warn!(
                        platform = client.platform().seat_id(),
                        error = %e,
                        "DSP bid failed"
                    );
                    metrics::counter!("dsp.errors", "platform" => client.platform().seat_id())
                        .increment(1);
                }
            }
        }

        responses
    }

    /// Record a win notification for spend tracking.
    pub fn record_win(&self, platform: DspPlatform, win_price: f64) {
        self.spend_tracker
            .entry(platform)
            .and_modify(|record| {
                record.wins += 1;
                record.spend_usd += win_price;
            })
            .or_insert_with(|| DspSpendRecord {
                platform,
                hour: Utc::now(),
                impressions: 0,
                spend_usd: win_price,
                wins: 1,
                avg_win_price: win_price,
            });
    }

    pub fn active_dsp_count(&self) -> usize {
        self.clients.len()
    }
}
