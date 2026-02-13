//! Nagle's Algorithm-inspired batching buffer for NPU/accelerator throughput.
//!
//! The [`InferenceBatcher`] collects individual inference requests and flushes
//! them as a single batch through any [`CoLaNetProvider`] backend that supports
//! batched inference.  For non-batching providers (e.g. CPU), requests are
//! forwarded immediately without buffering.

use campaign_core::inference::CoLaNetProvider;
use campaign_core::types::{InferenceResult, UserProfile};
use std::sync::Arc;

/// Batching adapter that sits in front of any [`CoLaNetProvider`].
///
/// For providers that support batching, individual requests can be collected
/// and flushed together via [`flush_batch`](InferenceBatcher::flush_batch).
/// For non-batching providers, [`submit`](InferenceBatcher::submit) falls
/// through directly to `predict`.
pub struct InferenceBatcher {
    provider: Arc<dyn CoLaNetProvider>,
    max_batch_size: usize,
    max_wait_us: u64,
}

impl InferenceBatcher {
    /// Create a new batcher wrapping the given provider.
    ///
    /// * `provider` — the backend to delegate inference to.
    /// * `max_wait_us` — maximum microseconds to wait before flushing a
    ///   partial batch (Nagle-style coalescing window).
    pub fn new(provider: Arc<dyn CoLaNetProvider>, max_wait_us: u64) -> Self {
        let max_batch_size = provider.max_batch_size();
        Self {
            provider,
            max_batch_size,
            max_wait_us,
        }
    }

    /// Submit a single inference request.
    ///
    /// For non-batching providers this calls `predict` directly.
    /// For batching providers this also calls `predict` directly as the
    /// synchronous fallback; actual async batching requires a tokio runtime
    /// and is handled at the service layer.
    pub fn submit(&self, profile: UserProfile, offer_ids: Vec<String>) -> Vec<InferenceResult> {
        // For non-batching providers, just call predict directly.
        if !self.provider.supports_batching() {
            return self
                .provider
                .predict(&profile, &offer_ids)
                .unwrap_or_default();
        }
        // For batching providers, call predict directly too (actual async
        // batching requires tokio runtime; this is the synchronous fallback).
        self.provider
            .predict(&profile, &offer_ids)
            .unwrap_or_default()
    }

    /// Flush a collected batch of requests through the provider.
    ///
    /// Returns one `Vec<InferenceResult>` per request in the batch.
    pub fn flush_batch(
        &self,
        requests: Vec<(UserProfile, Vec<String>)>,
    ) -> Vec<Vec<InferenceResult>> {
        self.provider.predict_batch(requests).unwrap_or_default()
    }

    /// The name of the underlying provider (for metrics/logging).
    pub fn provider_name(&self) -> &str {
        self.provider.provider_name()
    }

    /// Maximum batch size supported by the underlying provider.
    pub fn max_batch_size(&self) -> usize {
        self.max_batch_size
    }

    /// Maximum wait time in microseconds before flushing a partial batch.
    pub fn max_wait_us(&self) -> u64 {
        self.max_wait_us
    }
}
