//! DSP integration REST API endpoints.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use campaign_core::dsp::*;
use campaign_dsp::DspRouter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared state for DSP endpoints.
#[derive(Clone)]
pub struct DspState {
    pub router: Arc<DspRouter>,
}

/// POST /v1/dsp/bid — Route a bid request to DSPs.
pub async fn handle_dsp_bid(
    State(state): State<DspState>,
    Json(request): Json<DspBidApiRequest>,
) -> Json<DspBidApiResponse> {
    let responses = state.router.route_bid(
        &request.request_id,
        &request.openrtb_json,
        &request.impression_ids,
    );

    let total_bids: usize = responses.iter().filter(|r| !r.no_bid).count();

    Json(DspBidApiResponse {
        request_id: request.request_id,
        dsp_responses: responses.len(),
        bids_received: total_bids,
        responses,
    })
}

/// POST /v1/dsp/win — Record a win notification.
pub async fn handle_dsp_win(
    State(state): State<DspState>,
    Json(request): Json<DspWinRequest>,
) -> StatusCode {
    state.router.record_win(request.platform, request.win_price);
    metrics::counter!(
        "dsp.wins",
        "platform" => request.platform.seat_id()
    )
    .increment(1);
    StatusCode::OK
}

/// GET /v1/dsp/status — Get DSP routing status.
pub async fn handle_dsp_status(
    State(state): State<DspState>,
) -> Json<DspStatusResponse> {
    Json(DspStatusResponse {
        active_dsps: state.router.active_dsp_count(),
    })
}

#[derive(Deserialize)]
pub struct DspBidApiRequest {
    pub request_id: String,
    pub openrtb_json: String,
    pub impression_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct DspBidApiResponse {
    pub request_id: String,
    pub dsp_responses: usize,
    pub bids_received: usize,
    pub responses: Vec<DspBidResponse>,
}

#[derive(Deserialize)]
pub struct DspWinRequest {
    pub platform: DspPlatform,
    pub win_price: f64,
}

#[derive(Serialize)]
pub struct DspStatusResponse {
    pub active_dsps: usize,
}
