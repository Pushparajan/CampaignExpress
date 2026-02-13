//! Web event types — click streams, page views, scrolls, form interactions,
//! and custom behavior events from web applications.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Top-level web event type for the SDK.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebEventType {
    PageView,
    Click,
    Scroll,
    FormSubmit,
    FormFieldChange,
    SessionStart,
    SessionEnd,
    CustomEvent,
    Error,
    PerformanceMark,
    MediaPlay,
    MediaPause,
    MediaComplete,
    FileDownload,
    OutboundLink,
    SiteSearch,
}

/// Device and browser context sent with each web event batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebContext {
    pub user_agent: String,
    pub language: String,
    pub screen_width: u32,
    pub screen_height: u32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub timezone: String,
    pub referrer: Option<String>,
    pub page_url: String,
    pub page_title: String,
}

/// A single web event captured by the SDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEvent {
    pub id: Uuid,
    pub event_type: WebEventType,
    pub user_id: Option<String>,
    pub anonymous_id: String,
    pub session_id: Uuid,
    pub name: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub context: WebContext,
    pub timestamp: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

/// Click-stream specific payload carried inside `properties`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickPayload {
    pub element_tag: String,
    pub element_id: Option<String>,
    pub element_classes: Vec<String>,
    pub element_text: Option<String>,
    pub href: Option<String>,
    pub x: f64,
    pub y: f64,
}

/// Scroll depth payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollPayload {
    pub depth_percent: u8,
    pub depth_pixels: u32,
    pub direction: ScrollDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Down,
    Up,
}

/// Form submission payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormPayload {
    pub form_id: Option<String>,
    pub form_name: Option<String>,
    pub form_action: Option<String>,
    pub field_count: u32,
}

/// Page view payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageViewPayload {
    pub url: String,
    pub title: String,
    pub referrer: Option<String>,
    pub load_time_ms: Option<u64>,
}

/// Batch of web events from a single page/session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEventBatch {
    pub api_key: String,
    pub anonymous_id: String,
    pub session_id: Uuid,
    pub context: WebContext,
    pub events: Vec<WebEvent>,
    pub sent_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_event_serde() {
        let event = WebEvent {
            id: Uuid::new_v4(),
            event_type: WebEventType::Click,
            user_id: Some("u-123".into()),
            anonymous_id: "anon-abc".into(),
            session_id: Uuid::new_v4(),
            name: Some("cta_button".into()),
            properties: HashMap::from([(
                "element_tag".to_string(),
                serde_json::json!("button"),
            )]),
            context: WebContext {
                user_agent: "Mozilla/5.0".into(),
                language: "en-US".into(),
                screen_width: 1920,
                screen_height: 1080,
                viewport_width: 1440,
                viewport_height: 900,
                timezone: "America/New_York".into(),
                referrer: Some("https://google.com".into()),
                page_url: "https://example.com/products".into(),
                page_title: "Products".into(),
            },
            timestamp: Utc::now(),
            received_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: WebEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, WebEventType::Click);
        assert_eq!(parsed.anonymous_id, "anon-abc");
    }

    #[test]
    fn test_click_payload_serde() {
        let click = ClickPayload {
            element_tag: "a".into(),
            element_id: Some("buy-btn".into()),
            element_classes: vec!["btn".into(), "btn-primary".into()],
            element_text: Some("Buy Now".into()),
            href: Some("https://example.com/checkout".into()),
            x: 450.5,
            y: 320.0,
        };
        let json = serde_json::to_string(&click).unwrap();
        let parsed: ClickPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.element_tag, "a");
        assert_eq!(parsed.element_classes.len(), 2);
    }
}
