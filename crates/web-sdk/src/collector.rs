//! Event collector — ingests web events, normalises them, and forwards to the
//! event bus. Also maintains per-session metrics (page views, clicks, duration).

use std::sync::Arc;

use dashmap::DashMap;
use tracing::{debug, info};
use uuid::Uuid;

use campaign_core::event_bus::{make_event, EventSink};
use campaign_core::types::EventType;

use crate::events::{WebEvent, WebEventBatch, WebEventType};

/// Per-session aggregate counters.
#[derive(Debug, Clone, Default)]
pub struct SessionMetrics {
    pub session_id: Uuid,
    pub page_views: u64,
    pub clicks: u64,
    pub scrolls: u64,
    pub form_submits: u64,
    pub custom_events: u64,
    pub total_events: u64,
}

/// Collects web events, emits to the event bus, and tracks per-session metrics.
pub struct WebEventCollector {
    buffer: Vec<WebEvent>,
    buffer_capacity: usize,
    session_metrics: DashMap<Uuid, SessionMetrics>,
    event_sink: Arc<dyn EventSink>,
}

impl WebEventCollector {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_capacity),
            buffer_capacity,
            session_metrics: DashMap::new(),
            event_sink: campaign_core::event_bus::noop_sink(),
        }
    }

    /// Attach an event sink for emitting analytics events.
    pub fn with_event_sink(mut self, sink: Arc<dyn EventSink>) -> Self {
        self.event_sink = sink;
        self
    }

    /// Ingest a single web event.
    pub fn ingest(&mut self, event: WebEvent) {
        let core_event_type = map_event_type(event.event_type);

        self.event_sink.emit(make_event(
            core_event_type,
            event.id.to_string(),
            event.user_id.clone(),
            None,
        ));

        // Update session metrics (scoped to drop the DashMap ref before flush)
        {
            let mut metrics = self
                .session_metrics
                .entry(event.session_id)
                .or_insert_with(|| SessionMetrics {
                    session_id: event.session_id,
                    ..Default::default()
                });
            metrics.total_events += 1;
            match event.event_type {
                WebEventType::PageView => metrics.page_views += 1,
                WebEventType::Click | WebEventType::OutboundLink => metrics.clicks += 1,
                WebEventType::Scroll => metrics.scrolls += 1,
                WebEventType::FormSubmit => metrics.form_submits += 1,
                WebEventType::CustomEvent => metrics.custom_events += 1,
                _ => {}
            }
        }

        debug!(
            event_id = %event.id,
            event_type = ?event.event_type,
            session_id = %event.session_id,
            "web event ingested"
        );

        self.buffer.push(event);
        if self.buffer.len() >= self.buffer_capacity {
            self.flush();
        }
    }

    /// Ingest a full batch of web events.
    pub fn ingest_batch(&mut self, batch: WebEventBatch) {
        info!(
            anonymous_id = %batch.anonymous_id,
            session_id = %batch.session_id,
            event_count = batch.events.len(),
            "ingesting web event batch"
        );
        for event in batch.events {
            self.ingest(event);
        }
    }

    /// Flush buffered events and return them.
    pub fn flush(&mut self) -> Vec<WebEvent> {
        let flushed = std::mem::take(&mut self.buffer);
        if !flushed.is_empty() {
            info!(count = flushed.len(), "flushed web event buffer");
        }
        flushed
    }

    /// Number of events currently buffered.
    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }

    /// Get session metrics for a specific session.
    pub fn session_metrics(&self, session_id: &Uuid) -> Option<SessionMetrics> {
        self.session_metrics.get(session_id).map(|m| m.clone())
    }

    /// Get all session metrics.
    pub fn all_session_metrics(&self) -> Vec<SessionMetrics> {
        self.session_metrics
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// Map web event types to core `EventType` variants.
fn map_event_type(web_type: WebEventType) -> EventType {
    match web_type {
        WebEventType::PageView => EventType::WebPageView,
        WebEventType::Click | WebEventType::OutboundLink => EventType::WebClick,
        WebEventType::Scroll => EventType::WebScroll,
        WebEventType::FormSubmit | WebEventType::FormFieldChange => EventType::WebFormSubmit,
        WebEventType::SessionStart => EventType::WebSessionStart,
        WebEventType::SessionEnd => EventType::WebSessionEnd,
        WebEventType::CustomEvent
        | WebEventType::Error
        | WebEventType::PerformanceMark
        | WebEventType::MediaPlay
        | WebEventType::MediaPause
        | WebEventType::MediaComplete
        | WebEventType::FileDownload
        | WebEventType::SiteSearch => EventType::WebCustomEvent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::WebContext;
    use campaign_core::event_bus::capture_sink;
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_web_event(event_type: WebEventType, session_id: Uuid) -> WebEvent {
        let now = Utc::now();
        WebEvent {
            id: Uuid::new_v4(),
            event_type,
            user_id: Some("u-1".into()),
            anonymous_id: "anon-1".into(),
            session_id,
            name: None,
            properties: HashMap::new(),
            context: WebContext {
                user_agent: "test".into(),
                language: "en".into(),
                screen_width: 1920,
                screen_height: 1080,
                viewport_width: 1440,
                viewport_height: 900,
                timezone: "UTC".into(),
                referrer: None,
                page_url: "https://example.com".into(),
                page_title: "Test".into(),
            },
            timestamp: now,
            received_at: now,
        }
    }

    #[test]
    fn test_ingest_and_flush() {
        let sink = capture_sink();
        let mut collector =
            WebEventCollector::new(100).with_event_sink(sink.clone() as Arc<dyn EventSink>);

        let session_id = Uuid::new_v4();
        collector.ingest(make_web_event(WebEventType::PageView, session_id));
        collector.ingest(make_web_event(WebEventType::Click, session_id));
        collector.ingest(make_web_event(WebEventType::Scroll, session_id));

        assert_eq!(collector.buffered_count(), 3);
        assert_eq!(sink.count(), 3);
        assert_eq!(sink.count_type(EventType::WebPageView), 1);
        assert_eq!(sink.count_type(EventType::WebClick), 1);
        assert_eq!(sink.count_type(EventType::WebScroll), 1);

        let flushed = collector.flush();
        assert_eq!(flushed.len(), 3);
        assert_eq!(collector.buffered_count(), 0);
    }

    #[test]
    fn test_session_metrics() {
        let mut collector = WebEventCollector::new(100);
        let session_id = Uuid::new_v4();

        collector.ingest(make_web_event(WebEventType::PageView, session_id));
        collector.ingest(make_web_event(WebEventType::Click, session_id));
        collector.ingest(make_web_event(WebEventType::Click, session_id));
        collector.ingest(make_web_event(WebEventType::FormSubmit, session_id));
        collector.ingest(make_web_event(WebEventType::CustomEvent, session_id));

        let metrics = collector.session_metrics(&session_id).unwrap();
        assert_eq!(metrics.page_views, 1);
        assert_eq!(metrics.clicks, 2);
        assert_eq!(metrics.form_submits, 1);
        assert_eq!(metrics.custom_events, 1);
        assert_eq!(metrics.total_events, 5);
    }

    #[test]
    fn test_auto_flush() {
        let mut collector = WebEventCollector::new(3);
        let sid = Uuid::new_v4();

        collector.ingest(make_web_event(WebEventType::PageView, sid));
        collector.ingest(make_web_event(WebEventType::Click, sid));
        assert_eq!(collector.buffered_count(), 2);

        // Third event triggers auto-flush
        collector.ingest(make_web_event(WebEventType::Scroll, sid));
        assert_eq!(collector.buffered_count(), 0);
    }
}
