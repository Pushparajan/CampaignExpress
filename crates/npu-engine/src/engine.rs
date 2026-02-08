//! NPU inference engine — manages model lifecycle, batching, and
//! provides the high-level inference API used by agents.

use crate::model::CoLaNetModel;
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
    fn build_features(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
        input_dim: usize,
    ) -> Array2<f32> {
        let batch_size = offer_ids.len();
        let mut features = Array2::<f32>::zeros((batch_size, input_dim));

        for (i, _offer_id) in offer_ids.iter().enumerate() {
            let mut row = features.row_mut(i);

            // Encode user interests (first N features)
            for (j, &interest) in profile.interests.iter().enumerate() {
                if j >= input_dim / 2 {
                    break;
                }
                row[j] = interest;
            }

            // Encode user segments as one-hot-ish features
            for &seg in &profile.segments {
                let idx = (input_dim / 2) + (seg as usize % (input_dim / 4));
                if idx < input_dim {
                    row[idx] = 1.0;
                }
            }

            // Encode recency
            if input_dim > 4 {
                row[input_dim - 4] = profile.recency_score;
            }

            // Encode frequency cap utilization
            if input_dim > 3 {
                let freq_util = if profile.frequency_cap.max_per_hour > 0 {
                    profile.frequency_cap.impressions_1h as f32
                        / profile.frequency_cap.max_per_hour as f32
                } else {
                    0.0
                };
                row[input_dim - 3] = freq_util;
            }

            // Encode device type
            if input_dim > 2 {
                row[input_dim - 2] = match profile.device_type {
                    Some(campaign_core::types::DeviceType::Desktop) => 0.0,
                    Some(campaign_core::types::DeviceType::Mobile) => 1.0,
                    Some(campaign_core::types::DeviceType::Tablet) => 0.5,
                    Some(campaign_core::types::DeviceType::Ctv) => 0.75,
                    None => -1.0,
                };
            }

            // Offer index encoding (simple positional)
            if input_dim > 1 {
                row[input_dim - 1] = i as f32 / batch_size.max(1) as f32;
            }
        }

        features
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
