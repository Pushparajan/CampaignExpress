//! Ingest processor — consumes real-time events from omnichannel sources
//! via NATS queue subscriptions and transforms them into internal events.

use campaign_core::channels::*;
use chrono::Utc;
use tracing::{debug, info};

/// Processes ingest events from all source channels.
pub struct IngestProcessor {
    enabled_sources: Vec<IngestSource>,
}

impl IngestProcessor {
    pub fn new(sources: Vec<IngestSource>) -> Self {
        info!(
            sources = ?sources,
            "Ingest processor initialized"
        );
        Self {
            enabled_sources: sources,
        }
    }

    /// Process a raw ingest event: validate, enrich, and route.
    pub fn process_event(&self, event: &IngestEvent) -> Result<ProcessedIngest, anyhow::Error> {
        if !self.enabled_sources.contains(&event.source) {
            return Err(anyhow::anyhow!(
                "Source {:?} not enabled",
                event.source
            ));
        }

        metrics::counter!(
            "ingest.events",
            "source" => event.source.display_name(),
            "type" => format!("{:?}", event.event_type)
        )
        .increment(1);

        // Validate required fields
        let user_id = event
            .user_id
            .clone()
            .or_else(|| event.device_id.clone())
            .ok_or_else(|| anyhow::anyhow!("No user_id or device_id in event"))?;

        // Determine if this event should trigger an activation
        let should_activate = self.should_trigger_activation(event);

        // Determine loyalty relevance
        let loyalty_relevant = matches!(
            event.event_type,
            IngestEventType::Purchase
                | IngestEventType::LoyaltySwipe
                | IngestEventType::CheckIn
        );

        debug!(
            event_id = %event.event_id,
            source = ?event.source,
            event_type = ?event.event_type,
            user_id = %user_id,
            should_activate = should_activate,
            "Ingest event processed"
        );

        Ok(ProcessedIngest {
            event_id: event.event_id.clone(),
            user_id,
            source: event.source,
            event_type: event.event_type,
            should_activate,
            loyalty_relevant,
            payload: event.payload.clone(),
            processed_at: Utc::now(),
        })
    }

    /// Determine if an ingest event should trigger a real-time activation.
    fn should_trigger_activation(&self, event: &IngestEvent) -> bool {
        match event.event_type {
            // High-intent events trigger immediate activation
            IngestEventType::CartAbandon => true,
            IngestEventType::Purchase => true, // Post-purchase upsell
            IngestEventType::StoreVisit => true,
            IngestEventType::LoyaltySwipe => true,
            IngestEventType::CheckIn => true,
            // Medium-intent: depends on recency
            IngestEventType::ProductView | IngestEventType::WishlistAdd => {
                // Only trigger if recent (within session)
                let age = Utc::now() - event.occurred_at;
                age.num_minutes() < 30
            }
            _ => false,
        }
    }

    /// Get NATS subjects to subscribe to for each source.
    pub fn nats_subjects(&self) -> Vec<String> {
        self.enabled_sources
            .iter()
            .map(|s| format!("ingest.{:?}.>", s).to_lowercase())
            .collect()
    }
}

/// Result of processing an ingest event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedIngest {
    pub event_id: String,
    pub user_id: String,
    pub source: IngestSource,
    pub event_type: IngestEventType,
    pub should_activate: bool,
    pub loyalty_relevant: bool,
    pub payload: serde_json::Value,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}
