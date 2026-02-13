//! Integration marketplace — connectors for Segment, Salesforce, Snowflake,
//! and 20+ third-party platforms.

pub mod bi_tools;
pub mod connector;
pub mod dam;
pub mod marketplace;
pub mod task_management;
pub mod webhook;

pub use bi_tools::BiToolsAdaptor;
pub use connector::IntegrationConnector;
pub use dam::DamAdaptor;
pub use marketplace::IntegrationMarketplace;
pub use task_management::TaskManagementAdaptor;
pub use webhook::WebhookManager;
