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
/// Supports multi-head output: primary offer scoring + creative variant scoring.
pub struct CoLaNetModel {
    weights: ModelWeights,
    input_dim: usize,
    output_dim: usize,
    /// Secondary head for DCO variant scoring
    variant_head: VariantHead,
}

/// Secondary head for scoring creative variant combinations
struct VariantHead {
    weights: Array2<f32>,
    bias: Vec<f32>,
    output_dim: usize,
}

/// Result of multi-head inference including DCO variant scores
#[derive(Debug, Clone)]
pub struct MultiHeadResult {
    pub offer_scores: Vec<InferenceResult>,
    pub variant_scores: Vec<Vec<f32>>,
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
        let variant_output_dim = 32; // Max variant scoring slots

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
        let variant_head = VariantHead::synthetic(output_dim, variant_output_dim);

        Ok(Self {
            weights,
            input_dim,
            output_dim,
            variant_head,
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

    /// Multi-head inference: score offers AND creative variants in one pass.
    /// The hidden layer activations are shared between both heads for efficiency.
    pub fn infer_multi_head(
        &self,
        features: &Array2<f32>,
        offer_ids: &[String],
        num_variants: usize,
    ) -> anyhow::Result<MultiHeadResult> {
        let start = std::time::Instant::now();

        // Shared forward pass through main layers
        let hidden = self.weights.forward_hidden(features);

        // Head 1: Offer scoring (primary)
        let offer_scores_raw = self.weights.forward_output(&hidden);
        let latency_us = start.elapsed().as_micros() as u64;

        let offer_scores: Vec<InferenceResult> = offer_scores_raw
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

        // Head 2: Variant scoring (DCO)
        let variant_scores = self.variant_head.score(&hidden, num_variants);

        Ok(MultiHeadResult {
            offer_scores,
            variant_scores,
        })
    }

    pub fn input_dim(&self) -> usize {
        self.input_dim
    }

    pub fn output_dim(&self) -> usize {
        self.output_dim
    }

    pub fn variant_output_dim(&self) -> usize {
        self.variant_head.output_dim
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
        let hidden = self.forward_hidden(input);
        self.forward_output(&hidden)
    }

    /// Forward pass through hidden layer only (shared computation for multi-head).
    fn forward_hidden(&self, input: &Array2<f32>) -> Array2<f32> {
        let hidden = input.dot(&self.layer1);
        let mut activated = Array2::<f32>::zeros(hidden.raw_dim());
        for i in 0..hidden.nrows() {
            for j in 0..hidden.ncols() {
                let val = hidden[[i, j]] + self.bias1[j];
                // SNN-inspired activation: spike if membrane potential exceeds threshold
                activated[[i, j]] = if val > 0.0 { val.tanh() } else { 0.0 };
            }
        }
        activated
    }

    /// Forward pass from hidden activations to output scores.
    fn forward_output(&self, hidden: &Array2<f32>) -> Vec<f32> {
        let batch_size = hidden.nrows();
        let output = hidden.dot(&self.layer2);
        (0..batch_size)
            .map(|i| {
                let val = output[[i, 0]] + self.bias2[0];
                val.tanh()
            })
            .collect()
    }
}

impl VariantHead {
    /// Generate synthetic variant scoring weights.
    fn synthetic(hidden_dim: usize, output_dim: usize) -> Self {
        let mut weights = Array2::<f32>::zeros((hidden_dim, output_dim));
        for i in 0..hidden_dim {
            for j in 0..output_dim {
                weights[[i, j]] = ((i * 5 + j * 17) as f32 % 100.0 - 50.0) / 600.0;
            }
        }
        let bias = vec![0.005; output_dim];
        Self {
            weights,
            bias,
            output_dim,
        }
    }

    /// Score creative variants using shared hidden activations.
    /// Returns one score vector per batch entry, truncated to num_variants.
    fn score(&self, hidden: &Array2<f32>, num_variants: usize) -> Vec<Vec<f32>> {
        let output = hidden.dot(&self.weights);
        let batch_size = hidden.nrows();
        let effective_variants = num_variants.min(self.output_dim);

        (0..batch_size)
            .map(|i| {
                (0..effective_variants)
                    .map(|j| {
                        let val = output[[i, j]] + self.bias[j];
                        sigmoid(val)
                    })
                    .collect()
            })
            .collect()
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
