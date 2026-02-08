//! Campaign Express — High-throughput real-time ad offer personalization platform.
//!
//! Main entry point that initializes all subsystems and starts the server.

use campaign_agents::AgentManager;
use campaign_analytics::AnalyticsLogger;
use campaign_api::ApiServer;
use campaign_cache::RedisCache;
use campaign_core::config::AppConfig;
use campaign_npu::NpuEngine;
use clap::Parser;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "campaign-express")]
#[command(about = "High-throughput real-time ad offer personalization platform")]
#[command(version)]
struct Cli {
    /// Node identifier (overrides config)
    #[arg(long, env = "CAMPAIGN_EXPRESS__NODE_ID")]
    node_id: Option<String>,

    /// Number of agents per node (overrides config)
    #[arg(long, env = "CAMPAIGN_EXPRESS__AGENTS_PER_NODE")]
    agents: Option<usize>,

    /// HTTP port (overrides config)
    #[arg(long, env = "CAMPAIGN_EXPRESS__API__HTTP_PORT")]
    http_port: Option<u16>,

    /// gRPC port (overrides config)
    #[arg(long, env = "CAMPAIGN_EXPRESS__API__GRPC_PORT")]
    grpc_port: Option<u16>,

    /// Skip NATS agent spawning (API-only mode)
    #[arg(long, default_value_t = false)]
    api_only: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "campaign_express=info,tower_http=info".into()),
        )
        .json()
        .init();

    let cli = Cli::parse();

    info!("Campaign Express starting up");

    // Load configuration
    let mut config = AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load config, using defaults");
        AppConfig::default()
    });

    // Apply CLI overrides
    if let Some(node_id) = cli.node_id {
        config.node_id = node_id;
    }
    if let Some(agents) = cli.agents {
        config.agents_per_node = agents;
    }
    if let Some(port) = cli.http_port {
        config.api.http_port = port;
    }
    if let Some(port) = cli.grpc_port {
        config.api.grpc_port = port;
    }

    info!(
        node_id = %config.node_id,
        agents = config.agents_per_node,
        http_port = config.api.http_port,
        grpc_port = config.api.grpc_port,
        "Configuration loaded"
    );

    // Initialize NPU engine
    let npu = Arc::new(NpuEngine::new(&config.npu)?);

    // Initialize Redis cache
    let cache = Arc::new(
        RedisCache::new(&config.redis)
            .await
            .unwrap_or_else(|e| {
                error!(error = %e, "Failed to connect to Redis, will retry on demand");
                // In production, this would block until connected.
                // For development, we proceed with a placeholder.
                panic!("Redis connection required: {}", e);
            }),
    );

    // Initialize analytics logger
    let analytics = Arc::new(
        AnalyticsLogger::new(&config.clickhouse, config.node_id.clone())
            .await
            .unwrap_or_else(|e| {
                error!(error = %e, "Failed to connect to ClickHouse, analytics disabled");
                panic!("ClickHouse connection required: {}", e);
            }),
    );

    // Initialize agent manager and processor
    let mut agent_manager = AgentManager::new(
        config.clone(),
        npu.clone(),
        cache.clone(),
        analytics.clone(),
    );

    let processor = agent_manager.processor();

    // Start NATS-based agents (unless API-only mode)
    if !cli.api_only {
        match agent_manager.start().await {
            Ok(_) => info!("Agent manager started with {} agents", config.agents_per_node),
            Err(e) => {
                error!(error = %e, "Failed to start agent manager, running in API-only mode");
            }
        }
    } else {
        info!("Running in API-only mode (no NATS agents)");
    }

    // Start API server
    let api_server = ApiServer::new(config.clone(), processor);

    // Start metrics exporter
    if let Err(e) = api_server.start_metrics().await {
        error!(error = %e, "Failed to start metrics exporter");
    }

    // Spawn cache maintenance task
    let cache_for_maintenance = cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cache_for_maintenance.maintenance().await;
        }
    });

    info!("Campaign Express is ready to serve traffic");

    // Start HTTP server (blocks until shutdown)
    api_server.start_http().await?;

    Ok(())
}
