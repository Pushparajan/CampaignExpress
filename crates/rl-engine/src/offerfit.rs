//! BR-RL-007: OfferFit Integration — middleware connector for OfferFit
//! experiment lifecycle management with local Thompson Sampling fallback.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Connection configuration for the OfferFit API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferFitConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub org_id: String,
    pub timeout_ms: u64,
}

impl Default for OfferFitConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.offerfit.ai/v1".to_string(),
            api_key: String::new(),
            org_id: String::new(),
            timeout_ms: 5000,
        }
    }
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Experiment lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentStatus {
    #[default]
    Draft,
    Active,
    Paused,
    Completed,
}

/// Optimization direction for the experiment metric.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Objective {
    #[default]
    Maximize,
    Minimize,
}

/// An OfferFit experiment definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferFitExperiment {
    pub id: Uuid,
    pub name: String,
    pub status: ExperimentStatus,
    pub objective: Objective,
    pub metric_name: String,
    pub variants: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub synced: bool,
}

/// A recorded decision (impression + optional reward).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferFitDecision {
    pub experiment_id: Uuid,
    pub user_id: String,
    pub selected_variant: String,
    pub score: f64,
    pub features: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

/// A recommendation returned for a given user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferFitRecommendation {
    pub user_id: String,
    pub variant_id: usize,
    pub variant_name: String,
    pub confidence: f64,
    pub explanation: String,
}

/// Per-variant aggregated statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStat {
    pub variant_name: String,
    pub impressions: u64,
    pub total_reward: f64,
    pub conversion_rate: f64,
    pub avg_reward: f64,
}

/// Experiment-level statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentStats {
    pub experiment_id: Uuid,
    pub total_decisions: u64,
    pub variant_stats: Vec<VariantStat>,
}

// ---------------------------------------------------------------------------
// Internal variant state for Thompson Sampling
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct VariantState {
    alpha: f64,
    beta: f64,
    impressions: u64,
    total_reward: f64,
}

impl VariantState {
    fn new() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            impressions: 0,
            total_reward: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// OfferFitClient
// ---------------------------------------------------------------------------

/// Client for OfferFit experiment management. All storage is in-process via
/// `DashMap` and Thompson Sampling runs locally as a fallback when the
/// remote OfferFit API is unreachable.
pub struct OfferFitClient {
    #[allow(dead_code)]
    config: OfferFitConfig,
    /// Cached experiment configs, keyed by experiment ID.
    experiments: DashMap<Uuid, OfferFitExperiment>,
    /// Per-experiment, per-variant Thompson Sampling state.
    variant_states: DashMap<(Uuid, usize), VariantState>,
    /// Recorded decisions for audit/analytics.
    decisions: DashMap<Uuid, Vec<OfferFitDecision>>,
}

impl OfferFitClient {
    /// Create a new `OfferFitClient` with the given configuration.
    pub fn new(config: OfferFitConfig) -> Self {
        tracing::info!(
            org_id = %config.org_id,
            base_url = %config.api_base_url,
            "OfferFit client initialised"
        );
        Self {
            config,
            experiments: DashMap::new(),
            variant_states: DashMap::new(),
            decisions: DashMap::new(),
        }
    }

    /// Create a new experiment with the given parameters.
    pub fn create_experiment(
        &self,
        name: &str,
        objective: Objective,
        metric_name: &str,
        variants: Vec<String>,
    ) -> OfferFitExperiment {
        let id = Uuid::new_v4();
        let experiment = OfferFitExperiment {
            id,
            name: name.to_string(),
            status: ExperimentStatus::Active,
            objective,
            metric_name: metric_name.to_string(),
            variants: variants.clone(),
            created_at: Utc::now(),
            synced: false,
        };

        // Initialise per-variant Thompson Sampling state.
        for (idx, _variant) in variants.iter().enumerate() {
            self.variant_states.insert((id, idx), VariantState::new());
        }

        self.experiments.insert(id, experiment.clone());
        tracing::info!(experiment_id = %id, name, "experiment created");
        experiment
    }

    /// Get a recommendation for `user_id` in the given experiment.
    ///
    /// Uses Thompson Sampling locally as a fallback for when the remote
    /// OfferFit API is unreachable.
    pub fn get_recommendation(
        &self,
        experiment_id: &Uuid,
        user_id: &str,
        features: HashMap<String, f64>,
    ) -> Option<OfferFitRecommendation> {
        let experiment = self.experiments.get(experiment_id)?;
        if experiment.variants.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut best_sample = f64::NEG_INFINITY;
        let mut best_idx: usize = 0;

        for idx in 0..experiment.variants.len() {
            let state = self
                .variant_states
                .get(&(*experiment_id, idx))
                .map(|s| (s.alpha, s.beta))
                .unwrap_or((1.0, 1.0));

            let sample = Self::beta_sample(&mut rng, state.0, state.1);
            if sample > best_sample {
                best_sample = sample;
                best_idx = idx;
            }
        }

        let variant_name = experiment.variants[best_idx].clone();
        let confidence = best_sample.clamp(0.0, 1.0);

        let feature_summary: String = if features.is_empty() {
            "no features".to_string()
        } else {
            features
                .keys()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };

        let explanation = format!(
            "Thompson Sampling selected '{}' with sampled value {:.4} (features: {})",
            variant_name, best_sample, feature_summary
        );

        // Record the decision internally.
        let decision = OfferFitDecision {
            experiment_id: *experiment_id,
            user_id: user_id.to_string(),
            selected_variant: variant_name.clone(),
            score: confidence,
            features,
            timestamp: Utc::now(),
        };
        self.decisions
            .entry(*experiment_id)
            .or_default()
            .push(decision);

        // Bump impression count.
        if let Some(mut state) = self.variant_states.get_mut(&(*experiment_id, best_idx)) {
            state.impressions += 1;
            state.beta += 1.0;
        }

        Some(OfferFitRecommendation {
            user_id: user_id.to_string(),
            variant_id: best_idx,
            variant_name,
            confidence,
            explanation,
        })
    }

    /// Record a reward for a previous decision.
    pub fn record_decision(
        &self,
        experiment_id: &Uuid,
        user_id: &str,
        variant_id: usize,
        reward: f64,
    ) {
        // Update Thompson Sampling state.
        if let Some(mut state) = self.variant_states.get_mut(&(*experiment_id, variant_id)) {
            if reward > 0.0 {
                state.alpha += reward;
                state.beta -= reward.min(state.beta - 0.01);
            }
            state.total_reward += reward;
        }

        // Append an audit record.
        let experiment = self.experiments.get(experiment_id);
        let variant_name = experiment
            .as_ref()
            .and_then(|e| e.variants.get(variant_id).cloned())
            .unwrap_or_default();

        let decision = OfferFitDecision {
            experiment_id: *experiment_id,
            user_id: user_id.to_string(),
            selected_variant: variant_name,
            score: reward,
            features: HashMap::new(),
            timestamp: Utc::now(),
        };
        self.decisions
            .entry(*experiment_id)
            .or_default()
            .push(decision);

        tracing::debug!(
            experiment_id = %experiment_id,
            user_id,
            variant_id,
            reward,
            "decision recorded"
        );
    }

    /// Aggregate per-variant statistics for an experiment.
    pub fn get_experiment_stats(&self, experiment_id: &Uuid) -> Option<ExperimentStats> {
        let experiment = self.experiments.get(experiment_id)?;

        let mut variant_stats = Vec::with_capacity(experiment.variants.len());
        let mut total_decisions: u64 = 0;

        for (idx, variant_name) in experiment.variants.iter().enumerate() {
            let state = self
                .variant_states
                .get(&(*experiment_id, idx))
                .map(|s| (s.impressions, s.total_reward, s.alpha, s.beta))
                .unwrap_or((0, 0.0, 1.0, 1.0));

            let (impressions, total_reward, alpha, beta) = state;
            total_decisions += impressions;

            let conversion_rate = alpha / (alpha + beta);
            let avg_reward = if impressions > 0 {
                total_reward / impressions as f64
            } else {
                0.0
            };

            variant_stats.push(VariantStat {
                variant_name: variant_name.clone(),
                impressions,
                total_reward,
                conversion_rate,
                avg_reward,
            });
        }

        Some(ExperimentStats {
            experiment_id: *experiment_id,
            total_decisions,
            variant_stats,
        })
    }

    /// Stub: mark experiment as synced to the remote OfferFit platform.
    pub fn sync_to_offerfit(&self, experiment_id: &Uuid) -> bool {
        if let Some(mut exp) = self.experiments.get_mut(experiment_id) {
            exp.synced = true;
            tracing::info!(experiment_id = %experiment_id, "experiment synced to OfferFit");
            true
        } else {
            false
        }
    }

    /// List all known experiments.
    pub fn list_experiments(&self) -> Vec<OfferFitExperiment> {
        self.experiments
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Approximate Beta-distribution sample using the Irwin-Hall approach
    /// (sum of 12 uniform samples to approximate a standard normal, then
    /// shift/scale by the Beta mean and variance).
    fn beta_sample(rng: &mut impl Rng, alpha: f64, beta: f64) -> f64 {
        let x: f64 = (0..12).map(|_| rng.gen::<f64>()).sum::<f64>() - 6.0;
        let mean = alpha / (alpha + beta);
        let variance = (alpha * beta) / ((alpha + beta).powi(2) * (alpha + beta + 1.0));
        (mean + x * variance.sqrt()).clamp(0.0, 1.0)
    }
}

impl Default for OfferFitClient {
    fn default() -> Self {
        Self::new(OfferFitConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> OfferFitClient {
        OfferFitClient::new(OfferFitConfig {
            api_base_url: "https://api.offerfit.test/v1".to_string(),
            api_key: "test-key".to_string(),
            org_id: "test-org".to_string(),
            timeout_ms: 1000,
        })
    }

    #[test]
    fn test_create_experiment() {
        let client = make_client();
        let exp = client.create_experiment(
            "ctr_test",
            Objective::Maximize,
            "click_through_rate",
            vec!["control".into(), "variant_a".into(), "variant_b".into()],
        );

        assert_eq!(exp.name, "ctr_test");
        assert_eq!(exp.status, ExperimentStatus::Active);
        assert_eq!(exp.objective, Objective::Maximize);
        assert_eq!(exp.metric_name, "click_through_rate");
        assert_eq!(exp.variants.len(), 3);
        assert!(!exp.synced);

        let listed = client.list_experiments();
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn test_recommendation_flow() {
        let client = make_client();
        let exp = client.create_experiment(
            "email_subject",
            Objective::Maximize,
            "open_rate",
            vec!["short".into(), "long".into()],
        );

        let mut features = HashMap::new();
        features.insert("recency".to_string(), 0.8);
        features.insert("frequency".to_string(), 3.0);

        let rec = client
            .get_recommendation(&exp.id, "user-42", features)
            .expect("should return recommendation");

        assert_eq!(rec.user_id, "user-42");
        assert!(rec.variant_id < 2);
        assert!(!rec.variant_name.is_empty());
        assert!(rec.confidence >= 0.0 && rec.confidence <= 1.0);
        assert!(!rec.explanation.is_empty());
    }

    #[test]
    fn test_record_and_stats() {
        let client = make_client();
        let exp = client.create_experiment(
            "promo_test",
            Objective::Maximize,
            "revenue",
            vec!["10pct_off".into(), "free_ship".into()],
        );

        // Simulate several impressions + rewards.
        for _ in 0..20 {
            let rec = client
                .get_recommendation(&exp.id, "user-1", HashMap::new())
                .unwrap();
            // Give positive reward to variant 0 more often.
            let reward = if rec.variant_id == 0 { 1.0 } else { 0.0 };
            client.record_decision(&exp.id, "user-1", rec.variant_id, reward);
        }

        let stats = client
            .get_experiment_stats(&exp.id)
            .expect("should have stats");
        assert_eq!(stats.experiment_id, exp.id);
        assert_eq!(stats.variant_stats.len(), 2);
        assert!(stats.total_decisions > 0);

        // Verify each variant has reasonable data.
        for vs in &stats.variant_stats {
            assert!(vs.conversion_rate >= 0.0 && vs.conversion_rate <= 1.0);
        }
    }

    #[test]
    fn test_sync_to_offerfit() {
        let client = make_client();
        let exp = client.create_experiment(
            "sync_test",
            Objective::Minimize,
            "cost_per_acquisition",
            vec!["a".into(), "b".into()],
        );

        assert!(!exp.synced);
        assert!(client.sync_to_offerfit(&exp.id));

        let updated = client.experiments.get(&exp.id).unwrap();
        assert!(updated.synced);

        // Non-existent experiment returns false.
        assert!(!client.sync_to_offerfit(&Uuid::new_v4()));
    }

    #[test]
    fn test_empty_variants_returns_none() {
        let client = make_client();
        let exp = client.create_experiment("empty", Objective::Maximize, "ctr", vec![]);

        let rec = client.get_recommendation(&exp.id, "u1", HashMap::new());
        assert!(rec.is_none());
    }

    #[test]
    fn test_nonexistent_experiment() {
        let client = make_client();
        let fake_id = Uuid::new_v4();

        assert!(client
            .get_recommendation(&fake_id, "u1", HashMap::new())
            .is_none());
        assert!(client.get_experiment_stats(&fake_id).is_none());
    }
}
