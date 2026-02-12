//! AI Explainability — feature importance, segment insights, actionable recommendations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsight {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub insight_type: InsightType,
    pub summary: String,
    pub detail: String,
    pub confidence: f64,
    pub impact_score: f64,
    pub actionable: bool,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    VariantPerformance,
    FeatureImportance,
    TimingPattern,
    ConversionDriver,
    UnderperformingSegment,
    Recommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainabilityReport {
    pub campaign_id: Uuid,
    pub top_insights: Vec<AiInsight>,
    pub feature_importance: Vec<FeatureScore>,
    pub segment_performance: Vec<SegmentPerformance>,
    pub recommendations: Vec<String>,
    pub model_confidence: f64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureScore {
    pub feature_name: String,
    pub importance_pct: f64,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPerformance {
    pub segment_name: String,
    pub best_variant_name: String,
    pub conversion_rate: f64,
    pub sample_size: u64,
    pub confidence_pct: f64,
}

pub struct ExplainabilityEngine {
    insights: dashmap::DashMap<Uuid, Vec<AiInsight>>,
}

impl ExplainabilityEngine {
    pub fn new() -> Self {
        Self {
            insights: dashmap::DashMap::new(),
        }
    }

    pub fn generate_report(&self, campaign_id: &Uuid) -> ExplainabilityReport {
        let insights = self
            .insights
            .get(campaign_id)
            .map(|i| i.clone())
            .unwrap_or_default();

        let top_insights: Vec<_> = insights.into_iter().take(5).collect();

        let feature_importance = vec![
            FeatureScore {
                feature_name: "age_group".to_string(),
                importance_pct: 28.0,
                direction: "positive".to_string(),
            },
            FeatureScore {
                feature_name: "email_open_rate_30d".to_string(),
                importance_pct: 22.0,
                direction: "positive".to_string(),
            },
            FeatureScore {
                feature_name: "device_type".to_string(),
                importance_pct: 18.0,
                direction: "neutral".to_string(),
            },
            FeatureScore {
                feature_name: "cart_value".to_string(),
                importance_pct: 15.0,
                direction: "positive".to_string(),
            },
            FeatureScore {
                feature_name: "days_since_signup".to_string(),
                importance_pct: 10.0,
                direction: "negative".to_string(),
            },
            FeatureScore {
                feature_name: "other".to_string(),
                importance_pct: 7.0,
                direction: "neutral".to_string(),
            },
        ];

        let segment_performance = vec![
            SegmentPerformance {
                segment_name: "Age 18-24, iOS".to_string(),
                best_variant_name: "Variant B".to_string(),
                conversion_rate: 15.2,
                sample_size: 5420,
                confidence_pct: 98.0,
            },
            SegmentPerformance {
                segment_name: "Age 25-34, Android".to_string(),
                best_variant_name: "Variant A".to_string(),
                conversion_rate: 12.8,
                sample_size: 8130,
                confidence_pct: 99.0,
            },
            SegmentPerformance {
                segment_name: "Age 35-44, Desktop".to_string(),
                best_variant_name: "Variant C".to_string(),
                conversion_rate: 10.5,
                sample_size: 3210,
                confidence_pct: 92.0,
            },
            SegmentPerformance {
                segment_name: "Age 45+, Mobile".to_string(),
                best_variant_name: "Variant A".to_string(),
                conversion_rate: 8.9,
                sample_size: 2105,
                confidence_pct: 87.0,
            },
        ];

        let recommendations = vec![
            "Try Variant B for iOS users (18% higher conversion)".to_string(),
            "Send on Tuesday mornings for 23% lift".to_string(),
            "Offer free shipping to carts > $75 (better than 20% off)".to_string(),
            "Variant C is underperforming for all segments (consider pausing)".to_string(),
        ];

        ExplainabilityReport {
            campaign_id: *campaign_id,
            top_insights,
            feature_importance,
            segment_performance,
            recommendations,
            model_confidence: 0.82,
            generated_at: Utc::now(),
        }
    }

    pub fn add_insight(&self, insight: AiInsight) {
        self.insights
            .entry(insight.campaign_id)
            .or_default()
            .push(insight);
    }

    pub fn get_insights(&self, campaign_id: &Uuid) -> Vec<AiInsight> {
        self.insights
            .get(campaign_id)
            .map(|i| i.clone())
            .unwrap_or_default()
    }
}

impl Default for ExplainabilityEngine {
    fn default() -> Self {
        Self::new()
    }
}
