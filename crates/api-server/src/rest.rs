//! REST API handlers for OpenRTB bid requests and operational endpoints.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use campaign_agents::BidProcessor;
use campaign_core::openrtb::{BidRequest, BidResponse};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tracing::error;

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub processor: Arc<BidProcessor>,
    pub node_id: String,
    pub start_time: Instant,
}

/// POST /v1/bid — OpenRTB bid request endpoint.
pub async fn handle_bid(
    State(state): State<AppState>,
    Json(request): Json<BidRequest>,
) -> Result<Json<BidResponse>, (StatusCode, Json<ErrorResponse>)> {
    let agent_id = format!("{}-rest", state.node_id);

    match state.processor.process(&request, &agent_id).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!(error = %e, request_id = %request.id, "Bid processing failed");
            metrics::counter!("api.errors").increment(1);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "bid_processing_failed".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

/// GET /health — Health check endpoint.
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        node_id: state.node_id.clone(),
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}

/// GET /ready — Readiness probe for Kubernetes.
pub async fn readiness() -> StatusCode {
    StatusCode::OK
}

/// GET /live — Liveness probe for Kubernetes.
pub async fn liveness() -> StatusCode {
    StatusCode::OK
}

/// GET /metrics — Prometheus metrics endpoint (handled by metrics-exporter-prometheus).
/// This is a placeholder; the actual metrics endpoint is mounted separately.

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub node_id: String,
    pub uptime_secs: u64,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}
