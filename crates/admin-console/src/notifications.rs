//! Notification system — in-app alerts, email triggers, webhook dispatches
//! for SaaS operational events (billing, quota, incidents, etc.).

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// Notification severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Notification delivery channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    InApp,
    Email,
    Webhook,
    Slack,
}

/// Event category that triggered the notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    Billing,
    QuotaWarning,
    SecurityAlert,
    Incident,
    TenantLifecycle,
    SystemUpdate,
    Compliance,
}

/// A single notification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub category: NotificationCategory,
    pub severity: Severity,
    pub title: String,
    pub message: String,
    pub channels: Vec<NotificationChannel>,
    pub read: bool,
    pub acknowledged: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

/// Webhook endpoint registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub url: String,
    pub events: Vec<NotificationCategory>,
    pub secret: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub failure_count: u32,
}

/// Notification manager for the provider admin console.
pub struct NotificationManager {
    notifications: DashMap<Uuid, Notification>,
    webhooks: DashMap<Uuid, WebhookEndpoint>,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: DashMap::new(),
            webhooks: DashMap::new(),
        }
    }

    /// Create and store a notification.
    pub fn notify(
        &self,
        tenant_id: Option<Uuid>,
        category: NotificationCategory,
        severity: Severity,
        title: impl Into<String>,
        message: impl Into<String>,
        channels: Vec<NotificationChannel>,
    ) -> Notification {
        let notification = Notification {
            id: Uuid::new_v4(),
            tenant_id,
            category,
            severity,
            title: title.into(),
            message: message.into(),
            channels,
            read: false,
            acknowledged: false,
            created_at: Utc::now(),
            read_at: None,
        };
        info!(
            notification_id = %notification.id,
            category = ?category,
            severity = ?severity,
            "Notification created"
        );
        self.notifications
            .insert(notification.id, notification.clone());
        notification
    }

    /// Mark a notification as read.
    pub fn mark_read(&self, notification_id: Uuid) -> bool {
        if let Some(mut entry) = self.notifications.get_mut(&notification_id) {
            entry.read = true;
            entry.read_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    /// Acknowledge a notification (dismiss from active alerts).
    pub fn acknowledge(&self, notification_id: Uuid) -> bool {
        if let Some(mut entry) = self.notifications.get_mut(&notification_id) {
            entry.acknowledged = true;
            true
        } else {
            false
        }
    }

    /// List all unread notifications, optionally filtered by tenant.
    pub fn unread(&self, tenant_id: Option<Uuid>) -> Vec<Notification> {
        let mut results: Vec<_> = self
            .notifications
            .iter()
            .filter(|e| {
                let n = e.value();
                !n.read
                    && (tenant_id.is_none() || n.tenant_id.is_none() || n.tenant_id == tenant_id)
            })
            .map(|e| e.value().clone())
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results
    }

    /// List notifications by severity (for escalation views).
    pub fn by_severity(&self, severity: Severity) -> Vec<Notification> {
        let mut results: Vec<_> = self
            .notifications
            .iter()
            .filter(|e| e.value().severity == severity)
            .map(|e| e.value().clone())
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results
    }

    /// Register a webhook endpoint for a tenant.
    pub fn register_webhook(
        &self,
        tenant_id: Uuid,
        url: String,
        events: Vec<NotificationCategory>,
        secret: String,
    ) -> WebhookEndpoint {
        let endpoint = WebhookEndpoint {
            id: Uuid::new_v4(),
            tenant_id,
            url: url.clone(),
            events,
            secret,
            enabled: true,
            created_at: Utc::now(),
            last_triggered: None,
            failure_count: 0,
        };
        info!(webhook_id = %endpoint.id, url = %url, "Webhook registered");
        self.webhooks.insert(endpoint.id, endpoint.clone());
        endpoint
    }

    /// List webhook endpoints for a tenant.
    pub fn list_webhooks(&self, tenant_id: Uuid) -> Vec<WebhookEndpoint> {
        self.webhooks
            .iter()
            .filter(|e| e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Disable a webhook endpoint.
    pub fn disable_webhook(&self, webhook_id: Uuid) -> bool {
        if let Some(mut entry) = self.webhooks.get_mut(&webhook_id) {
            entry.enabled = false;
            true
        } else {
            false
        }
    }

    /// Seed demo notifications.
    pub fn seed_demo(&self) {
        self.notify(
            None,
            NotificationCategory::SystemUpdate,
            Severity::Info,
            "Platform v0.1.0 deployed",
            "Campaign Express v0.1.0 has been deployed across all nodes.",
            vec![NotificationChannel::InApp],
        );
        self.notify(
            Some(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()),
            NotificationCategory::Billing,
            Severity::Warning,
            "Invoice overdue",
            "Invoice INV-2024-001 for Acme Corp is 15 days overdue ($501.50).",
            vec![NotificationChannel::InApp, NotificationChannel::Email],
        );
        self.notify(
            None,
            NotificationCategory::QuotaWarning,
            Severity::Warning,
            "Cluster usage at 85%",
            "NPU cluster usage has reached 85% capacity. Consider scaling.",
            vec![NotificationChannel::InApp, NotificationChannel::Slack],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_and_read() {
        let mgr = NotificationManager::new();
        let n = mgr.notify(
            None,
            NotificationCategory::SystemUpdate,
            Severity::Info,
            "Test",
            "Test message",
            vec![NotificationChannel::InApp],
        );
        assert!(!n.read);
        assert_eq!(mgr.unread(None).len(), 1);

        mgr.mark_read(n.id);
        assert_eq!(mgr.unread(None).len(), 0);
    }

    #[test]
    fn test_severity_filter() {
        let mgr = NotificationManager::new();
        mgr.notify(
            None,
            NotificationCategory::Incident,
            Severity::Critical,
            "Down",
            "System down",
            vec![NotificationChannel::InApp],
        );
        mgr.notify(
            None,
            NotificationCategory::SystemUpdate,
            Severity::Info,
            "Up",
            "System up",
            vec![NotificationChannel::InApp],
        );

        assert_eq!(mgr.by_severity(Severity::Critical).len(), 1);
        assert_eq!(mgr.by_severity(Severity::Info).len(), 1);
        assert_eq!(mgr.by_severity(Severity::Warning).len(), 0);
    }

    #[test]
    fn test_webhook_lifecycle() {
        let mgr = NotificationManager::new();
        let tenant_id = Uuid::new_v4();

        let wh = mgr.register_webhook(
            tenant_id,
            "https://example.com/webhook".into(),
            vec![NotificationCategory::Billing],
            "secret123".into(),
        );
        assert!(wh.enabled);
        assert_eq!(mgr.list_webhooks(tenant_id).len(), 1);

        mgr.disable_webhook(wh.id);
        let hooks = mgr.list_webhooks(tenant_id);
        assert!(!hooks[0].enabled);
    }
}
