//! Google Tag Manager adaptor — transforms web events into GTM dataLayer
//! push payloads that can be forwarded to a server-side GTM container.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::WebAdaptor;
use crate::events::{WebEvent, WebEventType};

/// Configuration for the GTM adaptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtmConfig {
    /// GTM container ID, e.g. "GTM-XXXXXXX".
    pub container_id: String,
    /// Server-side GTM endpoint URL (optional — if set, events are forwarded).
    pub server_endpoint: Option<String>,
    /// Whether to include full context in each push (default: true).
    pub include_context: bool,
    /// Custom dataLayer variable name (default: "dataLayer").
    pub data_layer_name: String,
}

impl Default for GtmConfig {
    fn default() -> Self {
        Self {
            container_id: String::new(),
            server_endpoint: None,
            include_context: true,
            data_layer_name: "dataLayer".into(),
        }
    }
}

/// Google Tag Manager adaptor.
pub struct GtmAdaptor {
    config: GtmConfig,
}

impl GtmAdaptor {
    pub fn new(config: GtmConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &GtmConfig {
        &self.config
    }

    /// Map a web event type to a GTM event name.
    fn gtm_event_name(event_type: WebEventType) -> &'static str {
        match event_type {
            WebEventType::PageView => "page_view",
            WebEventType::Click => "click",
            WebEventType::Scroll => "scroll",
            WebEventType::FormSubmit => "form_submit",
            WebEventType::FormFieldChange => "form_field_change",
            WebEventType::SessionStart => "session_start",
            WebEventType::SessionEnd => "session_end",
            WebEventType::CustomEvent => "custom_event",
            WebEventType::Error => "exception",
            WebEventType::PerformanceMark => "performance_mark",
            WebEventType::MediaPlay => "video_start",
            WebEventType::MediaPause => "video_pause",
            WebEventType::MediaComplete => "video_complete",
            WebEventType::FileDownload => "file_download",
            WebEventType::OutboundLink => "outbound_click",
            WebEventType::SiteSearch => "search",
        }
    }
}

impl WebAdaptor for GtmAdaptor {
    fn platform(&self) -> &str {
        "gtm"
    }

    fn transform(&self, event: &WebEvent) -> Result<serde_json::Value> {
        let event_name = Self::gtm_event_name(event.event_type);

        let mut payload = serde_json::json!({
            "event": event_name,
            "gtm.uniqueEventId": event.id.to_string(),
            "campaign_session_id": event.session_id.to_string(),
            "timestamp": event.timestamp.to_rfc3339(),
        });

        // Add user identity
        if let Some(ref uid) = event.user_id {
            payload["user_id"] = serde_json::json!(uid);
        }
        payload["anonymous_id"] = serde_json::json!(event.anonymous_id);

        // Add event name if custom
        if let Some(ref name) = event.name {
            payload["event_name"] = serde_json::json!(name);
        }

        // Merge event properties
        for (key, value) in &event.properties {
            payload[key] = value.clone();
        }

        // Add page data for page views
        if event.event_type == WebEventType::PageView {
            payload["page_location"] = serde_json::json!(event.context.page_url);
            payload["page_title"] = serde_json::json!(event.context.page_title);
            if let Some(ref referrer) = event.context.referrer {
                payload["page_referrer"] = serde_json::json!(referrer);
            }
        }

        // Add context if configured
        if self.config.include_context {
            payload["screen_resolution"] = serde_json::json!(format!(
                "{}x{}",
                event.context.screen_width, event.context.screen_height
            ));
            payload["viewport_size"] = serde_json::json!(format!(
                "{}x{}",
                event.context.viewport_width, event.context.viewport_height
            ));
            payload["language"] = serde_json::json!(event.context.language);
        }

        debug!(
            event_name,
            container_id = %self.config.container_id,
            "GTM dataLayer push transformed"
        );

        Ok(payload)
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.container_id.is_empty() {
            return Err(anyhow!("GTM container_id must not be empty"));
        }
        if !self.config.container_id.starts_with("GTM-") {
            return Err(anyhow!(
                "GTM container_id must start with 'GTM-', got '{}'",
                self.config.container_id
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::WebContext;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn test_config() -> GtmConfig {
        GtmConfig {
            container_id: "GTM-ABC1234".into(),
            server_endpoint: Some("https://gtm.example.com".into()),
            include_context: true,
            data_layer_name: "dataLayer".into(),
        }
    }

    fn test_event(event_type: WebEventType) -> WebEvent {
        let now = Utc::now();
        WebEvent {
            id: Uuid::new_v4(),
            event_type,
            user_id: Some("user-42".into()),
            anonymous_id: "anon-xyz".into(),
            session_id: Uuid::new_v4(),
            name: None,
            properties: HashMap::new(),
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
            timestamp: now,
            received_at: now,
        }
    }

    #[test]
    fn test_page_view_transform() {
        let adaptor = GtmAdaptor::new(test_config());
        let event = test_event(WebEventType::PageView);

        let payload = adaptor.transform(&event).unwrap();
        assert_eq!(payload["event"], "page_view");
        assert_eq!(payload["page_location"], "https://example.com/products");
        assert_eq!(payload["page_title"], "Products");
        assert_eq!(payload["page_referrer"], "https://google.com");
        assert_eq!(payload["user_id"], "user-42");
        assert_eq!(payload["screen_resolution"], "1920x1080");
    }

    #[test]
    fn test_click_transform() {
        let adaptor = GtmAdaptor::new(test_config());
        let mut event = test_event(WebEventType::Click);
        event
            .properties
            .insert("element_tag".into(), serde_json::json!("button"));
        event
            .properties
            .insert("element_id".into(), serde_json::json!("cta-buy"));

        let payload = adaptor.transform(&event).unwrap();
        assert_eq!(payload["event"], "click");
        assert_eq!(payload["element_tag"], "button");
        assert_eq!(payload["element_id"], "cta-buy");
    }

    #[test]
    fn test_batch_transform() {
        let adaptor = GtmAdaptor::new(test_config());
        let events = vec![
            test_event(WebEventType::PageView),
            test_event(WebEventType::Click),
            test_event(WebEventType::Scroll),
        ];

        let payloads = adaptor.transform_batch(&events).unwrap();
        assert_eq!(payloads.len(), 3);
        assert_eq!(payloads[0]["event"], "page_view");
        assert_eq!(payloads[1]["event"], "click");
        assert_eq!(payloads[2]["event"], "scroll");
    }

    #[test]
    fn test_validate_config() {
        let adaptor = GtmAdaptor::new(test_config());
        assert!(adaptor.validate_config().is_ok());

        let bad = GtmAdaptor::new(GtmConfig {
            container_id: "".into(),
            ..Default::default()
        });
        assert!(bad.validate_config().is_err());

        let bad2 = GtmAdaptor::new(GtmConfig {
            container_id: "XYZ-123".into(),
            ..Default::default()
        });
        assert!(bad2.validate_config().is_err());
    }

    #[test]
    fn test_context_excluded() {
        let adaptor = GtmAdaptor::new(GtmConfig {
            container_id: "GTM-TEST123".into(),
            include_context: false,
            ..Default::default()
        });

        let event = test_event(WebEventType::Click);
        let payload = adaptor.transform(&event).unwrap();
        assert!(payload.get("screen_resolution").is_none());
        assert!(payload.get("viewport_size").is_none());
    }
}
