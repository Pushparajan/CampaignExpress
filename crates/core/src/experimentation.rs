//! Experimentation framework types and utilities.

use crate::types::{Experiment, ExperimentStatus, ExperimentVariant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Experimentation engine for A/B/n testing
pub struct ExperimentEngine {
    experiments: std::collections::HashMap<Uuid, Experiment>,
}

impl ExperimentEngine {
    pub fn new() -> Self {
        Self {
            experiments: std::collections::HashMap::new(),
        }
    }

    pub fn create_experiment(&mut self, experiment: Experiment) -> Uuid {
        let id = experiment.id;
        self.experiments.insert(id, experiment);
        id
    }

    pub fn get_experiment(&self, id: &Uuid) -> Option<&Experiment> {
        self.experiments.get(id)
    }

    pub fn list_experiments(&self) -> Vec<&Experiment> {
        self.experiments.values().collect()
    }

    pub fn assign_variant(&self, experiment_id: &Uuid, user_id: &str) -> Option<Uuid> {
        let experiment = self.experiments.get(experiment_id)?;
        if experiment.status != ExperimentStatus::Running {
            return None;
        }
        // Deterministic assignment based on user_id hash
        let hash = user_id
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let normalized = (hash % 10000) as f64 / 10000.0;

        let mut cumulative = 0.0;
        for variant in &experiment.variants {
            cumulative += variant.weight;
            if normalized < cumulative {
                return Some(variant.id);
            }
        }
        experiment.variants.last().map(|v| v.id)
    }

    pub fn record_conversion(&mut self, experiment_id: &Uuid, variant_id: &Uuid, revenue: f64) {
        if let Some(experiment) = self.experiments.get_mut(experiment_id) {
            for variant in &mut experiment.variants {
                if variant.id == *variant_id {
                    variant.results.sample_size += 1;
                    variant.results.conversions += 1;
                    variant.results.revenue += revenue;
                    if variant.results.sample_size > 0 {
                        variant.results.conversion_rate =
                            variant.results.conversions as f64 / variant.results.sample_size as f64;
                    }
                    break;
                }
            }
        }
    }

    /// Check if experiment has reached statistical significance
    pub fn check_significance(&self, experiment_id: &Uuid) -> Option<SignificanceResult> {
        let experiment = self.experiments.get(experiment_id)?;
        let control = experiment.variants.iter().find(|v| v.is_control)?;
        let mut best_variant: Option<&ExperimentVariant> = None;
        let mut best_lift = 0.0f64;

        for variant in &experiment.variants {
            if variant.is_control {
                continue;
            }
            if control.results.conversion_rate > 0.0 {
                let lift = (variant.results.conversion_rate - control.results.conversion_rate)
                    / control.results.conversion_rate;
                if lift > best_lift {
                    best_lift = lift;
                    best_variant = Some(variant);
                }
            }
        }

        let total_samples: u64 = experiment
            .variants
            .iter()
            .map(|v| v.results.sample_size)
            .sum();
        let is_significant = total_samples >= experiment.min_sample_size;

        Some(SignificanceResult {
            experiment_id: experiment.id,
            is_significant,
            best_variant_id: best_variant.map(|v| v.id),
            best_lift: best_lift,
            total_samples,
            required_samples: experiment.min_sample_size,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceResult {
    pub experiment_id: Uuid,
    pub is_significant: bool,
    pub best_variant_id: Option<Uuid>,
    pub best_lift: f64,
    pub total_samples: u64,
    pub required_samples: u64,
}

impl Default for ExperimentEngine {
    fn default() -> Self {
        Self::new()
    }
}
