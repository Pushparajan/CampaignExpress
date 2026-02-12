//! Revenue attribution — tracks conversions and revenue back to campaigns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionModel {
    LastTouch,
    FirstTouch,
    Linear,
    TimeDecay,
    PositionBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_name: String,
    pub revenue: f64,
    pub currency: String,
    pub touchpoints: Vec<Touchpoint>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Touchpoint {
    pub campaign_id: Uuid,
    pub channel: String,
    pub interaction_type: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionResult {
    pub campaign_id: Uuid,
    pub model: AttributionModel,
    pub attributed_conversions: u64,
    pub attributed_revenue: f64,
    pub total_touchpoints: u64,
    pub avg_time_to_conversion_hours: f64,
    pub computed_at: DateTime<Utc>,
}

pub struct RevenueAttributionEngine {
    conversions: dashmap::DashMap<Uuid, Vec<ConversionEvent>>,
    attribution_window_days: u32,
}

impl RevenueAttributionEngine {
    pub fn new(attribution_window_days: u32) -> Self {
        Self {
            conversions: dashmap::DashMap::new(),
            attribution_window_days,
        }
    }

    pub fn record_conversion(&self, event: ConversionEvent) {
        self.conversions
            .entry(event.user_id)
            .or_default()
            .push(event);
    }

    pub fn attribute(&self, campaign_id: &Uuid, model: &AttributionModel) -> AttributionResult {
        let mut total_revenue = 0.0;
        let mut total_conversions = 0u64;
        let mut total_touchpoints = 0u64;
        let cutoff = Utc::now() - chrono::Duration::days(self.attribution_window_days as i64);

        for entry in self.conversions.iter() {
            for conversion in entry.value() {
                if conversion.timestamp < cutoff {
                    continue;
                }
                let relevant: Vec<_> = conversion
                    .touchpoints
                    .iter()
                    .filter(|t| &t.campaign_id == campaign_id)
                    .collect();

                if relevant.is_empty() {
                    continue;
                }

                let credit = match model {
                    AttributionModel::LastTouch => {
                        if conversion.touchpoints.last().map(|t| &t.campaign_id)
                            == Some(campaign_id)
                        {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    AttributionModel::FirstTouch => {
                        if conversion.touchpoints.first().map(|t| &t.campaign_id)
                            == Some(campaign_id)
                        {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    _ => relevant.len() as f64 / conversion.touchpoints.len() as f64,
                };

                total_revenue += conversion.revenue * credit;
                if credit > 0.0 {
                    total_conversions += 1;
                }
                total_touchpoints += relevant.len() as u64;
            }
        }

        AttributionResult {
            campaign_id: *campaign_id,
            model: model.clone(),
            attributed_conversions: total_conversions,
            attributed_revenue: total_revenue,
            total_touchpoints,
            avg_time_to_conversion_hours: 24.0,
            computed_at: Utc::now(),
        }
    }
}
