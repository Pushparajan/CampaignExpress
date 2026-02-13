//! Groq LPU Cloud inference backend.
//!
//! Simulates inference through the Groq Language Processing Unit cloud API.
//! Groq LPUs excel at throughput-optimised batch inference with ultra-low
//! latency per token.  In production, this backend would issue HTTP requests
//! to the Groq API; here it generates deterministic synthetic scores.

use campaign_core::inference::{CoLaNetProvider, InferenceError};
use campaign_core::types::{InferenceResult, UserProfile};

/// Groq LPU Cloud inference backend.
pub struct GroqBackend {
    api_endpoint: String,
    #[allow(dead_code)]
    api_key: String,
    model_id: String,
    max_batch: usize,
}

impl GroqBackend {
    /// Create a new Groq backend targeting the given API endpoint.
    pub fn new(api_endpoint: String, api_key: String, model_id: String) -> Self {
        Self {
            api_endpoint,
            api_key,
            model_id,
            max_batch: 64,
        }
    }
}

impl CoLaNetProvider for GroqBackend {
    fn predict(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> Result<Vec<InferenceResult>, InferenceError> {
        // Simulate Groq API call with ~100us latency per request.
        let latency_us: u64 = 100;

        let results = offer_ids
            .iter()
            .enumerate()
            .map(|(i, offer_id)| {
                let score = synthetic_score(&profile.user_id, offer_id, i);
                let predicted_ctr = sigmoid(score);
                let recommended_bid = (predicted_ctr as f64) * 12.0;
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

        // Groq LPU processes the entire batch efficiently.
        // Simulated: 80us base + 5us per request in batch.
        let per_item_us: u64 = 5;
        let base_us: u64 = 80;

        let results = requests
            .iter()
            .enumerate()
            .map(|(req_idx, (profile, offer_ids))| {
                let latency_us = base_us + (req_idx as u64) * per_item_us;
                offer_ids
                    .iter()
                    .enumerate()
                    .map(|(i, offer_id)| {
                        let score = synthetic_score(&profile.user_id, offer_id, i);
                        let predicted_ctr = sigmoid(score);
                        let recommended_bid = (predicted_ctr as f64) * 12.0;
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
        "groq_lpu"
    }

    fn supports_batching(&self) -> bool {
        true
    }

    fn max_batch_size(&self) -> usize {
        self.max_batch
    }

    fn warm_up(&self) -> Result<(), InferenceError> {
        // Validate that the API endpoint and model ID are set.
        if self.api_endpoint.is_empty() {
            return Err(InferenceError::HardwareUnavailable(
                "Groq API endpoint not configured".to_string(),
            ));
        }
        if self.model_id.is_empty() {
            return Err(InferenceError::HardwareUnavailable(
                "Groq model ID not configured".to_string(),
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
    // Map to [-1.0, 1.0]
    ((combined % 2000) as f32 - 1000.0) / 1000.0
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
