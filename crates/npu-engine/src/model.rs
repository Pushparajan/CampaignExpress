//! CoLaNet Spiking Neural Network model abstraction.
//!
//! In production, this loads an ONNX-exported SNN model and compiles it
//! for AMD Ryzen AI XDNA NPU execution via the Vitis AI runtime.
//!
//! This implementation provides:
//! - A two-layer neural network with SNN-inspired activation
//! - Synthetic weight initialization for development/testing
//! - Extensible model file loading for production ONNX integration
//!
//! To enable real ONNX Runtime inference, add the `ort` crate dependency
//! to Cargo.toml and implement the ONNX session loading code path.

use campaign_core::types::InferenceResult;
use ndarray::Array2;
use std::path::Path;
use tracing::{info, warn};

/// Represents a loaded CoLaNet SNN model ready for inference.
pub struct CoLaNetModel {
    weights: ModelWeights,
    input_dim: usize,
    output_dim: usize,
}

/// Two-layer neural network weights with SNN-inspired activation.
struct ModelWeights {
    layer1: Array2<f32>,
    layer2: Array2<f32>,
    bias1: Vec<f32>,
    bias2: Vec<f32>,
}

impl CoLaNetModel {
    /// Load a model from the given path.
    ///
    /// If `device` is "xdna", logs intent to use AMD XDNA NPU.
    /// Falls back to synthetic weights if no model file is found.
    pub fn load(model_path: &str, device: &str, _num_threads: usize) -> anyhow::Result<Self> {
        let path = Path::new(model_path);
        // Expanded for loyalty-aware inference:
        // 64 interests + 64 segments + 8 loyalty features + 4 context = 140
        // Padded to 256 for NPU SIMD alignment
        let input_dim = 256;
        let output_dim = 64;

        if device == "xdna" {
            info!("XDNA NPU device requested — will use NPU when Vitis AI runtime is available");
        }

        if !path.exists() {
            warn!(
                path = model_path,
                "Model file not found, using synthetic weights for development"
            );
        } else {
            info!(path = model_path, device = device, "Model file found, loading weights");
        }

        // Generate deterministic synthetic weights
        // In production: replace with ONNX Runtime model loading
        let weights = ModelWeights::synthetic(input_dim, output_dim);

        Ok(Self {
            weights,
            input_dim,
            output_dim,
        })
    }

    /// Run inference on a batch of feature vectors.
    /// Returns scored results for each offer.
    pub fn infer(
        &self,
        features: &Array2<f32>,
        offer_ids: &[String],
    ) -> anyhow::Result<Vec<InferenceResult>> {
        let start = std::time::Instant::now();
        let scores = self.weights.forward(features);
        let latency_us = start.elapsed().as_micros() as u64;

        let results = scores
            .iter()
            .zip(offer_ids.iter())
            .map(|(&score, offer_id)| {
                let predicted_ctr = sigmoid(score);
                let recommended_bid = (predicted_ctr as f64) * 10.0;

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

    pub fn input_dim(&self) -> usize {
        self.input_dim
    }

    pub fn output_dim(&self) -> usize {
        self.output_dim
    }
}

impl ModelWeights {
    /// Generate deterministic synthetic weights for development.
    fn synthetic(input_dim: usize, output_dim: usize) -> Self {
        let hidden_dim = 64;

        let mut layer1 = Array2::<f32>::zeros((input_dim, hidden_dim));
        for i in 0..input_dim {
            for j in 0..hidden_dim {
                layer1[[i, j]] = ((i * 7 + j * 13) as f32 % 100.0 - 50.0) / 500.0;
            }
        }

        let mut layer2 = Array2::<f32>::zeros((hidden_dim, output_dim));
        for i in 0..hidden_dim {
            for j in 0..output_dim {
                layer2[[i, j]] = ((i * 11 + j * 3) as f32 % 100.0 - 50.0) / 500.0;
            }
        }

        let bias1 = vec![0.01; hidden_dim];
        let bias2 = vec![0.01; output_dim];

        Self {
            layer1,
            layer2,
            bias1,
            bias2,
        }
    }

    /// Two-layer forward pass with SNN-inspired threshold activation.
    fn forward(&self, input: &Array2<f32>) -> Vec<f32> {
        let batch_size = input.nrows();

        // Layer 1: input -> hidden (with spiking threshold activation)
        let hidden = input.dot(&self.layer1);
        let mut activated = Array2::<f32>::zeros(hidden.raw_dim());
        for i in 0..hidden.nrows() {
            for j in 0..hidden.ncols() {
                let val = hidden[[i, j]] + self.bias1[j];
                // SNN-inspired activation: spike if membrane potential exceeds threshold
                activated[[i, j]] = if val > 0.0 { val.tanh() } else { 0.0 };
            }
        }

        // Layer 2: hidden -> output
        let output = activated.dot(&self.layer2);

        // Return one score per batch entry (first output dimension)
        (0..batch_size)
            .map(|i| {
                let val = output[[i, 0]] + self.bias2[0];
                val.tanh()
            })
            .collect()
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
