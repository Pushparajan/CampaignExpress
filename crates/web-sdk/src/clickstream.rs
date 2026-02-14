//! Click-stream processor — aggregates click-stream data for heatmaps,
//! funnel analysis, and user journey reconstruction.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::{ClickPayload, WebEvent, WebEventType};

/// A processed click-stream session with ordered page trail and interaction summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickStreamSession {
    pub session_id: Uuid,
    pub anonymous_id: String,
    pub user_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub page_trail: Vec<PageHit>,
    pub click_count: u64,
    pub total_events: u64,
}

/// A single page visit within a click-stream session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageHit {
    pub url: String,
    pub title: String,
    pub entered_at: DateTime<Utc>,
    pub time_on_page_ms: Option<u64>,
    pub scroll_depth_percent: Option<u8>,
    pub click_count: u32,
}

/// Aggregated click-map entry for a single page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickMapEntry {
    pub page_url: String,
    pub element_tag: String,
    pub element_id: Option<String>,
    pub click_count: u64,
    pub unique_users: u64,
}

/// Builds click-stream sessions from raw web events.
pub struct ClickStreamProcessor {
    sessions: HashMap<Uuid, ClickStreamSession>,
    click_map: HashMap<String, Vec<ClickMapEntry>>,
}

impl Default for ClickStreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickStreamProcessor {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            click_map: HashMap::new(),
        }
    }

    /// Process a batch of web events and update internal sessions and click maps.
    pub fn process_events(&mut self, events: &[WebEvent]) {
        for event in events {
            self.process_single(event);
        }
    }

    fn process_single(&mut self, event: &WebEvent) {
        let session = self
            .sessions
            .entry(event.session_id)
            .or_insert_with(|| ClickStreamSession {
                session_id: event.session_id,
                anonymous_id: event.anonymous_id.clone(),
                user_id: event.user_id.clone(),
                started_at: event.timestamp,
                last_activity: event.timestamp,
                page_trail: Vec::new(),
                click_count: 0,
                total_events: 0,
            });

        session.total_events += 1;
        if event.timestamp > session.last_activity {
            session.last_activity = event.timestamp;
        }

        match event.event_type {
            WebEventType::PageView => {
                session.page_trail.push(PageHit {
                    url: event.context.page_url.clone(),
                    title: event.context.page_title.clone(),
                    entered_at: event.timestamp,
                    time_on_page_ms: None,
                    scroll_depth_percent: None,
                    click_count: 0,
                });
            }
            WebEventType::Click => {
                session.click_count += 1;
                if let Some(page) = session.page_trail.last_mut() {
                    page.click_count += 1;
                }

                // Update click map
                if let Ok(payload) =
                    serde_json::from_value::<ClickPayload>(serde_json::json!(event.properties))
                {
                    let entries = self
                        .click_map
                        .entry(event.context.page_url.clone())
                        .or_default();
                    if let Some(entry) = entries.iter_mut().find(|e| {
                        e.element_tag == payload.element_tag && e.element_id == payload.element_id
                    }) {
                        entry.click_count += 1;
                    } else {
                        entries.push(ClickMapEntry {
                            page_url: event.context.page_url.clone(),
                            element_tag: payload.element_tag,
                            element_id: payload.element_id,
                            click_count: 1,
                            unique_users: 1,
                        });
                    }
                }
            }
            WebEventType::Scroll => {
                if let Some(depth) = event.properties.get("depth_percent") {
                    if let Some(pct) = depth.as_u64() {
                        let capped_pct = pct.min(100) as u8;
                        if let Some(page) = session.page_trail.last_mut() {
                            let current = page.scroll_depth_percent.unwrap_or(0);
                            page.scroll_depth_percent = Some(current.max(capped_pct));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Retrieve a specific session's click-stream data.
    pub fn get_session(&self, session_id: &Uuid) -> Option<&ClickStreamSession> {
        self.sessions.get(session_id)
    }

    /// Retrieve all sessions.
    pub fn all_sessions(&self) -> Vec<&ClickStreamSession> {
        self.sessions.values().collect()
    }

    /// Get click-map data for a specific page URL.
    pub fn click_map(&self, page_url: &str) -> Vec<&ClickMapEntry> {
        self.click_map
            .get(page_url)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::WebContext;
    use std::collections::HashMap;

    fn ctx() -> WebContext {
        WebContext {
            user_agent: "test".into(),
            language: "en".into(),
            screen_width: 1920,
            screen_height: 1080,
            viewport_width: 1440,
            viewport_height: 900,
            timezone: "UTC".into(),
            referrer: None,
            page_url: "https://example.com/products".into(),
            page_title: "Products".into(),
        }
    }

    fn make_event(event_type: WebEventType, session_id: Uuid) -> WebEvent {
        let now = Utc::now();
        WebEvent {
            id: Uuid::new_v4(),
            event_type,
            user_id: Some("u-1".into()),
            anonymous_id: "anon-1".into(),
            session_id,
            name: None,
            properties: HashMap::new(),
            context: ctx(),
            timestamp: now,
            received_at: now,
        }
    }

    #[test]
    fn test_process_page_trail() {
        let mut proc = ClickStreamProcessor::new();
        let sid = Uuid::new_v4();

        let events = vec![
            make_event(WebEventType::PageView, sid),
            make_event(WebEventType::Click, sid),
            make_event(WebEventType::Click, sid),
            make_event(WebEventType::PageView, sid),
            make_event(WebEventType::Click, sid),
        ];

        proc.process_events(&events);

        let session = proc.get_session(&sid).unwrap();
        assert_eq!(session.page_trail.len(), 2);
        assert_eq!(session.click_count, 3);
        assert_eq!(session.total_events, 5);
        // First page got 2 clicks, second page got 1
        assert_eq!(session.page_trail[0].click_count, 2);
        assert_eq!(session.page_trail[1].click_count, 1);
    }

    #[test]
    fn test_scroll_depth_tracking() {
        let mut proc = ClickStreamProcessor::new();
        let sid = Uuid::new_v4();

        let mut pv = make_event(WebEventType::PageView, sid);
        pv.context.page_url = "https://example.com/article".into();

        let mut scroll1 = make_event(WebEventType::Scroll, sid);
        scroll1.context.page_url = "https://example.com/article".into();
        scroll1
            .properties
            .insert("depth_percent".into(), serde_json::json!(25));

        let mut scroll2 = make_event(WebEventType::Scroll, sid);
        scroll2.context.page_url = "https://example.com/article".into();
        scroll2
            .properties
            .insert("depth_percent".into(), serde_json::json!(75));

        proc.process_events(&[pv, scroll1, scroll2]);

        let session = proc.get_session(&sid).unwrap();
        assert_eq!(session.page_trail[0].scroll_depth_percent, Some(75));
    }
}
