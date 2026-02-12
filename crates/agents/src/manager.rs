//! Agent manager — spawns and supervises N bid agents per node.

use crate::agent::BidAgent;
use crate::processor::BidProcessor;
use campaign_analytics::AnalyticsLogger;
use campaign_cache::RedisCache;
use campaign_core::config::AppConfig;
use campaign_npu::NpuEngine;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// Manages the lifecycle of all bid agents on this node.
pub struct AgentManager {
    config: AppConfig,
    npu: Arc<NpuEngine>,
    cache: Arc<RedisCache>,
    analytics: Arc<AnalyticsLogger>,
    handles: Vec<JoinHandle<()>>,
}

impl AgentManager {
    pub fn new(
        config: AppConfig,
        npu: Arc<NpuEngine>,
        cache: Arc<RedisCache>,
        analytics: Arc<AnalyticsLogger>,
    ) -> Self {
        Self {
            config,
            npu,
            cache,
            analytics,
            handles: Vec::new(),
        }
    }

    /// Connect to NATS and spawn all agents.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let nats_url = self
            .config
            .nats
            .urls
            .first()
            .cloned()
            .unwrap_or_else(|| "nats://localhost:4222".to_string());

        info!(url = %nats_url, "Connecting to NATS");

        let nats_client = async_nats::ConnectOptions::new()
            .max_reconnects(Some(self.config.nats.max_reconnects))
            .connect(&nats_url)
            .await?;

        info!("NATS connection established");

        let processor = Arc::new(BidProcessor::new(
            self.npu.clone(),
            self.cache.clone(),
            self.analytics.clone(),
            self.config.node_id.clone(),
        ));

        let subject = format!("{}.bid-requests", self.config.nats.stream_name);

        for i in 0..self.config.agents_per_node {
            let agent_id = format!("{}-agent-{:02}", self.config.node_id, i);

            let agent = BidAgent::new(
                agent_id.clone(),
                self.config.node_id.clone(),
                processor.clone(),
            );

            let handle = agent.spawn(nats_client.clone(), subject.clone());
            self.handles.push(handle);

            info!(agent_id = %agent_id, "Agent spawned");
        }

        info!(
            count = self.config.agents_per_node,
            node = %self.config.node_id,
            "All agents started"
        );

        Ok(())
    }

    /// Get a reference to the bid processor for direct API use.
    pub fn processor(&self) -> Arc<BidProcessor> {
        Arc::new(BidProcessor::new(
            self.npu.clone(),
            self.cache.clone(),
            self.analytics.clone(),
            self.config.node_id.clone(),
        ))
    }

    /// Wait for all agents to complete (blocks until shutdown).
    pub async fn wait(&mut self) {
        for handle in self.handles.drain(..) {
            if let Err(e) = handle.await {
                error!(error = %e, "Agent task panicked");
            }
        }
    }

    pub fn agent_count(&self) -> usize {
        self.handles.len()
    }
}
