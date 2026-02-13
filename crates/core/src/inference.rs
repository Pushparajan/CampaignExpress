//! Hardware-agnostic inference provider abstraction.
//!
//! All backends (CPU, NPU, Groq, Inferentia, ARM, Tenstorrent) implement the
//! [`CoLaNetProvider`] trait, allowing the rest of the system to be decoupled
//! from the execution environment.

use crate::types::{InferenceResult, UserProfile};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Hardware-agnostic inference provider trait.
/// All backends (CPU, NPU, Groq, Inferentia, ARM, Tenstorrent) implement this.
pub trait CoLaNetProvider: Send + Sync {
    /// Score offers for a user (latency-optimized single request).
    fn predict(
        &self,
        profile: &UserProfile,
        offer_ids: &[String],
    ) -> Result<Vec<InferenceResult>, InferenceError>;

    /// Score a batch of requests (throughput-optimized for NPUs/accelerators).
    fn predict_batch(
        &self,
        requests: Vec<(UserProfile, Vec<String>)>,
    ) -> Result<Vec<Vec<InferenceResult>>, InferenceError>;

    /// Provider name for metrics/logging.
    fn provider_name(&self) -> &str;

    /// Whether this provider supports batched inference efficiently.
    fn supports_batching(&self) -> bool;

    /// Maximum recommended batch size for this hardware.
    fn max_batch_size(&self) -> usize;

    /// Warm up the provider (load model, allocate buffers).
    fn warm_up(&self) -> Result<(), InferenceError>;
}

/// Errors that can occur during inference.
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// The model has not been loaded yet.
    ModelNotLoaded(String),
    /// The requested batch size exceeds the hardware limit.
    BatchTooLarge { max: usize, got: usize },
    /// The target hardware is not available.
    HardwareUnavailable(String),
    /// Inference execution failed.
    InferenceFailure(String),
    /// Inference timed out.
    Timeout(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferenceError::ModelNotLoaded(msg) => write!(f, "model not loaded: {msg}"),
            InferenceError::BatchTooLarge { max, got } => {
                write!(f, "batch too large: max={max}, got={got}")
            }
            InferenceError::HardwareUnavailable(msg) => {
                write!(f, "hardware unavailable: {msg}")
            }
            InferenceError::InferenceFailure(msg) => write!(f, "inference failure: {msg}"),
            InferenceError::Timeout(msg) => write!(f, "inference timeout: {msg}"),
        }
    }
}

impl std::error::Error for InferenceError {}

/// Configuration for selecting and configuring an inference provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceProviderConfig {
    pub provider: ProviderType,
    pub model_path: String,
    pub num_threads: usize,
    pub max_batch_size: usize,
    pub timeout_ms: u64,
    pub device_id: Option<String>,
}

/// Supported inference provider backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    /// Pure CPU inference (synthetic weights, no hardware acceleration).
    Cpu,
    /// AMD XDNA NPU via Vitis AI / Ryzen AI.
    AmdXdna,
    /// AWS Inferentia 2 Neuron cores.
    AwsInferentia2,
    /// AWS Inferentia 3 Neuron cores.
    AwsInferentia3,
    /// Groq LPU Cloud inference.
    Groq,
    /// Oracle Cloud Ampere Altra ARM CPU.
    OracleAmpere,
    /// Tenstorrent Wormhole / Grayskull RISC-V mesh.
    Tenstorrent,
}
