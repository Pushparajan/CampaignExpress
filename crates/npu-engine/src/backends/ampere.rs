//! Oracle Ampere Altra ARM CPU inference backend.
//!
//! Simulates inference on Oracle Cloud Ampere Altra processors with up to
//! 128 ARM Neoverse N1 cores and NEON SIMD.  Batch inference is parallelised
//! across cores for throughput.

use campaign_core::inference::{CoLaNetProvider, InferenceError};
use campaign_core::types::{InferenceResult, UserProfile};

/// Oracle Ampere Altra ARM CPU inference backend.
pub struct AmpereBackend {
    #[allow(dead_code)]
    model_path: String,
    num_cores: u32,
    #[allow(dead_code)]
    thread_pool_size: usize,
    max_batch: usize,
}

impl AmpereBackend {
    /// Create a new Ampere Altra backend.
    ///
    /// * `model_path` — path to the model artifact.
    /// * `num_cores` — number of ARM cores available (up to 128).
    /// * `thread_pool_size` — worker threads for parallel inference.
    pub fn new(model_path: String, num_cores: u32, thread_pool_size: usize) -> Self {
        let max_batch = (num_cores as usize) / 2;
        Self {
            model_path,
            num_cores,
            thread_pool_size,
            max_batch,
        }
    }
}

impl CoLaNetProvider for AmpereBackend {
    fn predict(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> Result<Vec<InferenceResult>, InferenceError> {
        // Simulated ARM NEON SIMD inference: ~200us latency.
        let latency_us: u64 = 200;

        let results = offer_ids
            .iter()
            .enumerate()
            .map(|(i, offer_id)| {
                let score = synthetic_score(&profile.user_id, offer_id, i);
                let predicted_ctr = sigmoid(score);
                let recommended_bid = (predicted_ctr as f64) * 9.5;
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

        // Parallel processing across ARM cores: ~150us per item in batch.
        let per_item_us: u64 = 150;

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
                        let recommended_bid = (predicted_ctr as f64) * 9.5;
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
        "oracle_ampere_altra"
    }

    fn supports_batching(&self) -> bool {
        true
    }

    fn max_batch_size(&self) -> usize {
        self.max_batch
    }

    fn warm_up(&self) -> Result<(), InferenceError> {
        if self.num_cores == 0 {
            return Err(InferenceError::HardwareUnavailable(
                "Ampere Altra: num_cores must be > 0".to_string(),
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
