//! Loyalty program REST API endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use campaign_core::loyalty::*;
use campaign_loyalty::LoyaltyEngine;
use serde::Serialize;
use std::sync::Arc;

/// Shared state for loyalty endpoints.
#[derive(Clone)]
pub struct LoyaltyState {
    pub engine: Arc<LoyaltyEngine>,
}

/// POST /v1/loyalty/earn — Earn stars from a purchase.
pub async fn handle_earn_stars(
    State(state): State<LoyaltyState>,
    Json(request): Json<EarnStarsRequest>,
) -> Json<EarnStarsResponse> {
    let mut profile = state.engine.get_or_create_profile(&request.user_id);
    let response = state.engine.earn_stars(&mut profile, &request);
    metrics::counter!("loyalty.api.earn_stars").increment(1);
    Json(response)
}

/// POST /v1/loyalty/redeem — Redeem stars for a reward.
pub async fn handle_redeem(
    State(state): State<LoyaltyState>,
    Json(request): Json<RedeemRequest>,
) -> Json<RedeemResponse> {
    let mut profile = state.engine.get_or_create_profile(&request.user_id);
    let response = state.engine.redeem_stars(&mut profile, &request);
    if response.success {
        metrics::counter!("loyalty.api.redemptions").increment(1);
    }
    Json(response)
}

/// GET /v1/loyalty/balance/:user_id — Get loyalty balance and tier.
pub async fn handle_balance(
    State(state): State<LoyaltyState>,
    Path(user_id): Path<String>,
) -> Json<LoyaltyBalanceResponse> {
    let profile = state.engine.get_or_create_profile(&user_id);
    Json(LoyaltyBalanceResponse {
        user_id: profile.user_id.clone(),
        tier: profile.tier,
        stars_balance: profile.stars_balance,
        stars_qualifying: profile.stars_qualifying,
        tier_progress: profile.tier_progress(),
        effective_earn_rate: profile.effective_earn_rate(),
        lifetime_stars: profile.lifetime_stars,
        total_redemptions: profile.total_redemptions,
    })
}

/// POST /v1/loyalty/reward-signal — Record an RL reward signal for SNN training.
pub async fn handle_reward_signal(
    Json(signal): Json<LoyaltyRewardSignal>,
) -> StatusCode {
    metrics::counter!(
        "loyalty.reward_signals",
        "type" => format!("{:?}", signal.signal_type)
    )
    .increment(1);
    // In production: forward to SNN training pipeline via NATS
    StatusCode::ACCEPTED
}

#[derive(Serialize)]
pub struct LoyaltyBalanceResponse {
    pub user_id: String,
    pub tier: LoyaltyTier,
    pub stars_balance: u32,
    pub stars_qualifying: u32,
    pub tier_progress: f32,
    pub effective_earn_rate: f32,
    pub lifetime_stars: u64,
    pub total_redemptions: u32,
}
