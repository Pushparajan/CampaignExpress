//! Agentic AI Testing Framework for Campaign Express
//!
//! Provides autonomous AI-driven test agents that explore, interact with,
//! and validate Campaign Express UI pages and API endpoints.
//!
//! # Modules
//! - `actions` — Interaction primitives (click, navigate, type, API call)
//! - `agent` — Test agent with scripted, exploratory, and fuzzing strategies
//! - `assertions` — Typed assertion evaluation engine
//! - `page_objects` — Page object models for all UI pages
//! - `reporter` — Test execution reports with pass/fail/flaky analysis
//! - `scenario` — Declarative test plans with built-in scenarios

pub mod actions;
pub mod agent;
pub mod assertions;
pub mod page_objects;
pub mod reporter;
pub mod scenario;
