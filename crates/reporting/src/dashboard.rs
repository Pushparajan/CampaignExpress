//! Campaign performance dashboard — real-time metrics aggregation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMetrics {
    pub campaign_id: Uuid,
    pub name: String,
    pub sends: u64,
    pub deliveries: u64,
    pub opens: u64,
    pub unique_opens: u64,
    pub clicks: u64,
    pub unique_clicks: u64,
    pub bounces: u64,
    pub unsubscribes: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub delivery_rate: f64,
    pub open_rate: f64,
    pub click_rate: f64,
    pub click_to_open_rate: f64,
    pub conversion_rate: f64,
    pub bounce_rate: f64,
    pub unsubscribe_rate: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBreakdown {
    pub channel: String,
    pub sends: u64,
    pub deliveries: u64,
    pub engagements: u64,
    pub conversions: u64,
    pub revenue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub total_active_campaigns: u64,
    pub total_sends_today: u64,
    pub total_sends_week: u64,
    pub overall_delivery_rate: f64,
    pub overall_open_rate: f64,
    pub overall_click_rate: f64,
    pub overall_conversion_rate: f64,
    pub total_revenue_30d: f64,
    pub channel_breakdown: Vec<ChannelBreakdown>,
    pub sends_over_time: Vec<TimeSeriesPoint>,
    pub generated_at: DateTime<Utc>,
}

pub struct CampaignDashboard {
    metrics: dashmap::DashMap<Uuid, CampaignMetrics>,
}

impl CampaignDashboard {
    pub fn new() -> Self {
        Self {
            metrics: dashmap::DashMap::new(),
        }
    }

    pub fn update_metrics(&self, metrics: CampaignMetrics) {
        self.metrics.insert(metrics.campaign_id, metrics);
    }

    pub fn get_campaign_metrics(&self, campaign_id: &Uuid) -> Option<CampaignMetrics> {
        self.metrics.get(campaign_id).map(|m| m.clone())
    }

    pub fn get_overview(&self) -> DashboardOverview {
        let all: Vec<_> = self.metrics.iter().map(|m| m.value().clone()).collect();
        let total_sends: u64 = all.iter().map(|m| m.sends).sum();
        let total_opens: u64 = all.iter().map(|m| m.opens).sum();
        let total_clicks: u64 = all.iter().map(|m| m.clicks).sum();
        let total_conversions: u64 = all.iter().map(|m| m.conversions).sum();
        let total_deliveries: u64 = all.iter().map(|m| m.deliveries).sum();
        let total_revenue: f64 = all.iter().map(|m| m.revenue).sum();

        DashboardOverview {
            total_active_campaigns: all.len() as u64,
            total_sends_today: total_sends / 30,
            total_sends_week: total_sends / 4,
            overall_delivery_rate: if total_sends > 0 {
                total_deliveries as f64 / total_sends as f64
            } else {
                0.0
            },
            overall_open_rate: if total_deliveries > 0 {
                total_opens as f64 / total_deliveries as f64
            } else {
                0.0
            },
            overall_click_rate: if total_deliveries > 0 {
                total_clicks as f64 / total_deliveries as f64
            } else {
                0.0
            },
            overall_conversion_rate: if total_clicks > 0 {
                total_conversions as f64 / total_clicks as f64
            } else {
                0.0
            },
            total_revenue_30d: total_revenue,
            channel_breakdown: Vec::new(),
            sends_over_time: Vec::new(),
            generated_at: Utc::now(),
        }
    }

    pub fn list_campaign_metrics(&self) -> Vec<CampaignMetrics> {
        self.metrics.iter().map(|m| m.value().clone()).collect()
    }
}

impl Default for CampaignDashboard {
    fn default() -> Self {
        Self::new()
    }
}
