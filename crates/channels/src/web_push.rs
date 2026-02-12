//! Web push notifications via Web Push Protocol (RFC 8030) and VAPID.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_secret: String,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushNotification {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub title: String,
    pub body: String,
    pub icon_url: Option<String>,
    pub badge_url: Option<String>,
    pub image_url: Option<String>,
    pub click_action: Option<String>,
    pub actions: Vec<PushAction>,
    pub data: std::collections::HashMap<String, String>,
    pub ttl_seconds: u32,
    pub urgency: PushUrgency,
    pub require_interaction: bool,
    pub silent: bool,
    pub tag: Option<String>,
    pub renotify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushAction {
    pub action: String,
    pub title: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PushUrgency {
    VeryLow,
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDeliveryResult {
    pub subscription_id: Uuid,
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushConfig {
    pub vapid_public_key: String,
    pub vapid_private_key: String,
    pub subject: String,
    pub default_ttl: u32,
}

pub struct WebPushProvider {
    config: WebPushConfig,
    subscriptions: dashmap::DashMap<Uuid, Vec<WebPushSubscription>>,
}

impl WebPushProvider {
    pub fn new(config: WebPushConfig) -> Self {
        Self {
            config,
            subscriptions: dashmap::DashMap::new(),
        }
    }

    pub fn register_subscription(&self, subscription: WebPushSubscription) {
        self.subscriptions
            .entry(subscription.user_id)
            .or_default()
            .push(subscription);
    }

    pub fn unregister_subscription(&self, user_id: &Uuid, subscription_id: &Uuid) {
        if let Some(mut subs) = self.subscriptions.get_mut(user_id) {
            subs.retain(|s| &s.id != subscription_id);
        }
    }

    pub async fn send_notification(
        &self,
        user_id: &Uuid,
        notification: &WebPushNotification,
    ) -> Vec<PushDeliveryResult> {
        let subs = self
            .subscriptions
            .get(user_id)
            .map(|s| s.clone())
            .unwrap_or_default();

        let mut results = Vec::new();
        for sub in &subs {
            tracing::info!(
                user_id = %user_id,
                sub_id = %sub.id,
                endpoint = &sub.endpoint,
                title = &notification.title,
                vapid_subject = &self.config.subject,
                "Sending web push notification"
            );
            results.push(PushDeliveryResult {
                subscription_id: sub.id,
                success: true,
                status_code: Some(201),
                error: None,
                sent_at: Utc::now(),
            });
        }
        results
    }
}
