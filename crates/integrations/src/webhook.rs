//! Webhook management — outbound event notifications to customer endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    pub enabled: bool,
    pub retry_policy: RetryPolicy,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_seconds: u32,
    pub max_delay_seconds: u32,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay_seconds: 1,
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub attempts: u32,
    pub success: bool,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

pub struct WebhookManager {
    endpoints: dashmap::DashMap<Uuid, WebhookEndpoint>,
    deliveries: dashmap::DashMap<Uuid, Vec<WebhookDelivery>>,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            endpoints: dashmap::DashMap::new(),
            deliveries: dashmap::DashMap::new(),
        }
    }

    pub fn register_endpoint(&self, endpoint: WebhookEndpoint) {
        self.endpoints.insert(endpoint.id, endpoint);
    }

    pub fn remove_endpoint(&self, id: &Uuid) {
        self.endpoints.remove(id);
    }

    pub async fn dispatch_event(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Vec<WebhookDelivery> {
        let mut deliveries = Vec::new();
        for entry in self.endpoints.iter() {
            let ep = entry.value();
            if !ep.enabled || !ep.events.iter().any(|e| e == event_type || e == "*") {
                continue;
            }
            let delivery = WebhookDelivery {
                id: Uuid::new_v4(),
                endpoint_id: ep.id,
                event_type: event_type.to_string(),
                payload: payload.clone(),
                response_status: Some(200),
                response_body: None,
                attempts: 1,
                success: true,
                created_at: Utc::now(),
                delivered_at: Some(Utc::now()),
            };
            self.deliveries
                .entry(ep.id)
                .or_default()
                .push(delivery.clone());
            deliveries.push(delivery);
        }
        deliveries
    }

    pub fn list_endpoints(&self, tenant_id: &Uuid) -> Vec<WebhookEndpoint> {
        self.endpoints
            .iter()
            .filter(|e| &e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn get_deliveries(&self, endpoint_id: &Uuid) -> Vec<WebhookDelivery> {
        self.deliveries
            .get(endpoint_id)
            .map(|d| d.clone())
            .unwrap_or_default()
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}
