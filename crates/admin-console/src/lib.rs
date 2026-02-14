//! SaaS Admin Console — provider-level management for the Campaign Express
//! platform. Composes tenancy, billing, licensing, auth, RBAC, and ops into
//! a unified administration surface.
//!
//! # Modules
//!
//! - [`tenant_ops`] — Tenant lifecycle (suspend, reactivate, tier migration, quota alerts)
//! - [`user_ops`] — User management (CRUD, invitations, role assignment, sessions)
//! - [`provider_dashboard`] — Cross-tenant overview dashboard
//! - [`notifications`] — In-app alerts, email triggers, webhook management
//! - [`feature_flags`] — Per-tenant feature flag toggles
//! - [`system_settings`] — Global platform configuration and maintenance mode

pub mod feature_flags;
pub mod notifications;
pub mod provider_dashboard;
pub mod system_settings;
pub mod tenant_ops;
pub mod user_ops;

pub use feature_flags::FeatureFlagManager;
pub use notifications::NotificationManager;
pub use provider_dashboard::ProviderDashboard;
pub use system_settings::SystemSettings;
pub use tenant_ops::TenantOps;
pub use user_ops::UserOps;
