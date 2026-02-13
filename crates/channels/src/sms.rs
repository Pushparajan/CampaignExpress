//! Twilio SMS provider — send, track, and manage SMS messages with
//! segment calculation, delivery callbacks, and bulk send support.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for the Twilio SMS provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
    pub messaging_service_sid: Option<String>,
    pub status_callback_url: Option<String>,
}

/// Status of an SMS message through its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsStatus {
    Queued,
    Sent,
    Delivered,
    Failed,
    Undelivered,
}

/// An SMS message with metadata and delivery tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub id: Uuid,
    pub to: String,
    pub from: String,
    pub body: String,
    pub media_url: Option<String>,
    pub status: SmsStatus,
    pub provider_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub segments: u32,
}

/// A delivery event received from Twilio's status callback webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsDeliveryEvent {
    pub message_id: Uuid,
    pub status: SmsStatus,
    pub timestamp: DateTime<Utc>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

/// Twilio SMS provider with in-memory message store and delivery tracking.
pub struct SmsProvider {
    config: TwilioConfig,
    messages: DashMap<Uuid, SmsMessage>,
    /// Maps provider_id -> message Uuid for webhook lookups.
    provider_index: DashMap<String, Uuid>,
    /// Delivery events keyed by recipient phone number.
    delivery_events: DashMap<String, Vec<SmsDeliveryEvent>>,
}

impl SmsProvider {
    /// Create a new SMS provider with the given Twilio configuration.
    pub fn new(config: TwilioConfig) -> Self {
        tracing::info!(
            account_sid = %config.account_sid,
            from = %config.from_number,
            "Twilio SMS provider initialized"
        );
        Self {
            config,
            messages: DashMap::new(),
            provider_index: DashMap::new(),
            delivery_events: DashMap::new(),
        }
    }

    /// Send an SMS message. Simulates the Twilio API call and returns the
    /// created message with a generated provider_id.
    pub fn send(&self, to: &str, body: &str, media_url: Option<String>) -> SmsMessage {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let provider_id = format!("SM{}", Uuid::new_v4().to_string().replace('-', ""));
        let segments = Self::calculate_segments(body);

        let msg = SmsMessage {
            id,
            to: to.to_string(),
            from: self.config.from_number.clone(),
            body: body.to_string(),
            media_url,
            status: SmsStatus::Queued,
            provider_id: Some(provider_id.clone()),
            created_at: now,
            updated_at: now,
            segments,
        };

        tracing::info!(
            id = %id,
            to = %to,
            provider_id = %provider_id,
            segments = segments,
            "SMS message queued"
        );

        metrics::counter!("sms.messages_sent").increment(1);

        self.messages.insert(id, msg.clone());
        self.provider_index.insert(provider_id, id);

        msg
    }

    /// Retrieve a message by its internal ID.
    pub fn get_message(&self, id: Uuid) -> Option<SmsMessage> {
        self.messages.get(&id).map(|m| m.clone())
    }

    /// Handle a Twilio status callback webhook. Looks up the message by
    /// provider_id, updates its status, and records a delivery event.
    /// Returns true if the message was found and updated.
    pub fn handle_status_callback(
        &self,
        provider_id: &str,
        status: &str,
        error_code: Option<&str>,
    ) -> bool {
        let message_id = match self.provider_index.get(provider_id) {
            Some(entry) => *entry.value(),
            None => {
                tracing::warn!(provider_id = %provider_id, "Status callback for unknown provider_id");
                return false;
            }
        };

        let new_status = match status {
            "queued" => SmsStatus::Queued,
            "sent" => SmsStatus::Sent,
            "delivered" => SmsStatus::Delivered,
            "failed" => SmsStatus::Failed,
            "undelivered" => SmsStatus::Undelivered,
            other => {
                tracing::warn!(status = %other, "Unknown SMS status in callback");
                return false;
            }
        };

        let now = Utc::now();
        let mut to_number = String::new();

        if let Some(mut msg) = self.messages.get_mut(&message_id) {
            msg.status = new_status.clone();
            msg.updated_at = now;
            to_number.clone_from(&msg.to);
        } else {
            return false;
        }

        let error_message = error_code.map(|code| format!("Twilio error: {}", code));

        let event = SmsDeliveryEvent {
            message_id,
            status: new_status,
            timestamp: now,
            error_code: error_code.map(|c| c.to_string()),
            error_message,
        };

        self.delivery_events
            .entry(to_number)
            .or_default()
            .push(event);

        tracing::debug!(
            provider_id = %provider_id,
            status = %status,
            "SMS status callback processed"
        );

        metrics::counter!(
            "sms.status_callbacks",
            "status" => status.to_string()
        )
        .increment(1);

        true
    }

    /// Calculate the number of SMS segments for a message body.
    /// GSM 7-bit encoding: 160 chars per segment.
    /// Unicode (UCS-2): 70 chars per segment.
    pub fn calculate_segments(body: &str) -> u32 {
        if body.is_empty() {
            return 1;
        }

        let is_gsm = body.chars().all(is_gsm_7bit);
        let char_count = body.chars().count() as u32;

        if is_gsm {
            // GSM: 160 chars for single segment, 153 for multi-segment (UDH overhead)
            if char_count <= 160 {
                1
            } else {
                char_count.div_ceil(153)
            }
        } else {
            // Unicode: 70 chars for single segment, 67 for multi-segment
            if char_count <= 70 {
                1
            } else {
                char_count.div_ceil(67)
            }
        }
    }

    /// List messages, returning up to `limit` most recently created messages.
    pub fn list_messages(&self, limit: usize) -> Vec<SmsMessage> {
        let mut messages: Vec<SmsMessage> = self
            .messages
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        messages.truncate(limit);
        messages
    }

    /// Get all delivery events for a given recipient phone number.
    pub fn get_delivery_events(&self, to: &str) -> Vec<SmsDeliveryEvent> {
        self.delivery_events
            .get(to)
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    /// Send multiple SMS messages in bulk. Returns the list of created messages.
    pub fn send_bulk(&self, messages: Vec<(&str, &str)>) -> Vec<SmsMessage> {
        messages
            .into_iter()
            .map(|(to, body)| self.send(to, body, None))
            .collect()
    }

    /// Get a reference to the provider configuration.
    pub fn config(&self) -> &TwilioConfig {
        &self.config
    }
}

/// Check whether a character is in the GSM 7-bit default alphabet.
fn is_gsm_7bit(c: char) -> bool {
    matches!(c,
        'A'..='Z' | 'a'..='z' | '0'..='9'
        | ' ' | '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')'
        | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '='
        | '>' | '?' | '@' | '_' | '\n' | '\r'
        | '\u{00A3}' // Pound sign
        | '\u{00A5}' // Yen sign
        | '\u{00E8}' // e-grave
        | '\u{00E9}' // e-acute
        | '\u{00F9}' // u-grave
        | '\u{00EC}' // i-grave
        | '\u{00F2}' // o-grave
        | '\u{00C7}' // C-cedilla
        | '\u{00D8}' // O-stroke
        | '\u{00F8}' // o-stroke
        | '\u{00C5}' // A-ring
        | '\u{00E5}' // a-ring
        | '\u{0394}' // Greek Delta
        | '\u{03A6}' // Greek Phi
        | '\u{0393}' // Greek Gamma
        | '\u{039B}' // Greek Lambda
        | '\u{03A9}' // Greek Omega
        | '\u{03A0}' // Greek Pi
        | '\u{03A8}' // Greek Psi
        | '\u{03A3}' // Greek Sigma
        | '\u{0398}' // Greek Theta
        | '\u{039E}' // Greek Xi
        | '\u{00C6}' // AE ligature
        | '\u{00E6}' // ae ligature
        | '\u{00DF}' // Sharp s
        | '\u{00C9}' // E-acute
        | '\u{00A4}' // Currency sign
        | '\u{00A1}' // Inverted exclamation
        | '\u{00BF}' // Inverted question
        | '\u{00C4}' // A-umlaut
        | '\u{00D6}' // O-umlaut
        | '\u{00D1}' // N-tilde
        | '\u{00DC}' // U-umlaut
        | '\u{00A7}' // Section sign
        | '\u{00E4}' // a-umlaut
        | '\u{00F6}' // o-umlaut
        | '\u{00F1}' // n-tilde
        | '\u{00FC}' // u-umlaut
        | '\u{00E0}' // a-grave
        // GSM extension characters (counted as 2 but still GSM)
        | '{' | '}' | '[' | ']' | '~' | '\\' | '^' | '|' | '\u{20AC}' // Euro sign
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TwilioConfig {
        TwilioConfig {
            account_sid: "AC_test_sid".to_string(),
            auth_token: "test_auth_token".to_string(),
            from_number: "+15551234567".to_string(),
            messaging_service_sid: None,
            status_callback_url: Some("https://example.com/callback".to_string()),
        }
    }

    #[test]
    fn test_send_and_retrieve() {
        let provider = SmsProvider::new(test_config());
        let msg = provider.send("+15559876543", "Hello, World!", None);

        assert_eq!(msg.to, "+15559876543");
        assert_eq!(msg.from, "+15551234567");
        assert_eq!(msg.body, "Hello, World!");
        assert_eq!(msg.status, SmsStatus::Queued);
        assert!(msg.provider_id.is_some());
        assert_eq!(msg.segments, 1);

        // Retrieve the message by ID
        let retrieved = provider.get_message(msg.id).unwrap();
        assert_eq!(retrieved.id, msg.id);
        assert_eq!(retrieved.body, "Hello, World!");
    }

    #[test]
    fn test_status_callback() {
        let provider = SmsProvider::new(test_config());
        let msg = provider.send("+15559876543", "Test callback", None);
        let provider_id = msg.provider_id.as_ref().unwrap().clone();

        // Simulate delivery callback
        let updated = provider.handle_status_callback(&provider_id, "delivered", None);
        assert!(updated);

        let retrieved = provider.get_message(msg.id).unwrap();
        assert_eq!(retrieved.status, SmsStatus::Delivered);

        // Check delivery events
        let events = provider.get_delivery_events("+15559876543");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, SmsStatus::Delivered);
        assert!(events[0].error_code.is_none());
    }

    #[test]
    fn test_status_callback_with_error() {
        let provider = SmsProvider::new(test_config());
        let msg = provider.send("+15559876543", "Test failure", None);
        let provider_id = msg.provider_id.as_ref().unwrap().clone();

        let updated = provider.handle_status_callback(&provider_id, "failed", Some("30006"));
        assert!(updated);

        let retrieved = provider.get_message(msg.id).unwrap();
        assert_eq!(retrieved.status, SmsStatus::Failed);

        let events = provider.get_delivery_events("+15559876543");
        assert_eq!(events[0].error_code, Some("30006".to_string()));
    }

    #[test]
    fn test_callback_unknown_provider_id() {
        let provider = SmsProvider::new(test_config());
        let result = provider.handle_status_callback("SM_nonexistent", "delivered", None);
        assert!(!result);
    }

    #[test]
    fn test_calculate_segments_gsm_short() {
        // Short GSM message: 1 segment
        let body = "Hello";
        assert_eq!(SmsProvider::calculate_segments(body), 1);
    }

    #[test]
    fn test_calculate_segments_gsm_exactly_160() {
        let body = "A".repeat(160);
        assert_eq!(SmsProvider::calculate_segments(&body), 1);
    }

    #[test]
    fn test_calculate_segments_gsm_multi() {
        // 161 GSM chars -> 2 segments (153 + 8)
        let body = "A".repeat(161);
        assert_eq!(SmsProvider::calculate_segments(&body), 2);

        // 306 chars -> 2 segments (2 * 153 = 306)
        let body = "B".repeat(306);
        assert_eq!(SmsProvider::calculate_segments(&body), 2);

        // 307 chars -> 3 segments
        let body = "C".repeat(307);
        assert_eq!(SmsProvider::calculate_segments(&body), 3);
    }

    #[test]
    fn test_calculate_segments_unicode_short() {
        // Unicode message with emoji: 1 segment if <= 70 chars
        let body = "\u{1F600}".repeat(10); // 10 emoji
        assert_eq!(SmsProvider::calculate_segments(&body), 1);
    }

    #[test]
    fn test_calculate_segments_unicode_exactly_70() {
        // 69 ascii + 1 emoji = 70 chars with unicode -> 1 segment
        let body = format!("{}\u{1F600}", "A".repeat(69));
        assert_eq!(SmsProvider::calculate_segments(&body), 1);
    }

    #[test]
    fn test_calculate_segments_unicode_multi() {
        // 71 unicode chars -> 2 segments (67 + 4)
        let body = format!("{}\u{1F600}", "A".repeat(70));
        // 71 chars, has emoji => Unicode => ceil(71/67) = 2
        assert_eq!(SmsProvider::calculate_segments(&body), 2);
    }

    #[test]
    fn test_calculate_segments_empty() {
        assert_eq!(SmsProvider::calculate_segments(""), 1);
    }

    #[test]
    fn test_list_messages() {
        let provider = SmsProvider::new(test_config());
        provider.send("+15551111111", "Message 1", None);
        provider.send("+15552222222", "Message 2", None);
        provider.send("+15553333333", "Message 3", None);

        let msgs = provider.list_messages(2);
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn test_send_bulk() {
        let provider = SmsProvider::new(test_config());
        let messages = vec![
            ("+15551111111", "Bulk msg 1"),
            ("+15552222222", "Bulk msg 2"),
            ("+15553333333", "Bulk msg 3"),
        ];

        let results = provider.send_bulk(messages);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].to, "+15551111111");
        assert_eq!(results[1].to, "+15552222222");
        assert_eq!(results[2].to, "+15553333333");

        // All should be in the message store
        for msg in &results {
            assert!(provider.get_message(msg.id).is_some());
        }
    }

    #[test]
    fn test_send_with_media() {
        let provider = SmsProvider::new(test_config());
        let msg = provider.send(
            "+15559876543",
            "Check this out!",
            Some("https://example.com/image.jpg".to_string()),
        );
        assert_eq!(
            msg.media_url,
            Some("https://example.com/image.jpg".to_string())
        );
    }
}
