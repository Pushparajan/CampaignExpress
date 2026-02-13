//! Web SDK server-side support — click-stream capture, web behavior event
//! ingestion, session tracking, and adaptors for Google Tag Manager and
//! Google Analytics 4.
//!
//! # Modules
//!
//! - [`events`] — Web event types (page views, clicks, scrolls, forms, etc.)
//! - [`collector`] — Event collector/ingester wired into the unified event bus
//! - [`clickstream`] — Click-stream processor for heatmaps and funnel analysis
//! - [`adaptors`] — Third-party platform adaptors (GTM, GA4)

pub mod adaptors;
pub mod clickstream;
pub mod collector;
pub mod events;

pub use adaptors::ga::GaAdaptor;
pub use adaptors::gtm::GtmAdaptor;
pub use adaptors::WebAdaptor;
pub use clickstream::ClickStreamProcessor;
pub use collector::WebEventCollector;
