//! Individual bid agent — a Tokio task that subscribes to a NATS queue,
//! processes bid requests, and publishes responses.

use crate::processor::BidProcessor;
use campaign_core::openrtb::BidRequest;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// A single autonomous bid processing agent.
pub struct BidAgent {
    pub agent_id: String,
    pub node_id: String,
    processor: Arc<BidProcessor>,
}

impl BidAgent {
    pub fn new(agent_id: String, node_id: String, processor: Arc<BidProcessor>) -> Self {
        Self {
            agent_id,
            node_id,
            processor,
        }
    }

    /// Spawn this agent as a Tokio task that consumes from a NATS queue.
    pub fn spawn(self, nats_client: async_nats::Client, subject: String) -> JoinHandle<()> {
        let agent_id = self.agent_id.clone();
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            info!(
                agent_id = %agent_id,
                node_id = %node_id,
                subject = %subject,
                "Agent started, subscribing to NATS queue"
            );

            let subscriber = match nats_client
                .queue_subscribe(subject.clone(), "bid-agents".to_string())
                .await
            {
                Ok(sub) => sub,
                Err(e) => {
                    error!(agent_id = %agent_id, error = %e, "Failed to subscribe to NATS");
                    return;
                }
            };

            self.process_messages(subscriber).await;
        })
    }

    async fn process_messages(self, mut subscriber: async_nats::Subscriber) {
        while let Some(msg) = subscriber.next().await {
            let request: BidRequest = match serde_json::from_slice(&msg.payload) {
                Ok(req) => req,
                Err(e) => {
                    warn!(
                        agent_id = %self.agent_id,
                        error = %e,
                        "Failed to deserialize bid request"
                    );
                    metrics::counter!("agent.deserialize_errors").increment(1);
                    continue;
                }
            };

            match self.processor.process(&request, &self.agent_id).await {
                Ok(response) => {
                    if let Some(reply) = msg.reply {
                        let payload = match serde_json::to_vec(&response) {
                            Ok(p) => p,
                            Err(e) => {
                                error!(
                                    agent_id = %self.agent_id,
                                    error = %e,
                                    "Failed to serialize bid response"
                                );
                                continue;
                            }
                        };
                        if let Err(e) = nats_publish(&self.agent_id, reply, payload).await {
                            error!(agent_id = %self.agent_id, error = %e, "Failed to publish response");
                        }
                    }
                }
                Err(e) => {
                    error!(
                        agent_id = %self.agent_id,
                        request_id = %request.id,
                        error = %e,
                        "Bid processing failed"
                    );
                    metrics::counter!("agent.processing_errors").increment(1);
                }
            }
        }

        warn!(agent_id = %self.agent_id, "NATS subscription ended");
    }
}

/// Placeholder for NATS publish — in the full flow, the agent would hold
/// its own client reference. For now this is a stub.
async fn nats_publish(
    _agent_id: &str,
    _reply: async_nats::Subject,
    _payload: Vec<u8>,
) -> anyhow::Result<()> {
    // In the real flow, the NATS client is used here to publish the response.
    // The API server handles direct HTTP responses instead of NATS reply for REST.
    Ok(())
}

use tokio_stream::StreamExt;
