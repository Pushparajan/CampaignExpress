//! Billing, metering, and onboarding engine for Campaign Express.
//!
//! Provides usage metering, subscription billing, invoice generation,
//! and guided onboarding flows. Data stored in DashMap (development);
//! swap to PostgreSQL / Stripe for production.

pub mod billing;
pub mod metering;
pub mod onboarding;

pub use billing::BillingEngine;
pub use metering::MeteringEngine;
pub use onboarding::OnboardingEngine;
