//! Tenstorrent Wormhole / Grayskull RISC-V mesh inference backend.
//!
//! Simulates inference on a Tenstorrent mesh of RISC-V Tensix cores
//! arranged in a 2-D grid.  The mesh architecture enables massive
//! data-parallel inference where the batch is distributed across
//! (rows x cols) cores simultaneously.

use campaign_core::inference::{CoLaNetProvider, InferenceError};
use campaign_core::types::{InferenceResult, UserProfile};

/// Tenstorrent RISC-V mesh inference backend.
pub struct TenstorrentBackend {
    device_id: String,
    #[allow(dead_code)]
    model_path: String,
    #[allow(dead_code)]
    mesh_rows: u32,
    #[allow(dead_code)]
    mesh_cols: u32,
    max_batch: usize,
}

impl TenstorrentBackend {
    /// Create a new Tenstorrent backend.
    ///
    /// * `model_path` — path to the compiled model artifact.
    /// * `device_id` — Tenstorrent device identifier.
    /// * `mesh_rows` — number of rows in the Tensix core mesh.
    /// * `mesh_cols` — number of columns in the Tensix core mesh.
    pub fn new(model_path: String, device_id: String, mesh_rows: u32, mesh_cols: u32) -> Self {
        let max_batch = (mesh_rows * mesh_cols) as usize;
        Self {
            device_id,
            model_path,
            mesh_rows,
            mesh_cols,
            max_batch,
        }
    }
}

impl CoLaNetProvider for TenstorrentBackend {
    fn predict(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> Result<Vec<InferenceResult>, InferenceError> {
        // Simulated Wormhole/Grayskull single-request latency: ~40us.
        let latency_us: u64 = 40;

        let results = offer_ids
            .iter()
            .enumerate()
            .map(|(i, offer_id)| {
                let score = synthetic_score(&profile.user_id, offer_id, i);
                let predicted_ctr = sigmoid(score);
                let recommended_bid = (predicted_ctr as f64) * 13.0;
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

        // Mesh-parallel processing: ~20us per item in batch.
        let per_item_us: u64 = 20;

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
                        let recommended_bid = (predicted_ctr as f64) * 13.0;
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
        "tenstorrent_wormhole"
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
                "Tenstorrent device ID not configured".to_string(),
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
