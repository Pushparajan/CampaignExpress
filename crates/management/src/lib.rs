//! Campaign management UI backend — campaigns, creatives, targeting, monitoring.
//!
//! Provides REST API endpoints for the management dashboard UI.
//! Data stored in DashMap (development); swap to PostgreSQL for production.

pub mod auth;
pub mod handlers;
pub mod models;
pub mod router;
pub mod store;
pub mod workflows;

pub use handlers::ManagementState;
pub use router::management_router;
pub use store::ManagementStore;
pub use workflows::{CampaignCalendar, WorkflowEngine};
