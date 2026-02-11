//! Axum REST handlers for the management API.

use crate::auth;
use crate::models::*;
use crate::store::ManagementStore;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

/// Shared management state.
#[derive(Clone)]
pub struct ManagementState {
    pub store: Arc<ManagementStore>,
}

// ─── Auth ──────────────────────────────────────────────────────────────────

pub async fn handle_login(
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    match auth::authenticate(&req) {
        Ok(resp) => Ok(Json(resp)),
        Err(msg) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "auth_failed".to_string(),
                message: msg,
            }),
        )),
    }
}

// ─── Campaigns ─────────────────────────────────────────────────────────────

pub async fn list_campaigns(State(state): State<ManagementState>) -> Json<Vec<Campaign>> {
    Json(state.store.list_campaigns())
}

pub async fn get_campaign(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Campaign>, StatusCode> {
    state
        .store
        .get_campaign(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_campaign(
    State(state): State<ManagementState>,
    Json(req): Json<CreateCampaignRequest>,
) -> (StatusCode, Json<Campaign>) {
    let campaign = state.store.create_campaign(req, "admin");
    metrics::counter!("management.campaigns.created").increment(1);
    (StatusCode::CREATED, Json(campaign))
}

pub async fn update_campaign(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> Result<Json<Campaign>, StatusCode> {
    state
        .store
        .update_campaign(id, req, "admin")
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_campaign(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    if state.store.delete_campaign(id, "admin") {
        metrics::counter!("management.campaigns.deleted").increment(1);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn pause_campaign(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Campaign>, StatusCode> {
    state
        .store
        .pause_campaign(id, "admin")
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn resume_campaign(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Campaign>, StatusCode> {
    state
        .store
        .resume_campaign(id, "admin")
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

// ─── Creatives ─────────────────────────────────────────────────────────────

pub async fn list_creatives(State(state): State<ManagementState>) -> Json<Vec<Creative>> {
    Json(state.store.list_creatives())
}

pub async fn get_creative(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Creative>, StatusCode> {
    state
        .store
        .get_creative(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_creative(
    State(state): State<ManagementState>,
    Json(req): Json<CreateCreativeRequest>,
) -> (StatusCode, Json<Creative>) {
    let creative = state.store.create_creative(req, "admin");
    metrics::counter!("management.creatives.created").increment(1);
    (StatusCode::CREATED, Json(creative))
}

pub async fn update_creative(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCreativeRequest>,
) -> Result<Json<Creative>, StatusCode> {
    state
        .store
        .update_creative(id, req, "admin")
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_creative(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    if state.store.delete_creative(id, "admin") {
        metrics::counter!("management.creatives.deleted").increment(1);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ─── Monitoring ────────────────────────────────────────────────────────────

pub async fn monitoring_overview(State(state): State<ManagementState>) -> Json<MonitoringOverview> {
    Json(state.store.get_monitoring_overview())
}

pub async fn campaign_stats(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CampaignStats>, StatusCode> {
    state
        .store
        .get_campaign_stats(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

// ─── Models ────────────────────────────────────────────────────────────────

pub async fn model_reload(State(state): State<ManagementState>) -> Json<serde_json::Value> {
    state.store.get_audit_log(); // Touch store to prove it's alive
    metrics::counter!("management.model_reloads").increment(1);
    // In production: trigger NPU hot-reload via NATS ModelUpdate message
    Json(serde_json::json!({
        "status": "accepted",
        "message": "Model reload initiated. Check NPU engine logs for progress."
    }))
}

// ─── Audit Log ─────────────────────────────────────────────────────────────

pub async fn audit_log(State(state): State<ManagementState>) -> Json<Vec<AuditLogEntry>> {
    Json(state.store.get_audit_log())
}

// ─── Journeys ─────────────────────────────────────────────────────────

pub async fn list_journeys(State(state): State<ManagementState>) -> Json<Vec<serde_json::Value>> {
    Json(state.store.list_journeys())
}

pub async fn get_journey(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .get_journey(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_journey(
    State(state): State<ManagementState>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let journey = state.store.create_journey(req, "admin");
    metrics::counter!("management.journeys.created").increment(1);
    (StatusCode::CREATED, Json(journey))
}

pub async fn delete_journey(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    if state.store.delete_journey(id, "admin") {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn journey_stats(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    Json(state.store.get_journey_stats(id))
}

// ─── DCO Templates ────────────────────────────────────────────────────

pub async fn list_dco_templates(
    State(state): State<ManagementState>,
) -> Json<Vec<serde_json::Value>> {
    Json(state.store.list_dco_templates())
}

pub async fn get_dco_template(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .get_dco_template(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_dco_template(
    State(state): State<ManagementState>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let template = state.store.create_dco_template(req, "admin");
    metrics::counter!("management.dco_templates.created").increment(1);
    (StatusCode::CREATED, Json(template))
}

pub async fn delete_dco_template(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    if state.store.delete_dco_template(id, "admin") {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ─── CDP Platforms ────────────────────────────────────────────────────

pub async fn list_cdp_platforms(
    State(state): State<ManagementState>,
) -> Json<Vec<serde_json::Value>> {
    Json(state.store.list_cdp_platforms())
}

pub async fn cdp_sync_history(
    State(state): State<ManagementState>,
) -> Json<Vec<serde_json::Value>> {
    Json(state.store.get_cdp_sync_history())
}

// ─── Experiments ──────────────────────────────────────────────────────

pub async fn list_experiments(
    State(state): State<ManagementState>,
) -> Json<Vec<serde_json::Value>> {
    Json(state.store.list_experiments())
}

pub async fn get_experiment(
    State(state): State<ManagementState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .get_experiment(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_experiment(
    State(state): State<ManagementState>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let experiment = state.store.create_experiment(req, "admin");
    metrics::counter!("management.experiments.created").increment(1);
    (StatusCode::CREATED, Json(experiment))
}
