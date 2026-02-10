//! Omnichannel ingest and activation REST API endpoints.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use campaign_channels::{ActivationDispatcher, IngestProcessor, SendGridProvider};
use campaign_core::channels::*;
use serde::Serialize;
use std::sync::Arc;
use tracing::error;

/// Shared state for channel endpoints.
#[derive(Clone)]
pub struct ChannelState {
    pub ingest: Arc<IngestProcessor>,
    pub activation: Arc<ActivationDispatcher>,
    pub sendgrid: Arc<SendGridProvider>,
}

/// POST /v1/channels/ingest — Process a real-time ingest event.
pub async fn handle_ingest(
    State(state): State<ChannelState>,
    Json(event): Json<IngestEvent>,
) -> Result<Json<IngestResponse>, (StatusCode, Json<ChannelErrorResponse>)> {
    match state.ingest.process_event(&event) {
        Ok(processed) => {
            metrics::counter!(
                "channels.ingest.processed",
                "source" => event.source.display_name()
            )
            .increment(1);
            Ok(Json(IngestResponse {
                event_id: processed.event_id,
                user_id: processed.user_id,
                should_activate: processed.should_activate,
                loyalty_relevant: processed.loyalty_relevant,
            }))
        }
        Err(e) => {
            error!(error = %e, "Ingest processing failed");
            Err((
                StatusCode::BAD_REQUEST,
                Json(ChannelErrorResponse {
                    error: "ingest_failed".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

/// POST /v1/channels/activate — Dispatch an activation to a channel.
pub async fn handle_activate(
    State(state): State<ChannelState>,
    Json(request): Json<ActivationRequest>,
) -> Json<ActivationResult> {
    let result = state.activation.dispatch(&request).await;
    Json(result)
}

/// POST /v1/webhooks/sendgrid — SendGrid delivery webhook receiver.
pub async fn handle_sendgrid_webhook(
    State(state): State<ChannelState>,
    Json(events): Json<Vec<EmailWebhookEvent>>,
) -> StatusCode {
    for event in &events {
        state.sendgrid.process_webhook(event);
    }
    metrics::counter!("sendgrid.webhooks_received").increment(events.len() as u64);
    StatusCode::OK
}

/// GET /v1/channels/email/analytics/:activation_id — Get email analytics.
pub async fn handle_email_analytics(
    State(state): State<ChannelState>,
    axum::extract::Path(activation_id): axum::extract::Path<String>,
) -> Result<Json<EmailAnalytics>, StatusCode> {
    match state.sendgrid.get_analytics(&activation_id) {
        Some(analytics) => Ok(Json(analytics)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// GET /v1/channels/email/analytics — Get all email analytics.
pub async fn handle_all_email_analytics(
    State(state): State<ChannelState>,
) -> Json<Vec<EmailAnalytics>> {
    Json(state.sendgrid.all_analytics())
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub event_id: String,
    pub user_id: String,
    pub should_activate: bool,
    pub loyalty_relevant: bool,
}

#[derive(Serialize)]
pub struct ChannelErrorResponse {
    pub error: String,
    pub message: String,
}
