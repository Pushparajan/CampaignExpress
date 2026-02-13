//! Contextual Bandits — LinUCB for personalized variant selection using user features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeatures {
    pub user_id: Uuid,
    pub features: Vec<f64>,
    pub feature_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualConfig {
    pub campaign_id: Uuid,
    pub enabled: bool,
    pub feature_names: Vec<String>,
    pub alpha: f64,
    pub min_samples_for_personalization: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualDecision {
    pub user_id: Uuid,
    pub variant_id: Uuid,
    pub confidence: f64,
    pub top_features: Vec<(String, f64)>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance: f64,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInsight {
    pub segment_description: String,
    pub best_variant_id: Uuid,
    pub conversion_rate: f64,
    pub sample_size: u64,
    pub confidence: f64,
}

pub struct ContextualBanditEngine {
    configs: dashmap::DashMap<Uuid, ContextualConfig>,
    weights: dashmap::DashMap<(Uuid, Uuid), Vec<f64>>,
    sample_count: dashmap::DashMap<Uuid, u64>,
}

impl ContextualBanditEngine {
    pub fn new() -> Self {
        Self {
            configs: dashmap::DashMap::new(),
            weights: dashmap::DashMap::new(),
            sample_count: dashmap::DashMap::new(),
        }
    }

    pub fn configure(&self, config: ContextualConfig) {
        self.configs.insert(config.campaign_id, config);
    }

    pub fn select_variant(
        &self,
        campaign_id: &Uuid,
        user: &UserFeatures,
        variant_ids: &[Uuid],
    ) -> ContextualDecision {
        let config = self.configs.get(campaign_id);
        let samples = self.sample_count.get(campaign_id).map(|s| *s).unwrap_or(0);

        let min_samples = config
            .as_ref()
            .map(|c| c.min_samples_for_personalization)
            .unwrap_or(1000);

        if samples < min_samples || variant_ids.is_empty() {
            let variant_id = variant_ids.first().copied().unwrap_or_else(Uuid::new_v4);
            return ContextualDecision {
                user_id: user.user_id,
                variant_id,
                confidence: 0.3,
                top_features: Vec::new(),
                method: "cold_start_fallback".to_string(),
            };
        }

        let alpha = config.as_ref().map(|c| c.alpha).unwrap_or(1.0);
        let mut best_score = f64::NEG_INFINITY;
        let mut best_variant = variant_ids[0];
        let mut best_features = Vec::new();

        for &variant_id in variant_ids {
            let w = self
                .weights
                .get(&(*campaign_id, variant_id))
                .map(|w| w.clone())
                .unwrap_or_else(|| vec![0.0; user.features.len()]);

            let score: f64 = w
                .iter()
                .zip(user.features.iter())
                .map(|(wi, xi)| wi * xi)
                .sum();

            let exploration = alpha * (1.0 / (samples as f64 + 1.0)).sqrt();
            let ucb_score = score + exploration;

            if ucb_score > best_score {
                best_score = ucb_score;
                best_variant = variant_id;
                best_features = w
                    .iter()
                    .zip(user.feature_names.iter())
                    .map(|(wi, name)| (name.clone(), wi.abs()))
                    .collect();
                best_features
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                best_features.truncate(3);
            }
        }

        ContextualDecision {
            user_id: user.user_id,
            variant_id: best_variant,
            confidence: (0.5 + samples as f64 / (samples as f64 + 1000.0) * 0.5).min(0.95),
            top_features: best_features,
            method: "linucb".to_string(),
        }
    }

    pub fn record_outcome(
        &self,
        campaign_id: &Uuid,
        variant_id: &Uuid,
        features: &[f64],
        reward: f64,
    ) {
        self.sample_count
            .entry(*campaign_id)
            .and_modify(|c| *c += 1)
            .or_insert(1);

        let learning_rate = 0.01;
        let mut weights = self
            .weights
            .entry((*campaign_id, *variant_id))
            .or_insert_with(|| vec![0.0; features.len()])
            .clone();

        let prediction: f64 = weights
            .iter()
            .zip(features.iter())
            .map(|(w, x)| w * x)
            .sum();
        let error = reward - prediction;

        for (w, x) in weights.iter_mut().zip(features.iter()) {
            *w += learning_rate * error * x;
        }

        self.weights.insert((*campaign_id, *variant_id), weights);
    }

    pub fn get_feature_importance(&self, campaign_id: &Uuid) -> Vec<FeatureImportance> {
        let config = match self.configs.get(campaign_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut importance: Vec<f64> = vec![0.0; config.feature_names.len()];
        let mut count = 0;

        for entry in self.weights.iter() {
            let (cid, _) = entry.key();
            if cid == campaign_id {
                for (i, w) in entry.value().iter().enumerate() {
                    if i < importance.len() {
                        importance[i] += w.abs();
                    }
                }
                count += 1;
            }
        }

        if count > 0 {
            for v in &mut importance {
                *v /= count as f64;
            }
        }

        let total: f64 = importance.iter().sum();
        config
            .feature_names
            .iter()
            .enumerate()
            .map(|(i, name)| FeatureImportance {
                feature_name: name.clone(),
                importance: if total > 0.0 {
                    importance[i] / total
                } else {
                    0.0
                },
                direction: if importance.get(i).copied().unwrap_or(0.0) >= 0.0 {
                    "positive".to_string()
                } else {
                    "negative".to_string()
                },
            })
            .collect()
    }
}

impl Default for ContextualBanditEngine {
    fn default() -> Self {
        Self::new()
    }
}
