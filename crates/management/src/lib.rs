//! Campaign management UI backend — campaigns, creatives, targeting, monitoring.
//!
//! Provides REST API endpoints for the management dashboard UI.
//! Data stored in DashMap (development); swap to PostgreSQL for production.

pub mod auth;
pub mod governance;
pub mod handlers;
pub mod models;
pub mod preflight;
pub mod router;
pub mod store;
pub mod workflows;
pub mod workspace;

pub use governance::UnifiedGovernanceGate;
pub use handlers::ManagementState;
pub use router::management_router;
pub use store::ManagementStore;
pub use workflows::{CampaignCalendar, WorkflowEngine};
pub use workspace::{
    BulkOperationEngine, ExplainabilityEngine, OperatorCalendar, UnifiedCreateFlow,
};
