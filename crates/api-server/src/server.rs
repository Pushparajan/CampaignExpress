//! API server — starts both HTTP (REST) and gRPC servers.

use crate::rest::{self, AppState};
use axum::routing::{get, post};
use axum::Router;
use campaign_agents::BidProcessor;
use campaign_core::config::AppConfig;
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

        let app = Router::new()
            // OpenRTB bid endpoint
            .route("/v1/bid", post(rest::handle_bid))
            // Operational endpoints
            .route("/health", get(rest::health_check))
            .route("/ready", get(rest::readiness))
            .route("/live", get(rest::liveness))
            // Middleware
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let addr = SocketAddr::new(
            self.config.api.host.parse()?,
            self.config.api.http_port,
        );

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
