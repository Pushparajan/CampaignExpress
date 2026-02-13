//! Multi-Armed Bandit engine — Thompson Sampling, UCB1, Epsilon-Greedy
//! for dynamic creative optimization.

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum BanditAlgorithm {
    #[default]
    ThompsonSampling,
    Ucb1,
    EpsilonGreedy {
        epsilon: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditConfig {
    pub campaign_id: Uuid,
    pub algorithm: BanditAlgorithm,
    pub min_exploration_rate: f64,
    pub variants: Vec<VariantConfig>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantConfig {
    pub id: Uuid,
    pub name: String,
    pub creative_url: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStats {
    pub variant_id: Uuid,
    pub impressions: u64,
    pub conversions: u64,
    pub conversion_rate: f64,
    pub confidence_interval_lower: f64,
    pub confidence_interval_upper: f64,
    pub traffic_allocation: f64,
    pub is_winner: bool,
    pub estimated_value: f64,
}

#[derive(Debug, Clone)]
struct VariantState {
    alpha: f64,
    beta: f64,
    impressions: u64,
    conversions: u64,
}

pub struct BanditEngine {
    configs: dashmap::DashMap<Uuid, BanditConfig>,
    states: dashmap::DashMap<(Uuid, Uuid), VariantState>,
}

impl BanditEngine {
    pub fn new() -> Self {
        Self {
            configs: dashmap::DashMap::new(),
            states: dashmap::DashMap::new(),
        }
    }

    pub fn register_campaign(&self, config: BanditConfig) {
        for variant in &config.variants {
            self.states.insert(
                (config.campaign_id, variant.id),
                VariantState {
                    alpha: 1.0,
                    beta: 1.0,
                    impressions: 0,
                    conversions: 0,
                },
            );
        }
        self.configs.insert(config.campaign_id, config);
    }

    pub fn select_variant(&self, campaign_id: &Uuid) -> Option<Uuid> {
        let config = self.configs.get(campaign_id)?;
        let active_variants: Vec<_> = config.variants.iter().filter(|v| v.active).collect();

        if active_variants.is_empty() {
            return None;
        }

        match &config.algorithm {
            BanditAlgorithm::ThompsonSampling => {
                self.thompson_sampling(campaign_id, &active_variants)
            }
            BanditAlgorithm::Ucb1 => self.ucb1(campaign_id, &active_variants),
            BanditAlgorithm::EpsilonGreedy { epsilon } => {
                self.epsilon_greedy(campaign_id, &active_variants, *epsilon)
            }
        }
    }

    fn thompson_sampling(&self, campaign_id: &Uuid, variants: &[&VariantConfig]) -> Option<Uuid> {
        let mut rng = rand::thread_rng();
        let mut best_sample = f64::NEG_INFINITY;
        let mut best_variant = None;

        for variant in variants {
            let state = self
                .states
                .get(&(*campaign_id, variant.id))
                .map(|s| (s.alpha, s.beta))
                .unwrap_or((1.0, 1.0));

            // Beta distribution sampling approximation
            let sample = Self::beta_sample(&mut rng, state.0, state.1);
            if sample > best_sample {
                best_sample = sample;
                best_variant = Some(variant.id);
            }
        }

        best_variant
    }

    fn ucb1(&self, campaign_id: &Uuid, variants: &[&VariantConfig]) -> Option<Uuid> {
        let total_impressions: u64 = variants
            .iter()
            .filter_map(|v| {
                self.states
                    .get(&(*campaign_id, v.id))
                    .map(|s| s.impressions)
            })
            .sum();

        if total_impressions == 0 {
            return variants.first().map(|v| v.id);
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_variant = None;
        let log_total = (total_impressions as f64).ln();

        for variant in variants {
            let state = self.states.get(&(*campaign_id, variant.id));
            let (impressions, conversions) = state
                .map(|s| (s.impressions, s.conversions))
                .unwrap_or((0, 0));

            if impressions == 0 {
                return Some(variant.id);
            }

            let avg_reward = conversions as f64 / impressions as f64;
            let exploration = (2.0 * log_total / impressions as f64).sqrt();
            let score = avg_reward + exploration;

            if score > best_score {
                best_score = score;
                best_variant = Some(variant.id);
            }
        }

        best_variant
    }

    fn epsilon_greedy(
        &self,
        campaign_id: &Uuid,
        variants: &[&VariantConfig],
        epsilon: f64,
    ) -> Option<Uuid> {
        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < epsilon {
            let idx = rng.gen_range(0..variants.len());
            return Some(variants[idx].id);
        }

        let mut best_rate = f64::NEG_INFINITY;
        let mut best_variant = None;

        for variant in variants {
            let state = self.states.get(&(*campaign_id, variant.id));
            let rate = state
                .map(|s| {
                    if s.impressions > 0 {
                        s.conversions as f64 / s.impressions as f64
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0);

            if rate > best_rate {
                best_rate = rate;
                best_variant = Some(variant.id);
            }
        }

        best_variant
    }

    pub fn record_impression(&self, campaign_id: &Uuid, variant_id: &Uuid) {
        if let Some(mut state) = self.states.get_mut(&(*campaign_id, *variant_id)) {
            state.impressions += 1;
            state.beta += 1.0;
        }
    }

    pub fn record_reward(&self, campaign_id: &Uuid, variant_id: &Uuid) {
        if let Some(mut state) = self.states.get_mut(&(*campaign_id, *variant_id)) {
            state.conversions += 1;
            state.alpha += 1.0;
            state.beta -= 1.0;
        }
    }

    pub fn get_stats(&self, campaign_id: &Uuid) -> Vec<VariantStats> {
        let config = match self.configs.get(campaign_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let total_impressions: u64 = config
            .variants
            .iter()
            .filter_map(|v| {
                self.states
                    .get(&(*campaign_id, v.id))
                    .map(|s| s.impressions)
            })
            .sum();

        let mut stats = Vec::new();
        let mut best_rate = 0.0f64;

        for variant in &config.variants {
            let state = self.states.get(&(*campaign_id, variant.id));
            let (impressions, conversions, alpha, beta) = state
                .map(|s| (s.impressions, s.conversions, s.alpha, s.beta))
                .unwrap_or((0, 0, 1.0, 1.0));

            let rate = if impressions > 0 {
                conversions as f64 / impressions as f64
            } else {
                0.0
            };

            let ci_width = if impressions > 0 {
                1.96 * (rate * (1.0 - rate) / impressions as f64).sqrt()
            } else {
                0.5
            };

            let traffic = if total_impressions > 0 {
                impressions as f64 / total_impressions as f64
            } else {
                1.0 / config.variants.len() as f64
            };

            if rate > best_rate {
                best_rate = rate;
            }

            stats.push(VariantStats {
                variant_id: variant.id,
                impressions,
                conversions,
                conversion_rate: rate,
                confidence_interval_lower: (rate - ci_width).max(0.0),
                confidence_interval_upper: (rate + ci_width).min(1.0),
                traffic_allocation: traffic,
                is_winner: false,
                estimated_value: alpha / (alpha + beta),
            });
        }

        for stat in &mut stats {
            if (stat.conversion_rate - best_rate).abs() < f64::EPSILON && stat.impressions > 100 {
                stat.is_winner = true;
            }
        }

        stats
    }

    fn beta_sample(rng: &mut impl Rng, alpha: f64, beta: f64) -> f64 {
        let x: f64 = (0..12).map(|_| rng.gen::<f64>()).sum::<f64>() - 6.0;
        let mean = alpha / (alpha + beta);
        let variance = (alpha * beta) / ((alpha + beta).powi(2) * (alpha + beta + 1.0));
        (mean + x * variance.sqrt()).clamp(0.0, 1.0)
    }
}

impl Default for BanditEngine {
    fn default() -> Self {
        Self::new()
    }
}
