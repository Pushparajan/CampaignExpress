//! Integration marketplace — connectors for Segment, Salesforce, Snowflake,
//! and 20+ third-party platforms.

pub mod connector;
pub mod marketplace;
pub mod webhook;

pub use connector::IntegrationConnector;
pub use marketplace::IntegrationMarketplace;
pub use webhook::WebhookManager;
