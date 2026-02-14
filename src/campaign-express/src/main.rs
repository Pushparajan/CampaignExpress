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
use tracing::{error, info, warn};

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
        warn!(error = %e, "Failed to load config, using defaults");
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

    // Initialize Redis cache with retry
    let cache = Arc::new(
        connect_with_retry("Redis", || RedisCache::new(&config.redis)).await?,
    );

    // Initialize analytics logger with retry
    let analytics = Arc::new(
        connect_with_retry("ClickHouse", || {
            AnalyticsLogger::new(&config.clickhouse, config.node_id.clone())
        })
        .await?,
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
            Ok(_) => info!(
                "Agent manager started with {} agents",
                config.agents_per_node
            ),
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

    // Graceful shutdown: listen for SIGTERM/SIGINT
    let shutdown = async {
        let ctrl_c = tokio::signal::ctrl_c();

        #[cfg(unix)]
        {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("failed to register SIGTERM handler");
            tokio::select! {
                _ = ctrl_c => info!("Received SIGINT, shutting down"),
                _ = sigterm.recv() => info!("Received SIGTERM, shutting down"),
            }
        }

        #[cfg(not(unix))]
        {
            ctrl_c.await.ok();
            info!("Received SIGINT, shutting down");
        }
    };

    // Start HTTP server with graceful shutdown
    let addr = std::net::SocketAddr::new(
        config.api.host.parse()?,
        config.api.http_port,
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(addr = %addr, "Starting HTTP server");
    axum::serve(listener, api_server.into_router()?)
        .with_graceful_shutdown(shutdown)
        .await?;

    info!("Campaign Express shut down cleanly");
    Ok(())
}

/// Connect to an external service with exponential backoff (3 attempts).
async fn connect_with_retry<T, F, Fut>(service_name: &str, connect_fn: F) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    let delays = [
        std::time::Duration::from_secs(0),
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(4),
    ];
    let mut last_err = None;
    for (attempt, delay) in delays.iter().enumerate() {
        if attempt > 0 {
            warn!(service = service_name, attempt, "Retrying connection after {}s", delay.as_secs());
            tokio::time::sleep(*delay).await;
        }
        match connect_fn().await {
            Ok(conn) => {
                info!(service = service_name, "Connected successfully");
                return Ok(conn);
            }
            Err(e) => {
                error!(service = service_name, attempt, error = %e, "Connection failed");
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("{} connection failed", service_name)))
}
