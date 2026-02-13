//! Google Analytics 4 (GA4) Measurement Protocol adaptor — transforms web
//! events into GA4 event payloads for server-side forwarding via the
//! Measurement Protocol.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::WebAdaptor;
use crate::events::{WebEvent, WebEventType};

/// Configuration for the GA4 adaptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaConfig {
    /// GA4 Measurement ID, e.g. "G-XXXXXXXXXX".
    pub measurement_id: String,
    /// API secret for Measurement Protocol server-side hits.
    pub api_secret: String,
    /// Whether to send user properties alongside events (default: true).
    pub send_user_properties: bool,
    /// Enable debug mode for GA4 validation (default: false).
    pub debug_mode: bool,
}

impl Default for GaConfig {
    fn default() -> Self {
        Self {
            measurement_id: String::new(),
            api_secret: String::new(),
            send_user_properties: true,
            debug_mode: false,
        }
    }
}

/// Google Analytics 4 adaptor using the Measurement Protocol format.
pub struct GaAdaptor {
    config: GaConfig,
}

impl GaAdaptor {
    pub fn new(config: GaConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &GaConfig {
        &self.config
    }

    /// Map a web event type to a GA4 event name following Google's recommended
    /// event naming conventions.
    fn ga4_event_name(event_type: WebEventType) -> &'static str {
        match event_type {
            WebEventType::PageView => "page_view",
            WebEventType::Click => "click",
            WebEventType::Scroll => "scroll",
            WebEventType::FormSubmit => "generate_lead",
            WebEventType::FormFieldChange => "form_field_interaction",
            WebEventType::SessionStart => "session_start",
            WebEventType::SessionEnd => "session_end",
            WebEventType::CustomEvent => "custom_event",
            WebEventType::Error => "exception",
            WebEventType::PerformanceMark => "performance_timing",
            WebEventType::MediaPlay => "video_start",
            WebEventType::MediaPause => "video_pause",
            WebEventType::MediaComplete => "video_complete",
            WebEventType::FileDownload => "file_download",
            WebEventType::OutboundLink => "click",
            WebEventType::SiteSearch => "search",
        }
    }

    /// Build the GA4 Measurement Protocol event parameters.
    fn build_params(event: &WebEvent) -> serde_json::Value {
        let mut params = serde_json::json!({
            "page_location": event.context.page_url,
            "page_title": event.context.page_title,
            "language": event.context.language,
            "screen_resolution": format!("{}x{}", event.context.screen_width, event.context.screen_height),
        });

        if let Some(ref referrer) = event.context.referrer {
            params["page_referrer"] = serde_json::json!(referrer);
        }

        // Merge custom properties
        if let Some(obj) = params.as_object_mut() {
            for (key, value) in &event.properties {
                obj.insert(key.clone(), value.clone());
            }
        }

        // Add event name for custom events
        if let Some(ref name) = event.name {
            params["event_label"] = serde_json::json!(name);
        }

        params
    }
}

impl WebAdaptor for GaAdaptor {
    fn platform(&self) -> &str {
        "ga4"
    }

    fn transform(&self, event: &WebEvent) -> Result<serde_json::Value> {
        let event_name = Self::ga4_event_name(event.event_type);
        let params = Self::build_params(event);

        let mut ga_event = serde_json::json!({
            "name": event_name,
            "params": params,
        });

        // GA4 Measurement Protocol payload wraps events in a top-level object
        let mut payload = serde_json::json!({
            "client_id": event.anonymous_id,
            "timestamp_micros": event.timestamp.timestamp_micros().to_string(),
            "events": [ga_event.clone()],
        });

        // Set user_id if identified
        if let Some(ref uid) = event.user_id {
            payload["user_id"] = serde_json::json!(uid);
        }

        // Add user properties
        if self.config.send_user_properties {
            payload["user_properties"] = serde_json::json!({
                "session_id": {
                    "value": event.session_id.to_string()
                },
                "timezone": {
                    "value": event.context.timezone
                }
            });
        }

        // Debug mode flag
        if self.config.debug_mode {
            if let Some(ev) = ga_event.as_object_mut() {
                if let Some(p) = ev.get_mut("params") {
                    p["debug_mode"] = serde_json::json!(true);
                }
            }
            payload["events"] = serde_json::json!([ga_event]);
        }

        debug!(
            event_name,
            measurement_id = %self.config.measurement_id,
            "GA4 event transformed"
        );

        Ok(payload)
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.measurement_id.is_empty() {
            return Err(anyhow!("GA4 measurement_id must not be empty"));
        }
        if !self.config.measurement_id.starts_with("G-") {
            return Err(anyhow!(
                "GA4 measurement_id must start with 'G-', got '{}'",
                self.config.measurement_id
            ));
        }
        if self.config.api_secret.is_empty() {
            return Err(anyhow!("GA4 api_secret must not be empty"));
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

    fn test_config() -> GaConfig {
        GaConfig {
            measurement_id: "G-TEST12345".into(),
            api_secret: "secret-abc".into(),
            send_user_properties: true,
            debug_mode: false,
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
        let adaptor = GaAdaptor::new(test_config());
        let event = test_event(WebEventType::PageView);

        let payload = adaptor.transform(&event).unwrap();
        assert_eq!(payload["client_id"], "anon-xyz");
        assert_eq!(payload["user_id"], "user-42");

        let events = payload["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["name"], "page_view");
        assert_eq!(
            events[0]["params"]["page_location"],
            "https://example.com/products"
        );
        assert_eq!(events[0]["params"]["page_title"], "Products");
        assert_eq!(
            events[0]["params"]["page_referrer"],
            "https://google.com"
        );
    }

    #[test]
    fn test_custom_event_transform() {
        let adaptor = GaAdaptor::new(test_config());
        let mut event = test_event(WebEventType::CustomEvent);
        event.name = Some("add_to_cart".into());
        event
            .properties
            .insert("item_id".into(), serde_json::json!("SKU-123"));
        event
            .properties
            .insert("price".into(), serde_json::json!(29.99));

        let payload = adaptor.transform(&event).unwrap();
        let ga_event = &payload["events"][0];
        assert_eq!(ga_event["name"], "custom_event");
        assert_eq!(ga_event["params"]["event_label"], "add_to_cart");
        assert_eq!(ga_event["params"]["item_id"], "SKU-123");
        assert_eq!(ga_event["params"]["price"], 29.99);
    }

    #[test]
    fn test_user_properties_included() {
        let adaptor = GaAdaptor::new(test_config());
        let event = test_event(WebEventType::PageView);
        let payload = adaptor.transform(&event).unwrap();

        assert!(payload.get("user_properties").is_some());
        assert!(payload["user_properties"]["timezone"]["value"]
            .as_str()
            .unwrap()
            .contains("New_York"));
    }

    #[test]
    fn test_user_properties_excluded() {
        let adaptor = GaAdaptor::new(GaConfig {
            send_user_properties: false,
            ..test_config()
        });
        let event = test_event(WebEventType::Click);
        let payload = adaptor.transform(&event).unwrap();
        assert!(payload.get("user_properties").is_none());
    }

    #[test]
    fn test_debug_mode() {
        let adaptor = GaAdaptor::new(GaConfig {
            debug_mode: true,
            ..test_config()
        });
        let event = test_event(WebEventType::Click);
        let payload = adaptor.transform(&event).unwrap();
        let ga_event = &payload["events"][0];
        assert_eq!(ga_event["params"]["debug_mode"], true);
    }

    #[test]
    fn test_validate_config() {
        let adaptor = GaAdaptor::new(test_config());
        assert!(adaptor.validate_config().is_ok());

        let bad = GaAdaptor::new(GaConfig {
            measurement_id: "".into(),
            ..test_config()
        });
        assert!(bad.validate_config().is_err());

        let bad2 = GaAdaptor::new(GaConfig {
            measurement_id: "UA-12345".into(),
            ..test_config()
        });
        assert!(bad2.validate_config().is_err());

        let bad3 = GaAdaptor::new(GaConfig {
            api_secret: "".into(),
            ..test_config()
        });
        assert!(bad3.validate_config().is_err());
    }

    #[test]
    fn test_batch_transform() {
        let adaptor = GaAdaptor::new(test_config());
        let events = vec![
            test_event(WebEventType::PageView),
            test_event(WebEventType::Click),
            test_event(WebEventType::FormSubmit),
        ];

        let payloads = adaptor.transform_batch(&events).unwrap();
        assert_eq!(payloads.len(), 3);
        assert_eq!(payloads[0]["events"][0]["name"], "page_view");
        assert_eq!(payloads[1]["events"][0]["name"], "click");
        assert_eq!(payloads[2]["events"][0]["name"], "generate_lead");
    }
}
