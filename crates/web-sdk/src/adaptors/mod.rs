//! Adaptors for translating web events into third-party analytics formats.
//!
//! Each adaptor implements [`WebAdaptor`] to transform [`WebEvent`]s into the
//! JSON payload expected by its target platform (Google Tag Manager, Google
//! Analytics 4, etc.).

pub mod ga;
pub mod gtm;

use anyhow::Result;

use crate::events::WebEvent;

/// Adaptor trait — transforms web events into a platform-specific JSON payload.
pub trait WebAdaptor: Send + Sync {
    /// Platform identifier (e.g. "gtm", "ga4").
    fn platform(&self) -> &str;

    /// Transform a web event into the target platform's payload format.
    fn transform(&self, event: &WebEvent) -> Result<serde_json::Value>;

    /// Transform a batch of events. Default implementation transforms one-by-one.
    fn transform_batch(&self, events: &[WebEvent]) -> Result<Vec<serde_json::Value>> {
        events.iter().map(|e| self.transform(e)).collect()
    }

    /// Validate that the adaptor configuration is correct.
    fn validate_config(&self) -> Result<()>;
}
