//! NPU inference engine — manages model lifecycle, batching, and
//! provides the high-level inference API used by agents.

use crate::model::{CoLaNetModel, MultiHeadResult};
use campaign_core::config::NpuConfig;
use campaign_core::types::{InferenceResult, UserProfile};
use ndarray::Array2;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info};

/// Thread-safe inference engine wrapping the CoLaNet model.
pub struct NpuEngine {
    model: Arc<RwLock<CoLaNetModel>>,
    config: NpuConfig,
}

impl NpuEngine {
    /// Initialize the engine: load the model and prepare for inference.
    pub fn new(config: &NpuConfig) -> anyhow::Result<Self> {
        let model = CoLaNetModel::load(&config.model_path, &config.device, config.num_threads)?;

        info!("NPU engine initialized (device={})", config.device);

        Ok(Self {
            model: Arc::new(RwLock::new(model)),
            config: config.clone(),
        })
    }

    /// Score a set of offers for a given user profile.
    ///
    /// Builds a feature matrix from the user profile and offer metadata,
    /// runs inference, and returns scored results.
    pub fn score_offers(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> anyhow::Result<Vec<InferenceResult>> {
        let batch_size = offer_ids.len();
        let input_dim = self.model.read().input_dim();

        debug!(
            batch_size = batch_size,
            input_dim = input_dim,
            "Building feature matrix for inference"
        );

        let features = self.build_features(profile, offer_ids, input_dim);
        let model = self.model.read();
        model.infer(&features, offer_ids)
    }

    /// Build a feature matrix from user profile and offer IDs.
    /// Each row is a feature vector for one user-offer pair.
    ///
    /// Layout (256 dims):
    ///   [0..64)   — user interests
    ///   [64..128) — segment one-hot encoding
    ///   [128..136) — loyalty features (tier, balance, progress, earn_rate, etc.)
    ///   [136..140) — context (recency, freq_cap, device, offer_position)
    ///   [140..256) — reserved / zero-padded for future features
    fn build_features(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
        input_dim: usize,
    ) -> Array2<f32> {
        let batch_size = offer_ids.len();
        let mut features = Array2::<f32>::zeros((batch_size, input_dim));

        // Pre-compute loyalty features (shared across all offers for this user)
        let loyalty_vec = profile
            .loyalty
            .as_ref()
            .map(|lp| lp.as_feature_vector())
            .unwrap_or([0.0; 8]);

        for (i, _offer_id) in offer_ids.iter().enumerate() {
            let mut row = features.row_mut(i);

            // [0..64) — user interests
            for (j, &interest) in profile.interests.iter().enumerate() {
                if j >= 64 {
                    break;
                }
                row[j] = interest;
            }

            // [64..128) — segment one-hot encoding
            for &seg in &profile.segments {
                let idx = 64 + (seg as usize % 64);
                row[idx] = 1.0;
            }

            // [128..136) — loyalty features
            for (j, &val) in loyalty_vec.iter().enumerate() {
                if 128 + j < input_dim {
                    row[128 + j] = val;
                }
            }

            // [136] — recency score
            if input_dim > 136 {
                row[136] = profile.recency_score;
            }

            // [137] — frequency cap utilization
            if input_dim > 137 {
                let freq_util = if profile.frequency_cap.max_per_hour > 0 {
                    profile.frequency_cap.impressions_1h as f32
                        / profile.frequency_cap.max_per_hour as f32
                } else {
                    0.0
                };
                row[137] = freq_util;
            }

            // [138] — device type
            if input_dim > 138 {
                row[138] = match profile.device_type {
                    Some(campaign_core::types::DeviceType::Desktop) => 0.0,
                    Some(campaign_core::types::DeviceType::Mobile) => 1.0,
                    Some(campaign_core::types::DeviceType::Tablet) => 0.5,
                    Some(campaign_core::types::DeviceType::Ctv) => 0.75,
                    None => -1.0,
                };
            }

            // [139] — offer positional encoding
            if input_dim > 139 {
                row[139] = i as f32 / batch_size.max(1) as f32;
            }
        }

        features
    }

    /// Multi-head scoring: score offers AND creative variants in a single pass.
    /// Used when DCO is enabled to select the best creative variant per offer.
    pub fn score_offers_with_variants(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
        num_variants: usize,
    ) -> anyhow::Result<MultiHeadResult> {
        let input_dim = self.model.read().input_dim();

        debug!(
            batch_size = offer_ids.len(),
            num_variants = num_variants,
            "Multi-head inference (offers + DCO variants)"
        );

        let features = self.build_features(profile, offer_ids, input_dim);
        let model = self.model.read();
        model.infer_multi_head(&features, offer_ids, num_variants)
    }

    /// Hot-reload a new model version without downtime.
    pub fn reload_model(&self, model_path: &str) -> anyhow::Result<()> {
        info!(path = model_path, "Hot-reloading model");
        let new_model =
            CoLaNetModel::load(model_path, &self.config.device, self.config.num_threads)?;
        let mut model = self.model.write();
        *model = new_model;
        info!("Model hot-reload complete");
        Ok(())
    }

    pub fn config(&self) -> &NpuConfig {
        &self.config
    }
}
