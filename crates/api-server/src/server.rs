//! API server — starts both HTTP (REST) and gRPC servers.

use crate::channel_rest::{self, ChannelState};
use crate::dsp_rest::{self, DspState};
use crate::loyalty_rest::{self, LoyaltyState};
use crate::rest::{self, AppState};
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use campaign_agents::BidProcessor;
use campaign_channels::{ActivationDispatcher, IngestProcessor, SendGridProvider};
use campaign_core::channels::{ActivationChannel, SendGridConfig};
use campaign_core::config::AppConfig;
use campaign_dsp::DspRouter;
use campaign_loyalty::LoyaltyEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Main API server managing both REST and gRPC endpoints.
pub struct ApiServer {
    config: AppConfig,
    processor: Arc<BidProcessor>,
}

impl ApiServer {
    pub fn new(config: AppConfig, processor: Arc<BidProcessor>) -> Self {
        Self { config, processor }
    }

    /// Start the HTTP REST server.
    pub async fn start_http(&self) -> anyhow::Result<()> {
        let state = AppState {
            processor: self.processor.clone(),
            node_id: self.config.node_id.clone(),
            start_time: Instant::now(),
        };

        // Initialize loyalty engine
        let loyalty_engine = Arc::new(LoyaltyEngine::new(&self.config.loyalty));
        let loyalty_state = LoyaltyState {
            engine: loyalty_engine,
        };

        // Initialize DSP router
        let dsp_router = Arc::new(DspRouter::new(&self.config.dsp, Vec::new()));
        let dsp_state = DspState { router: dsp_router };

        // Initialize channel processors
        let ingest = Arc::new(IngestProcessor::new(vec![
            campaign_core::channels::IngestSource::MobileApp,
            campaign_core::channels::IngestSource::Pos,
            campaign_core::channels::IngestSource::Kiosk,
            campaign_core::channels::IngestSource::Web,
            campaign_core::channels::IngestSource::CallCenter,
            campaign_core::channels::IngestSource::PartnerApi,
            campaign_core::channels::IngestSource::IoTDevice,
        ]));
        let activation = Arc::new(ActivationDispatcher::new(vec![
            ActivationChannel::PushNotification,
            ActivationChannel::Sms,
            ActivationChannel::Email,
            ActivationChannel::InAppMessage,
            ActivationChannel::WebPersonalization,
            ActivationChannel::PaidMediaFacebook,
            ActivationChannel::PaidMediaTradeDesk,
            ActivationChannel::PaidMediaGoogle,
            ActivationChannel::PaidMediaAmazon,
            ActivationChannel::DigitalSignage,
            ActivationChannel::KioskDisplay,
        ]));
        let sendgrid = Arc::new(SendGridProvider::new(SendGridConfig::default()));
        let channel_state = ChannelState {
            ingest,
            activation,
            sendgrid,
        };

        // Bid routes
        let bid_routes = Router::new()
            .route("/v1/bid", post(rest::handle_bid))
            .with_state(state.clone());

        // Operational routes
        let ops_routes = Router::new()
            .route("/health", get(rest::health_check))
            .route("/ready", get(rest::readiness))
            .route("/live", get(rest::liveness))
            .with_state(state);

        // Loyalty routes
        let loyalty_routes = Router::new()
            .route("/v1/loyalty/earn", post(loyalty_rest::handle_earn_stars))
            .route("/v1/loyalty/redeem", post(loyalty_rest::handle_redeem))
            .route(
                "/v1/loyalty/balance/{user_id}",
                get(loyalty_rest::handle_balance),
            )
            .route(
                "/v1/loyalty/reward-signal",
                post(loyalty_rest::handle_reward_signal),
            )
            .with_state(loyalty_state);

        // DSP routes
        let dsp_routes = Router::new()
            .route("/v1/dsp/bid", post(dsp_rest::handle_dsp_bid))
            .route("/v1/dsp/win", post(dsp_rest::handle_dsp_win))
            .route("/v1/dsp/status", get(dsp_rest::handle_dsp_status))
            .with_state(dsp_state);

        // Channel routes
        let channel_routes = Router::new()
            .route("/v1/channels/ingest", post(channel_rest::handle_ingest))
            .route("/v1/channels/activate", post(channel_rest::handle_activate))
            .route(
                "/v1/webhooks/sendgrid",
                post(channel_rest::handle_sendgrid_webhook),
            )
            .route(
                "/v1/channels/email/analytics/{activation_id}",
                get(channel_rest::handle_email_analytics),
            )
            .route(
                "/v1/channels/email/analytics",
                get(channel_rest::handle_all_email_analytics),
            )
            .with_state(channel_state);

        // Management UI routes (with auth middleware)
        let mgmt_routes = campaign_management::management_router().layer(middleware::from_fn(
            campaign_management::auth::auth_middleware,
        ));

        let app = Router::new()
            .merge(bid_routes)
            .merge(ops_routes)
            .merge(loyalty_routes)
            .merge(dsp_routes)
            .merge(channel_routes)
            .merge(mgmt_routes)
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::new(self.config.api.host.parse()?, self.config.api.http_port);

        info!(addr = %addr, "Starting HTTP server");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Start the metrics server on a separate port.
    pub async fn start_metrics(&self) -> anyhow::Result<()> {
        let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
        let handle = builder
            .with_http_listener(SocketAddr::new(
                self.config.api.host.parse()?,
                self.config.metrics.port,
            ))
            .install_recorder()?;

        info!(port = self.config.metrics.port, "Metrics exporter started");

        // Keep the handle alive
        std::mem::forget(handle);
        Ok(())
    }
}
