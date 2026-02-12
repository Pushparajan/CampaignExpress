//! SendGrid email activation with delivery analytics.
//!
//! Handles email sending via SendGrid API and processes inbound webhook
//! events for tracking: delivered, opened, clicked, bounced, unsubscribed.

use campaign_core::channels::*;
use dashmap::DashMap;
use tracing::{debug, info, warn};

/// SendGrid email activation provider.
pub struct SendGridProvider {
    config: SendGridConfig,
    /// Track email analytics keyed by activation_id.
    analytics: DashMap<String, EmailAnalytics>,
    /// Track unique openers/clickers per activation.
    unique_opens: DashMap<String, std::collections::HashSet<String>>,
    unique_clicks: DashMap<String, std::collections::HashSet<String>>,
}

impl SendGridProvider {
    pub fn new(config: SendGridConfig) -> Self {
        info!(
            from = %config.from_email,
            tracking = config.tracking_enabled,
            "SendGrid provider initialized"
        );
        Self {
            config,
            analytics: DashMap::new(),
            unique_opens: DashMap::new(),
            unique_clicks: DashMap::new(),
        }
    }

    /// Send an email via SendGrid API.
    /// In production: POST to https://api.sendgrid.com/v3/mail/send
    pub async fn send_email(&self, req: &ActivationRequest, to_email: &str) -> ActivationResult {
        let start = std::time::Instant::now();

        debug!(
            user_id = %req.user_id,
            to = %to_email,
            subject = %req.content.headline,
            "Sending email via SendGrid"
        );

        metrics::counter!(
            "sendgrid.emails_sent",
            "activation_id" => req.activation_id.clone()
        )
        .increment(1);

        // Build SendGrid API payload (stub — in production, HTTP POST to SendGrid)
        let _payload = serde_json::json!({
            "personalizations": [{
                "to": [{"email": to_email}],
                "custom_args": {
                    "activation_id": req.activation_id,
                    "user_id": req.user_id,
                    "offer_id": req.offer_id
                }
            }],
            "from": {
                "email": self.config.from_email,
                "name": self.config.from_name
            },
            "subject": req.content.headline,
            "content": [{
                "type": "text/html",
                "value": req.content.body
            }],
            "tracking_settings": {
                "click_tracking": {"enable": self.config.click_tracking},
                "open_tracking": {"enable": self.config.open_tracking}
            }
        });

        // Initialize analytics for this activation
        self.analytics
            .entry(req.activation_id.clone())
            .or_insert_with(|| EmailAnalytics {
                activation_id: req.activation_id.clone(),
                total_sent: 0,
                ..Default::default()
            })
            .total_sent += 1;

        let latency_ms = start.elapsed().as_millis() as u64;
        let sg_message_id = format!("sg-{}", uuid::Uuid::new_v4());

        ActivationResult {
            activation_id: req.activation_id.clone(),
            channel: ActivationChannel::Email,
            status: ActivationStatus::Queued,
            provider_message_id: Some(sg_message_id),
            latency_ms,
            error: None,
            delivered_at: None,
        }
    }

    /// Process a SendGrid webhook event and update analytics.
    pub fn process_webhook(&self, event: &EmailWebhookEvent) {
        let activation_id = match &event.activation_id {
            Some(id) => id.clone(),
            None => {
                warn!("SendGrid webhook missing activation_id, skipping");
                return;
            }
        };

        debug!(
            event_type = ?event.event,
            activation_id = %activation_id,
            email = %event.email,
            "Processing SendGrid webhook"
        );

        metrics::counter!(
            "sendgrid.webhook_events",
            "type" => format!("{:?}", event.event)
        )
        .increment(1);

        self.analytics
            .entry(activation_id.clone())
            .and_modify(|a| {
                match event.event {
                    EmailEventType::Delivered => {
                        a.delivered += 1;
                    }
                    EmailEventType::Open => {
                        a.opens += 1;
                        let mut unique =
                            self.unique_opens.entry(activation_id.clone()).or_default();
                        if unique.insert(event.email.clone()) {
                            a.unique_opens += 1;
                        }
                    }
                    EmailEventType::Click => {
                        a.clicks += 1;
                        let mut unique =
                            self.unique_clicks.entry(activation_id.clone()).or_default();
                        if unique.insert(event.email.clone()) {
                            a.unique_clicks += 1;
                        }
                    }
                    EmailEventType::Bounce => {
                        a.bounces += 1;
                    }
                    EmailEventType::SpamReport => {
                        a.spam_reports += 1;
                    }
                    EmailEventType::Unsubscribe | EmailEventType::GroupUnsubscribe => {
                        a.unsubscribes += 1;
                    }
                    _ => {}
                }

                // Recalculate rates
                if a.total_sent > 0 {
                    let sent = a.total_sent as f64;
                    a.open_rate = a.unique_opens as f64 / sent;
                    a.click_rate = a.unique_clicks as f64 / sent;
                    a.bounce_rate = a.bounces as f64 / sent;
                }
            })
            .or_insert_with(|| EmailAnalytics {
                activation_id: activation_id.clone(),
                ..Default::default()
            });
    }

    /// Get analytics for a specific activation.
    pub fn get_analytics(&self, activation_id: &str) -> Option<EmailAnalytics> {
        self.analytics.get(activation_id).map(|a| a.clone())
    }

    /// Get analytics for all activations.
    pub fn all_analytics(&self) -> Vec<EmailAnalytics> {
        self.analytics
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn config(&self) -> &SendGridConfig {
        &self.config
    }
}
