//! Activation dispatcher — delivers personalized offers/messages to users
//! across multiple output channels (push, SMS, email, paid media, in-store).

use campaign_core::channels::*;
use chrono::Utc;
use tracing::{debug, info};
use uuid::Uuid;

/// Dispatches activation messages to the appropriate output channel.
pub struct ActivationDispatcher {
    enabled_channels: Vec<ActivationChannel>,
}

impl ActivationDispatcher {
    pub fn new(channels: Vec<ActivationChannel>) -> Self {
        info!(
            channels = ?channels,
            "Activation dispatcher initialized"
        );
        Self {
            enabled_channels: channels,
        }
    }

    /// Dispatch an activation to the target channel.
    pub async fn dispatch(&self, request: &ActivationRequest) -> ActivationResult {
        if !self.enabled_channels.contains(&request.channel) {
            return ActivationResult {
                activation_id: request.activation_id.clone(),
                channel: request.channel,
                status: ActivationStatus::Failed,
                provider_message_id: None,
                latency_ms: 0,
                error: Some(format!("Channel {:?} not enabled", request.channel)),
                delivered_at: None,
            };
        }

        let start = std::time::Instant::now();

        metrics::counter!(
            "activation.dispatched",
            "channel" => request.channel.display_name()
        )
        .increment(1);

        let result = match request.channel {
            ActivationChannel::PushNotification => self.send_push(request).await,
            ActivationChannel::Sms => self.send_sms(request).await,
            ActivationChannel::Email => self.send_email(request).await,
            ActivationChannel::InAppMessage => self.send_in_app(request).await,
            ActivationChannel::WebPersonalization => self.send_web(request).await,
            ActivationChannel::PaidMediaFacebook
            | ActivationChannel::PaidMediaTradeDesk
            | ActivationChannel::PaidMediaGoogle
            | ActivationChannel::PaidMediaAmazon => self.send_paid_media(request).await,
            ActivationChannel::DigitalSignage | ActivationChannel::KioskDisplay => {
                self.send_in_store(request).await
            }
        };

        let latency_ms = start.elapsed().as_millis() as u64;
        metrics::histogram!(
            "activation.latency_ms",
            "channel" => request.channel.display_name()
        )
        .record(latency_ms as f64);

        ActivationResult {
            latency_ms,
            ..result
        }
    }

    /// Select the best activation channel for a user based on context.
    pub fn select_channel(
        &self,
        preferred: Option<ActivationChannel>,
        is_in_store: bool,
        has_push_token: bool,
        has_phone: bool,
        has_email: bool,
    ) -> Option<ActivationChannel> {
        // Priority: in-store > push > in-app > SMS > email > paid media
        if is_in_store {
            if self
                .enabled_channels
                .contains(&ActivationChannel::KioskDisplay)
            {
                return Some(ActivationChannel::KioskDisplay);
            }
            if self
                .enabled_channels
                .contains(&ActivationChannel::DigitalSignage)
            {
                return Some(ActivationChannel::DigitalSignage);
            }
        }

        if let Some(pref) = preferred {
            if self.enabled_channels.contains(&pref) {
                return Some(pref);
            }
        }

        if has_push_token
            && self
                .enabled_channels
                .contains(&ActivationChannel::PushNotification)
        {
            return Some(ActivationChannel::PushNotification);
        }
        if self
            .enabled_channels
            .contains(&ActivationChannel::InAppMessage)
        {
            return Some(ActivationChannel::InAppMessage);
        }
        if has_phone && self.enabled_channels.contains(&ActivationChannel::Sms) {
            return Some(ActivationChannel::Sms);
        }
        if has_email && self.enabled_channels.contains(&ActivationChannel::Email) {
            return Some(ActivationChannel::Email);
        }
        if self
            .enabled_channels
            .contains(&ActivationChannel::WebPersonalization)
        {
            return Some(ActivationChannel::WebPersonalization);
        }

        None
    }

    // ─── Channel-specific senders (stubs for production integration) ────────

    async fn send_push(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: call Firebase Cloud Messaging / APNs
        debug!(user_id = %req.user_id, "Sending push notification");
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: ActivationChannel::PushNotification,
            status: ActivationStatus::Sent,
            provider_message_id: Some(Uuid::new_v4().to_string()),
            latency_ms: 0,
            error: None,
            delivered_at: Some(Utc::now()),
        }
    }

    async fn send_sms(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: call Twilio / AWS SNS
        debug!(user_id = %req.user_id, "Sending SMS");
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: ActivationChannel::Sms,
            status: ActivationStatus::Sent,
            provider_message_id: Some(Uuid::new_v4().to_string()),
            latency_ms: 0,
            error: None,
            delivered_at: Some(Utc::now()),
        }
    }

    async fn send_email(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: call SendGrid / AWS SES
        debug!(user_id = %req.user_id, "Sending email");
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: ActivationChannel::Email,
            status: ActivationStatus::Queued,
            provider_message_id: Some(Uuid::new_v4().to_string()),
            latency_ms: 0,
            error: None,
            delivered_at: None,
        }
    }

    async fn send_in_app(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: publish to user's in-app message queue
        debug!(user_id = %req.user_id, "Sending in-app message");
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: ActivationChannel::InAppMessage,
            status: ActivationStatus::Delivered,
            provider_message_id: Some(Uuid::new_v4().to_string()),
            latency_ms: 0,
            error: None,
            delivered_at: Some(Utc::now()),
        }
    }

    async fn send_web(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: push to web personalization engine
        debug!(user_id = %req.user_id, "Sending web personalization");
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: ActivationChannel::WebPersonalization,
            status: ActivationStatus::Delivered,
            provider_message_id: None,
            latency_ms: 0,
            error: None,
            delivered_at: Some(Utc::now()),
        }
    }

    async fn send_paid_media(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: add user to DSP audience segment via API
        debug!(
            user_id = %req.user_id,
            channel = ?req.channel,
            "Adding to paid media audience"
        );
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: req.channel,
            status: ActivationStatus::Queued,
            provider_message_id: Some(Uuid::new_v4().to_string()),
            latency_ms: 0,
            error: None,
            delivered_at: None,
        }
    }

    async fn send_in_store(&self, req: &ActivationRequest) -> ActivationResult {
        // In production: push to in-store display management system
        debug!(
            user_id = %req.user_id,
            channel = ?req.channel,
            "Sending to in-store display"
        );
        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: req.channel,
            status: ActivationStatus::Delivered,
            provider_message_id: None,
            latency_ms: 0,
            error: None,
            delivered_at: Some(Utc::now()),
        }
    }
}
