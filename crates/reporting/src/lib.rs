//! Campaign analytics and reporting — dashboards, funnels, cohort analysis,
//! revenue attribution, and real-time metrics.

pub mod attribution;
pub mod cohort;
pub mod dashboard;
pub mod funnel;

pub use attribution::RevenueAttributionEngine;
pub use cohort::CohortAnalyzer;
pub use dashboard::CampaignDashboard;
pub use funnel::FunnelAnalyzer;
