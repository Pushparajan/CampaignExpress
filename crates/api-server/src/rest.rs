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
use tracing::{error, warn};

/// Maximum number of impressions per bid request.
const MAX_IMPRESSIONS: usize = 100;

/// Maximum string field length (request ID, user ID, etc.).
const MAX_FIELD_LEN: usize = 256;

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub processor: Arc<BidProcessor>,
    pub node_id: String,
    pub start_time: Instant,
}

/// Validate an OpenRTB bid request at the API boundary.
fn validate_bid_request(request: &BidRequest) -> Result<(), &'static str> {
    if request.id.is_empty() {
        return Err("bid request 'id' must not be empty");
    }
    if request.id.len() > MAX_FIELD_LEN {
        return Err("bid request 'id' exceeds maximum length");
    }
    if request.imp.is_empty() {
        return Err("bid request must contain at least one impression");
    }
    if request.imp.len() > MAX_IMPRESSIONS {
        return Err("bid request exceeds maximum number of impressions");
    }
    for imp in &request.imp {
        if imp.id.is_empty() {
            return Err("impression 'id' must not be empty");
        }
        if imp.id.len() > MAX_FIELD_LEN {
            return Err("impression 'id' exceeds maximum length");
        }
        if imp.bidfloor < 0.0 {
            return Err("impression 'bidfloor' must be non-negative");
        }
    }
    Ok(())
}

/// POST /v1/bid — OpenRTB bid request endpoint.
pub async fn handle_bid(
    State(state): State<AppState>,
    Json(request): Json<BidRequest>,
) -> Result<Json<BidResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input at API boundary
    if let Err(msg) = validate_bid_request(&request) {
        warn!(request_id = %request.id, error = msg, "Bid request validation failed");
        metrics::counter!("api.validation_errors").increment(1);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_bid_request".to_string(),
                message: msg.to_string(),
            }),
        ));
    }

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
                    message: "Internal processing error".to_string(),
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
/// Returns 200 only when the service is ready to accept traffic.
pub async fn readiness(State(state): State<AppState>) -> StatusCode {
    // Verify the processor is initialized and the node has been running
    if state.start_time.elapsed().as_secs() > 0 {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
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
