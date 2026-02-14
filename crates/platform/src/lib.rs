//! SaaS platform capabilities: authentication, authorization, multi-tenancy,
//! rate limiting, audit logging, and privacy/compliance (GDPR).

pub mod audit;
pub mod auth;
pub mod governance;
pub mod privacy;
pub mod rate_limit;
pub mod rbac;
pub mod tenancy;

pub use audit::AuditLogger;
pub use auth::AuthManager;
pub use governance::{LineageTracker, PiiClassifier, SchemaRegistry};
pub use privacy::PrivacyManager;
pub use rate_limit::RateLimiter;
pub use rbac::RbacEngine;
pub use tenancy::TenantManager;
