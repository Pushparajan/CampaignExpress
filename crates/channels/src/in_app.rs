//! In-app messaging — modal, slideup, fullscreen, and HTML custom messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InAppMessageType {
    Modal,
    Slideup,
    Fullscreen,
    HtmlCustom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InAppTrigger {
    SessionStart,
    CustomEvent(String),
    PurchaseCompleted,
    PushClick,
    ScreenView(String),
    ApiTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppMessage {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub message_type: InAppMessageType,
    pub trigger: InAppTrigger,
    pub header: Option<String>,
    pub body: String,
    pub image_url: Option<String>,
    pub buttons: Vec<InAppButton>,
    pub close_behavior: CloseBehavior,
    pub display_delay_ms: u64,
    pub priority: u8,
    pub max_impressions: Option<u32>,
    pub custom_html: Option<String>,
    pub css_override: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppButton {
    pub label: String,
    pub action: ButtonAction,
    pub style: ButtonStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonAction {
    DeepLink(String),
    Dismiss,
    CustomEvent(String),
    RequestPushPermission,
    OpenUrl(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Link,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseBehavior {
    AutoDismiss { after_seconds: u32 },
    ManualOnly,
    SwipeToDismiss,
}

pub struct InAppEngine {
    eligible_messages: dashmap::DashMap<Uuid, Vec<InAppMessage>>,
}

impl InAppEngine {
    pub fn new() -> Self {
        Self {
            eligible_messages: dashmap::DashMap::new(),
        }
    }

    pub fn register_message(&self, user_id: Uuid, message: InAppMessage) {
        self.eligible_messages
            .entry(user_id)
            .or_default()
            .push(message);
    }

    pub fn get_triggered_messages(
        &self,
        user_id: &Uuid,
        trigger: &InAppTrigger,
    ) -> Vec<InAppMessage> {
        self.eligible_messages
            .get(user_id)
            .map(|msgs| {
                msgs.iter()
                    .filter(|m| Self::trigger_matches(&m.trigger, trigger))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn trigger_matches(registered: &InAppTrigger, fired: &InAppTrigger) -> bool {
        matches!(
            (registered, fired),
            (InAppTrigger::SessionStart, InAppTrigger::SessionStart)
                | (
                    InAppTrigger::PurchaseCompleted,
                    InAppTrigger::PurchaseCompleted
                )
                | (InAppTrigger::PushClick, InAppTrigger::PushClick)
                | (InAppTrigger::ApiTriggered, InAppTrigger::ApiTriggered)
                | (InAppTrigger::CustomEvent(_), InAppTrigger::CustomEvent(_))
                | (InAppTrigger::ScreenView(_), InAppTrigger::ScreenView(_))
        )
    }
}

impl Default for InAppEngine {
    fn default() -> Self {
        Self::new()
    }
}
