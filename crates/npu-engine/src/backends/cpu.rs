//! CPU backend — wraps the existing [`NpuEngine`] for local / development inference.
//!
//! No hardware acceleration; offers are scored sequentially using the
//! synthetic-weight CoLaNet model on the host CPU.

use campaign_core::config::NpuConfig;
use campaign_core::inference::{CoLaNetProvider, InferenceError};
use campaign_core::types::{InferenceResult, UserProfile};

use crate::engine::NpuEngine;

/// CPU inference backend backed by the existing [`NpuEngine`].
pub struct CpuBackend {
    engine: NpuEngine,
}

impl CpuBackend {
    /// Create a new CPU backend from the given NPU configuration.
    pub fn new(config: &NpuConfig) -> Result<Self, InferenceError> {
        let engine =
            NpuEngine::new(config).map_err(|e| InferenceError::ModelNotLoaded(e.to_string()))?;
        Ok(Self { engine })
    }
}

impl CoLaNetProvider for CpuBackend {
    fn predict(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> Result<Vec<InferenceResult>, InferenceError> {
        self.engine
            .score_offers(profile, offer_ids)
            .map_err(|e| InferenceError::InferenceFailure(e.to_string()))
    }

    fn predict_batch(
        &self,
        requests: Vec<(UserProfile, Vec<String>)>,
    ) -> Result<Vec<Vec<InferenceResult>>, InferenceError> {
        // CPU: sequential iteration — no batch optimization available.
        requests
            .iter()
            .map(|(profile, offers)| self.predict(profile, offers))
            .collect()
    }

    fn provider_name(&self) -> &str {
        "cpu_synthetic"
    }

    fn supports_batching(&self) -> bool {
        false
    }

    fn max_batch_size(&self) -> usize {
        1
    }

    fn warm_up(&self) -> Result<(), InferenceError> {
        Ok(())
    }
}
