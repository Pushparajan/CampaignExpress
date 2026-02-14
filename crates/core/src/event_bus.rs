//! Unified event bus — trait for emitting analytics events from any module.
//!
//! Modules accept an `Arc<dyn EventSink>` to emit events into the analytics
//! pipeline (ClickHouse), NATS topics, and customer webhooks.

use crate::types::{AnalyticsEvent, EventType};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Trait for emitting analytics events. Implementations route events to
/// ClickHouse (via mpsc), NATS (pub/sub), or customer webhooks.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: AnalyticsEvent);
}

/// No-op sink for tests and modules that don't need event emission.
pub struct NoOpSink;

impl EventSink for NoOpSink {
    fn emit(&self, _event: AnalyticsEvent) {}
}

/// In-memory sink that captures events for testing.
#[derive(Default)]
pub struct CaptureSink {
    events: Mutex<Vec<AnalyticsEvent>>,
}

impl CaptureSink {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn events(&self) -> Vec<AnalyticsEvent> {
        self.events.lock().expect("event bus mutex poisoned").clone()
    }

    pub fn count(&self) -> usize {
        self.events.lock().expect("event bus mutex poisoned").len()
    }

    pub fn count_type(&self, event_type: EventType) -> usize {
        self.events
            .lock()
            .expect("event bus mutex poisoned")
            .iter()
            .filter(|e| e.event_type == event_type)
            .count()
    }

    pub fn clear(&self) {
        self.events.lock().expect("event bus mutex poisoned").clear();
    }
}

impl EventSink for CaptureSink {
    fn emit(&self, event: AnalyticsEvent) {
        self.events.lock().expect("event bus mutex poisoned").push(event);
    }
}

/// Convenience builder for creating `AnalyticsEvent` with minimal boilerplate.
pub fn make_event(
    event_type: EventType,
    request_id: impl Into<String>,
    user_id: Option<String>,
    offer_id: Option<String>,
) -> AnalyticsEvent {
    AnalyticsEvent {
        event_id: Uuid::new_v4(),
        event_type,
        request_id: request_id.into(),
        impression_id: None,
        user_id,
        offer_id,
        bid_price: None,
        win_price: None,
        agent_id: "system".into(),
        node_id: "local".into(),
        inference_latency_us: None,
        total_latency_us: None,
        timestamp: Utc::now(),
    }
}

/// Convenience: create a no-op event bus for modules that don't need it.
pub fn noop_sink() -> Arc<dyn EventSink> {
    Arc::new(NoOpSink)
}

/// Convenience: create a capture sink for tests.
pub fn capture_sink() -> Arc<CaptureSink> {
    Arc::new(CaptureSink::new())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_sink() {
        let sink = capture_sink();
        assert_eq!(sink.count(), 0);

        sink.emit(make_event(
            EventType::ChannelIngest,
            "req-1",
            Some("user-1".into()),
            None,
        ));
        sink.emit(make_event(
            EventType::ActivationSent,
            "req-2",
            Some("user-1".into()),
            Some("offer-1".into()),
        ));

        assert_eq!(sink.count(), 2);
        assert_eq!(sink.count_type(EventType::ChannelIngest), 1);
        assert_eq!(sink.count_type(EventType::ActivationSent), 1);

        let events = sink.events();
        assert_eq!(events[0].request_id, "req-1");
        assert_eq!(events[1].offer_id, Some("offer-1".into()));
    }

    #[test]
    fn test_noop_sink() {
        let sink = noop_sink();
        // Should not panic
        sink.emit(make_event(EventType::BidRequest, "req-1", None, None));
    }
}
