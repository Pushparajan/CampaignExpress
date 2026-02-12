//! WhatsApp Business API integration for conversational messaging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppMessageType {
    Template,
    Text,
    Image,
    Video,
    Document,
    Interactive,
    Location,
    Sticker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppTemplate {
    pub name: String,
    pub language: String,
    pub category: TemplateCategory,
    pub components: Vec<TemplateComponent>,
    pub status: TemplateStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Marketing,
    Utility,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateComponent {
    pub component_type: String,
    pub parameters: Vec<TemplateParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub param_type: String,
    pub text: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub to_phone: String,
    pub message_type: WhatsAppMessageType,
    pub template: Option<WhatsAppTemplate>,
    pub body: Option<String>,
    pub media_url: Option<String>,
    pub interactive: Option<InteractiveMessage>,
    pub reply_to: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveMessage {
    pub interactive_type: String,
    pub header: Option<String>,
    pub body: String,
    pub footer: Option<String>,
    pub buttons: Vec<InteractiveButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveButton {
    pub button_type: String,
    pub title: String,
    pub payload: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Queued,
    Sent,
    Delivered,
    Read,
    Failed,
}

pub struct WhatsAppProvider {
    api_base_url: String,
    access_token: String,
    phone_number_id: String,
}

impl WhatsAppProvider {
    pub fn new(api_base_url: String, access_token: String, phone_number_id: String) -> Self {
        Self {
            api_base_url,
            access_token,
            phone_number_id,
        }
    }

    pub async fn send_template_message(
        &self,
        to: &str,
        template: &WhatsAppTemplate,
    ) -> anyhow::Result<String> {
        tracing::info!(
            to = to,
            template = &template.name,
            phone_id = &self.phone_number_id,
            base = &self.api_base_url,
            token_len = self.access_token.len(),
            "Sending WhatsApp template message"
        );
        Ok(Uuid::new_v4().to_string())
    }

    pub async fn send_text_message(&self, to: &str, body: &str) -> anyhow::Result<String> {
        tracing::info!(
            to = to,
            body_len = body.len(),
            "Sending WhatsApp text message"
        );
        Ok(Uuid::new_v4().to_string())
    }
}
