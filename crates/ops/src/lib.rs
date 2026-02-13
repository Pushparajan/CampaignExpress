//! Operations crate — backup, SLA tracking, status page, and incident management.

pub mod backup;
pub mod incident;
pub mod sla;
pub mod status_page;

pub use backup::BackupManager;
pub use incident::IncidentManager;
pub use sla::SlaTracker;
pub use status_page::StatusPageManager;
