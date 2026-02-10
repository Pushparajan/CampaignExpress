//! Omnichannel ingest and activation types.
//!
//! Ingest sources: real-time event queues from mobile apps, POS terminals,
//! kiosks, web, and partner systems.
//!
//! Activation destinations: push notifications, SMS, email, web personalization,
//! paid media (Facebook, The Trade Desk, etc.), in-app messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Ingest Sources ─────────────────────────────────────────────────────────

/// Source channels that feed real-time events into the platform.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IngestSource {
    MobileApp,
    Pos,
    Kiosk,
    Web,
    CallCenter,
    PartnerApi,
    IoTDevice,
}

impl IngestSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            IngestSource::MobileApp => "Mobile App",
            IngestSource::Pos => "Point of Sale",
            IngestSource::Kiosk => "In-Store Kiosk",
            IngestSource::Web => "Website",
            IngestSource::CallCenter => "Call Center",
            IngestSource::PartnerApi => "Partner API",
            IngestSource::IoTDevice => "IoT Device",
        }
    }

    /// Priority weighting for deduplication (higher = preferred).
    pub fn priority(&self) -> u8 {
        match self {
            IngestSource::Pos => 10,       // Ground truth
            IngestSource::MobileApp => 8,
            IngestSource::Kiosk => 7,
            IngestSource::Web => 6,
            IngestSource::CallCenter => 5,
            IngestSource::PartnerApi => 4,
            IngestSource::IoTDevice => 3,
        }
    }
}

/// A real-time event ingested from any source channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEvent {
    pub event_id: String,
    pub source: IngestSource,
    pub event_type: IngestEventType,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub session_id: Option<String>,
    pub payload: serde_json::Value,
    pub location: Option<GeoLocation>,
    pub occurred_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestEventType {
    Purchase,
    ProductView,
    CartAdd,
    CartAbandon,
    AppOpen,
    PageView,
    Search,
    WishlistAdd,
    StoreVisit,
    LoyaltySwipe,
    CheckIn,
    Feedback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub lat: f64,
    pub lon: f64,
    pub accuracy_m: Option<f32>,
}

// ─── Activation Destinations ────────────────────────────────────────────────

/// Output channels for delivering personalized offers/messages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActivationChannel {
    PushNotification,
    Sms,
    Email,
    InAppMessage,
    WebPersonalization,
    PaidMediaFacebook,
    PaidMediaTradeDesk,
    PaidMediaGoogle,
    PaidMediaAmazon,
    DigitalSignage,
    KioskDisplay,
}

impl ActivationChannel {
    pub fn display_name(&self) -> &'static str {
        match self {
            ActivationChannel::PushNotification => "Push Notification",
            ActivationChannel::Sms => "SMS",
            ActivationChannel::Email => "Email",
            ActivationChannel::InAppMessage => "In-App Message",
            ActivationChannel::WebPersonalization => "Web Personalization",
            ActivationChannel::PaidMediaFacebook => "Facebook/Meta Ads",
            ActivationChannel::PaidMediaTradeDesk => "The Trade Desk",
            ActivationChannel::PaidMediaGoogle => "Google DV360",
            ActivationChannel::PaidMediaAmazon => "Amazon DSP",
            ActivationChannel::DigitalSignage => "In-Store Digital Signage",
            ActivationChannel::KioskDisplay => "Kiosk Display",
        }
    }

    /// Whether this channel is a paid media channel (requires DSP integration).
    pub fn is_paid_media(&self) -> bool {
        matches!(
            self,
            ActivationChannel::PaidMediaFacebook
                | ActivationChannel::PaidMediaTradeDesk
                | ActivationChannel::PaidMediaGoogle
                | ActivationChannel::PaidMediaAmazon
        )
    }

    /// Typical delivery latency in milliseconds.
    pub fn expected_latency_ms(&self) -> u64 {
        match self {
            ActivationChannel::WebPersonalization => 50,
            ActivationChannel::InAppMessage => 100,
            ActivationChannel::KioskDisplay => 100,
            ActivationChannel::DigitalSignage => 200,
            ActivationChannel::PushNotification => 500,
            ActivationChannel::Sms => 2000,
            ActivationChannel::Email => 5000,
            ActivationChannel::PaidMediaFacebook
            | ActivationChannel::PaidMediaTradeDesk
            | ActivationChannel::PaidMediaGoogle
            | ActivationChannel::PaidMediaAmazon => 200,
        }
    }
}

/// A message/offer to be delivered to a user via an activation channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRequest {
    pub activation_id: String,
    pub user_id: String,
    pub channel: ActivationChannel,
    pub offer_id: String,
    pub content: ActivationContent,
    /// Priority (1=highest, 10=lowest). Used for throttling.
    pub priority: u8,
    /// Schedule for future delivery, or None for immediate.
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Originating ingest event that triggered this activation.
    pub trigger_event_id: Option<String>,
    pub trigger_source: Option<IngestSource>,
}

/// Content payload for an activation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationContent {
    pub headline: String,
    pub body: String,
    pub image_url: Option<String>,
    pub cta_url: Option<String>,
    pub cta_text: Option<String>,
    /// Deep link for mobile push / in-app.
    pub deep_link: Option<String>,
    /// Paid media: audience segment to target.
    pub audience_segment_id: Option<String>,
    /// Extra platform-specific payload.
    pub extra: Option<serde_json::Value>,
}

/// Result of an activation attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub activation_id: String,
    pub channel: ActivationChannel,
    pub status: ActivationStatus,
    pub provider_message_id: Option<String>,
    pub latency_ms: u64,
    pub error: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivationStatus {
    Queued,
    Sent,
    Delivered,
    Opened,
    Clicked,
    Failed,
    Bounced,
    Throttled,
    OptedIn,
    Unsubscribed,
}

// ─── SendGrid Email Analytics ──────────────────────────────────────────────

/// SendGrid webhook event types for email delivery analytics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmailEventType {
    Processed,
    Dropped,
    Delivered,
    Deferred,
    Bounce,
    Open,
    Click,
    SpamReport,
    Unsubscribe,
    GroupUnsubscribe,
    GroupResubscribe,
}

/// A SendGrid webhook event for email delivery analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailWebhookEvent {
    pub email: String,
    pub event: EmailEventType,
    pub sg_message_id: Option<String>,
    pub activation_id: Option<String>,
    pub url: Option<String>,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Aggregated email analytics for a campaign or activation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailAnalytics {
    pub activation_id: String,
    pub total_sent: u64,
    pub delivered: u64,
    pub opens: u64,
    pub unique_opens: u64,
    pub clicks: u64,
    pub unique_clicks: u64,
    pub bounces: u64,
    pub spam_reports: u64,
    pub unsubscribes: u64,
    pub open_rate: f64,
    pub click_rate: f64,
    pub bounce_rate: f64,
}

/// SendGrid email provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendGridConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: String,
    pub webhook_url: String,
    pub tracking_enabled: bool,
    pub open_tracking: bool,
    pub click_tracking: bool,
    pub unsubscribe_group_id: Option<u64>,
}

impl Default for SendGridConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            from_email: "offers@campaignexpress.io".to_string(),
            from_name: "Campaign Express".to_string(),
            webhook_url: "https://api.campaignexpress.io/v1/webhooks/sendgrid".to_string(),
            tracking_enabled: true,
            open_tracking: true,
            click_tracking: true,
            unsubscribe_group_id: None,
        }
    }
}
