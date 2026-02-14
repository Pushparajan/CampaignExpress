//! Management API router — mounts all management endpoints under /api/v1/management.

use crate::handlers::{self, ManagementState};
use crate::store::ManagementStore;
use axum::routing::{delete, get, post, put};
use axum::Router;
use std::sync::Arc;

/// Build the management router with all endpoints.
/// Returns a Router that should be merged into the main app.
pub fn management_router() -> Router {
    let store = Arc::new(ManagementStore::new());
    let state = ManagementState { store };

    Router::new()
        // Auth
        .route(
            "/api/v1/management/auth/login",
            post(handlers::handle_login),
        )
        // Campaigns
        .route(
            "/api/v1/management/campaigns",
            get(handlers::list_campaigns).post(handlers::create_campaign),
        )
        .route(
            "/api/v1/management/campaigns/{id}",
            get(handlers::get_campaign)
                .put(handlers::update_campaign)
                .delete(handlers::delete_campaign),
        )
        .route(
            "/api/v1/management/campaigns/{id}/pause",
            post(handlers::pause_campaign),
        )
        .route(
            "/api/v1/management/campaigns/{id}/resume",
            post(handlers::resume_campaign),
        )
        // Creatives
        .route(
            "/api/v1/management/creatives",
            get(handlers::list_creatives).post(handlers::create_creative),
        )
        .route(
            "/api/v1/management/creatives/{id}",
            get(handlers::get_creative)
                .put(handlers::update_creative)
                .delete(handlers::delete_creative),
        )
        // Monitoring
        .route(
            "/api/v1/management/monitoring/overview",
            get(handlers::monitoring_overview),
        )
        .route(
            "/api/v1/management/monitoring/campaigns/{id}/stats",
            get(handlers::campaign_stats),
        )
        // Models
        .route(
            "/api/v1/management/models/reload",
            post(handlers::model_reload),
        )
        // Audit log
        .route("/api/v1/management/audit-log", get(handlers::audit_log))
        // Journeys
        .route(
            "/api/v1/management/journeys",
            get(handlers::list_journeys).post(handlers::create_journey),
        )
        .route(
            "/api/v1/management/journeys/{id}",
            get(handlers::get_journey).delete(handlers::delete_journey),
        )
        .route(
            "/api/v1/management/journeys/{id}/stats",
            get(handlers::journey_stats),
        )
        // DCO Templates
        .route(
            "/api/v1/management/dco/templates",
            get(handlers::list_dco_templates).post(handlers::create_dco_template),
        )
        .route(
            "/api/v1/management/dco/templates/{id}",
            get(handlers::get_dco_template).delete(handlers::delete_dco_template),
        )
        // CDP
        .route(
            "/api/v1/management/cdp/platforms",
            get(handlers::list_cdp_platforms),
        )
        .route(
            "/api/v1/management/cdp/sync-history",
            get(handlers::cdp_sync_history),
        )
        // Experiments
        .route(
            "/api/v1/management/experiments",
            get(handlers::list_experiments).post(handlers::create_experiment),
        )
        .route(
            "/api/v1/management/experiments/{id}",
            get(handlers::get_experiment),
        )
        // Platform — Tenants
        .route(
            "/api/v1/management/platform/tenants",
            get(handlers::list_tenants).post(handlers::create_tenant),
        )
        .route(
            "/api/v1/management/platform/tenants/{id}",
            get(handlers::get_tenant)
                .put(handlers::update_tenant)
                .delete(handlers::delete_tenant),
        )
        .route(
            "/api/v1/management/platform/tenants/{id}/suspend",
            post(handlers::suspend_tenant),
        )
        .route(
            "/api/v1/management/platform/tenants/{id}/activate",
            post(handlers::activate_tenant),
        )
        .route(
            "/api/v1/management/platform/roles",
            get(handlers::list_roles),
        )
        .route(
            "/api/v1/management/platform/compliance",
            get(handlers::compliance_status),
        )
        .route(
            "/api/v1/management/platform/privacy/dsrs",
            get(handlers::list_dsrs),
        )
        // Billing
        .route(
            "/api/v1/management/billing/plans",
            get(handlers::list_plans),
        )
        .route(
            "/api/v1/management/billing/subscriptions/{tenant_id}",
            get(handlers::get_subscription),
        )
        .route(
            "/api/v1/management/billing/invoices",
            get(handlers::list_invoices),
        )
        .route(
            "/api/v1/management/billing/usage/{tenant_id}",
            get(handlers::get_usage),
        )
        .route(
            "/api/v1/management/billing/onboarding/{tenant_id}",
            get(handlers::get_onboarding),
        )
        // Ops
        // Users
        .route(
            "/api/v1/management/users",
            get(handlers::list_users).post(handlers::create_user),
        )
        .route(
            "/api/v1/management/users/{id}",
            get(handlers::get_user).delete(handlers::delete_user),
        )
        .route(
            "/api/v1/management/users/{id}/disable",
            post(handlers::disable_user),
        )
        .route(
            "/api/v1/management/users/{id}/enable",
            post(handlers::enable_user),
        )
        .route(
            "/api/v1/management/users/{id}/role",
            put(handlers::update_user_role),
        )
        // Invitations
        .route(
            "/api/v1/management/invitations",
            get(handlers::list_invitations).post(handlers::create_invitation),
        )
        .route(
            "/api/v1/management/invitations/{id}",
            delete(handlers::revoke_invitation),
        )
        // Ops
        .route("/api/v1/management/ops/status", get(handlers::ops_status))
        .route(
            "/api/v1/management/ops/incidents",
            get(handlers::list_incidents),
        )
        .route("/api/v1/management/ops/sla", get(handlers::sla_report))
        .route(
            "/api/v1/management/ops/backups",
            get(handlers::list_backups),
        )
        .with_state(state)
}
