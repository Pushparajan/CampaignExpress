//! Campaign Express DevOps & SRE Toolkit
//!
//! Proactive issue detection, diagnosis, and automated remediation for
//! the Campaign Express platform. Designed for DevOps and SRE teams
//! to monitor, troubleshoot, and maintain production health.
//!
//! # Modules
//!
//! - [`health_checker`] — Deep service health probes and dependency checks
//! - [`resource_monitor`] — Memory, CPU, queue, and connection pool tracking
//! - [`log_analyzer`] — Pattern detection, anomaly scoring, and log correlation
//! - [`capacity_planner`] — Trend forecasting and resource exhaustion prediction
//! - [`auto_remediation`] — Automated runbooks and self-healing actions
//! - [`incident_detector`] — Proactive anomaly detection and SLO burn-rate alerts
//! - [`diagnostics`] — Full-stack triage CLI for rapid issue diagnosis

pub mod auto_remediation;
pub mod capacity_planner;
pub mod diagnostics;
pub mod health_checker;
pub mod incident_detector;
pub mod log_analyzer;
pub mod resource_monitor;
