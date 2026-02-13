//! Event ingestion — tracking user events from mobile SDKs.
//! Emits `ChannelIngest` events for every ingested SDK event.

use campaign_core::event_bus::{make_event, EventSink};
use campaign_core::types::EventType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkEventType {
    SessionStart,
    SessionEnd,
    CustomEvent,
    Purchase,
    ScreenView,
    PushOpen,
    PushReceived,
    InAppImpression,
    InAppClick,
    InAppDismiss,
    ContentCardImpression,
    ContentCardClick,
    ContentCardDismiss,
    LocationUpdate,
    GeofenceEntry,
    GeofenceExit,
    AttributeChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkEvent {
    pub id: Uuid,
    pub event_type: SdkEventType,
    pub user_id: Option<Uuid>,
    pub device_id: String,
    pub session_id: Option<Uuid>,
    pub name: Option<String>,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    pub api_key: String,
    pub device_id: String,
    pub events: Vec<SdkEvent>,
}

pub struct EventIngester {
    buffer: Vec<SdkEvent>,
    buffer_capacity: usize,
    event_sink: Arc<dyn EventSink>,
}

impl EventIngester {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_capacity),
            buffer_capacity,
            event_sink: campaign_core::event_bus::noop_sink(),
        }
    }

    /// Attach an event sink for emitting analytics events.
    pub fn with_event_sink(mut self, sink: Arc<dyn EventSink>) -> Self {
        self.event_sink = sink;
        self
    }

    pub fn ingest(&mut self, event: SdkEvent) {
        // Emit ChannelIngest event for each SDK event
        self.event_sink.emit(make_event(
            EventType::ChannelIngest,
            event.id.to_string(),
            event.user_id.map(|u| u.to_string()),
            None,
        ));

        self.buffer.push(event);
        if self.buffer.len() >= self.buffer_capacity {
            self.flush();
        }
    }

    pub fn ingest_batch(&mut self, batch: EventBatch) {
        for event in batch.events {
            self.ingest(event);
        }
    }

    pub fn flush(&mut self) -> Vec<SdkEvent> {
        std::mem::take(&mut self.buffer)
    }

    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }
}
