//! Management API router — mounts all management endpoints under /api/v1/management.

use crate::handlers::{self, ManagementState};
use crate::store::ManagementStore;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

/// Build the management router with all endpoints.
/// Returns a Router that should be merged into the main app.
pub fn management_router() -> Router {
    let store = Arc::new(ManagementStore::new());
    let state = ManagementState { store };

    Router::new()
        // Auth
        .route("/api/v1/management/auth/login", post(handlers::handle_login))
        // Campaigns
        .route("/api/v1/management/campaigns", get(handlers::list_campaigns).post(handlers::create_campaign))
        .route("/api/v1/management/campaigns/{id}", get(handlers::get_campaign).put(handlers::update_campaign).delete(handlers::delete_campaign))
        .route("/api/v1/management/campaigns/{id}/pause", post(handlers::pause_campaign))
        .route("/api/v1/management/campaigns/{id}/resume", post(handlers::resume_campaign))
        // Creatives
        .route("/api/v1/management/creatives", get(handlers::list_creatives).post(handlers::create_creative))
        .route("/api/v1/management/creatives/{id}", get(handlers::get_creative).put(handlers::update_creative).delete(handlers::delete_creative))
        // Monitoring
        .route("/api/v1/management/monitoring/overview", get(handlers::monitoring_overview))
        .route("/api/v1/management/monitoring/campaigns/{id}/stats", get(handlers::campaign_stats))
        // Models
        .route("/api/v1/management/models/reload", post(handlers::model_reload))
        // Audit log
        .route("/api/v1/management/audit-log", get(handlers::audit_log))
        // Journeys
        .route("/api/v1/management/journeys", get(handlers::list_journeys).post(handlers::create_journey))
        .route("/api/v1/management/journeys/{id}", get(handlers::get_journey).delete(handlers::delete_journey))
        .route("/api/v1/management/journeys/{id}/stats", get(handlers::journey_stats))
        // DCO Templates
        .route("/api/v1/management/dco/templates", get(handlers::list_dco_templates).post(handlers::create_dco_template))
        .route("/api/v1/management/dco/templates/{id}", get(handlers::get_dco_template).delete(handlers::delete_dco_template))
        // CDP
        .route("/api/v1/management/cdp/platforms", get(handlers::list_cdp_platforms))
        .route("/api/v1/management/cdp/sync-history", get(handlers::cdp_sync_history))
        // Experiments
        .route("/api/v1/management/experiments", get(handlers::list_experiments).post(handlers::create_experiment))
        .route("/api/v1/management/experiments/{id}", get(handlers::get_experiment))
        .with_state(state)
}
