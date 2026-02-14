//! Campaign analytics and reporting — dashboards, funnels, cohort analysis,
//! revenue attribution, and real-time metrics.

pub mod attribution;
pub mod budget;
pub mod cohort;
pub mod dashboard;
pub mod funnel;
pub mod measurement;
pub mod report_builder;

pub use attribution::RevenueAttributionEngine;
pub use budget::BudgetTracker;
pub use cohort::CohortAnalyzer;
pub use dashboard::CampaignDashboard;
pub use funnel::FunnelAnalyzer;
pub use measurement::MeasurementEngine;
pub use report_builder::ReportBuilder;
