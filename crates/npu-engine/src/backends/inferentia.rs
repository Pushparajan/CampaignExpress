//! AWS Inferentia 2 / 3 inference backend.
//!
//! Simulates inference on AWS Inferentia NeuronCores.  Inferentia 2 (inf2)
//! instances expose up to 12 NeuronCores; Inferentia 3 doubles the throughput
//! with improved pipeline parallelism.  In production, this backend would use
//! the AWS Neuron SDK (`libnrt`) to compile and execute the CoLaNet model on
//! NeuronCore hardware.

use campaign_core::inference::{CoLaNetProvider, InferenceError};
use campaign_core::types::{InferenceResult, UserProfile};
use serde::{Deserialize, Serialize};

/// Inferentia hardware generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InferentiaGeneration {
    /// AWS Inferentia 2 (inf2 instances).
    Inf2,
    /// AWS Inferentia 3 (inf3 instances).
    Inf3,
}

/// AWS Inferentia inference backend.
pub struct InferentiaBackend {
    device_id: String,
    #[allow(dead_code)]
    model_path: String,
    #[allow(dead_code)]
    neuron_cores: u32,
    generation: InferentiaGeneration,
    max_batch: usize,
}

impl InferentiaBackend {
    /// Create a new Inferentia backend.
    ///
    /// * `model_path` — path to the Neuron-compiled model artifact.
    /// * `device_id` — NeuronCore device identifier (e.g. "nd0").
    /// * `generation` — Inf2 or Inf3 hardware generation.
    pub fn new(model_path: String, device_id: String, generation: InferentiaGeneration) -> Self {
        let (neuron_cores, max_batch) = match generation {
            InferentiaGeneration::Inf2 => (12, 16),
            InferentiaGeneration::Inf3 => (24, 32),
        };
        Self {
            device_id,
            model_path,
            neuron_cores,
            generation,
            max_batch,
        }
    }
}

impl CoLaNetProvider for InferentiaBackend {
    fn predict(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> Result<Vec<InferenceResult>, InferenceError> {
        // Simulated NeuronCore latency: 50us for Inf3, 80us for Inf2.
        let latency_us: u64 = match self.generation {
            InferentiaGeneration::Inf2 => 80,
            InferentiaGeneration::Inf3 => 50,
        };

        let results = offer_ids
            .iter()
            .enumerate()
            .map(|(i, offer_id)| {
                let score = synthetic_score(&profile.user_id, offer_id, i);
                let predicted_ctr = sigmoid(score);
                let recommended_bid = (predicted_ctr as f64) * 11.0;
                InferenceResult {
                    offer_id: offer_id.clone(),
                    score,
                    predicted_ctr,
                    recommended_bid,
                    latency_us,
                }
            })
            .collect();

        Ok(results)
    }

    fn predict_batch(
        &self,
        requests: Vec<(UserProfile, Vec<String>)>,
    ) -> Result<Vec<Vec<InferenceResult>>, InferenceError> {
        if requests.len() > self.max_batch {
            return Err(InferenceError::BatchTooLarge {
                max: self.max_batch,
                got: requests.len(),
            });
        }

        // NeuronCore pipeline: ~30us per item in batch.
        let per_item_us: u64 = 30;

        let results = requests
            .iter()
            .enumerate()
            .map(|(req_idx, (profile, offer_ids))| {
                let latency_us = (req_idx as u64 + 1) * per_item_us;
                offer_ids
                    .iter()
                    .enumerate()
                    .map(|(i, offer_id)| {
                        let score = synthetic_score(&profile.user_id, offer_id, i);
                        let predicted_ctr = sigmoid(score);
                        let recommended_bid = (predicted_ctr as f64) * 11.0;
                        InferenceResult {
                            offer_id: offer_id.clone(),
                            score,
                            predicted_ctr,
                            recommended_bid,
                            latency_us,
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(results)
    }

    fn provider_name(&self) -> &str {
        match self.generation {
            InferentiaGeneration::Inf2 => "aws_inferentia_v2",
            InferentiaGeneration::Inf3 => "aws_inferentia_v3",
        }
    }

    fn supports_batching(&self) -> bool {
        true
    }

    fn max_batch_size(&self) -> usize {
        self.max_batch
    }

    fn warm_up(&self) -> Result<(), InferenceError> {
        if self.device_id.is_empty() {
            return Err(InferenceError::HardwareUnavailable(
                "Inferentia device ID not configured".to_string(),
            ));
        }
        Ok(())
    }
}

/// Generate a deterministic synthetic score from user/offer identifiers.
fn synthetic_score(user_id: &str, offer_id: &str, position: usize) -> f32 {
    let user_hash: u32 = user_id
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let offer_hash: u32 = offer_id
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u32));
    let combined = user_hash
        .wrapping_add(offer_hash)
        .wrapping_add(position as u32);
    ((combined % 2000) as f32 - 1000.0) / 1000.0
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
