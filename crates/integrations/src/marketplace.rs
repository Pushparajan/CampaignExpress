//! Integration marketplace — catalog of available integrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationCategory {
    Cdp,
    Crm,
    DataWarehouse,
    Analytics,
    Advertising,
    ECommerce,
    Messaging,
    PaymentProvider,
    CloudStorage,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    OAuth2,
    BasicAuth,
    BearerToken,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationDefinition {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub category: IntegrationCategory,
    pub icon_url: Option<String>,
    pub auth_type: AuthType,
    pub config_schema: serde_json::Value,
    pub supported_actions: Vec<String>,
    pub supported_triggers: Vec<String>,
    pub documentation_url: Option<String>,
    pub is_premium: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledIntegration {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
    pub error_count: u32,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Active,
    Paused,
    Error,
    Syncing,
    NeverSynced,
}

pub struct IntegrationMarketplace {
    catalog: dashmap::DashMap<Uuid, IntegrationDefinition>,
    installed: dashmap::DashMap<Uuid, InstalledIntegration>,
}

impl IntegrationMarketplace {
    pub fn new() -> Self {
        let marketplace = Self {
            catalog: dashmap::DashMap::new(),
            installed: dashmap::DashMap::new(),
        };
        marketplace.seed_catalog();
        marketplace
    }

    fn seed_catalog(&self) {
        let integrations = vec![
            (
                "Segment",
                "segment",
                IntegrationCategory::Cdp,
                "Customer data platform for unified user profiles",
            ),
            (
                "Salesforce",
                "salesforce",
                IntegrationCategory::Crm,
                "CRM integration for lead and contact sync",
            ),
            (
                "Snowflake",
                "snowflake",
                IntegrationCategory::DataWarehouse,
                "Cloud data warehouse for analytics",
            ),
            (
                "BigQuery",
                "bigquery",
                IntegrationCategory::DataWarehouse,
                "Google BigQuery data warehouse",
            ),
            (
                "Mixpanel",
                "mixpanel",
                IntegrationCategory::Analytics,
                "Product analytics platform",
            ),
            (
                "Amplitude",
                "amplitude",
                IntegrationCategory::Analytics,
                "Digital analytics platform",
            ),
            (
                "Google Ads",
                "google-ads",
                IntegrationCategory::Advertising,
                "Google advertising platform",
            ),
            (
                "Meta Ads",
                "meta-ads",
                IntegrationCategory::Advertising,
                "Meta/Facebook advertising",
            ),
            (
                "Shopify",
                "shopify",
                IntegrationCategory::ECommerce,
                "E-commerce platform integration",
            ),
            (
                "Stripe",
                "stripe",
                IntegrationCategory::PaymentProvider,
                "Payment processing platform",
            ),
            (
                "Twilio",
                "twilio",
                IntegrationCategory::Messaging,
                "SMS and voice messaging",
            ),
            (
                "AWS S3",
                "aws-s3",
                IntegrationCategory::CloudStorage,
                "Amazon S3 cloud storage",
            ),
            (
                "HubSpot",
                "hubspot",
                IntegrationCategory::Crm,
                "Inbound marketing and CRM",
            ),
            (
                "Zendesk",
                "zendesk",
                IntegrationCategory::Crm,
                "Customer service platform",
            ),
            (
                "Slack",
                "slack",
                IntegrationCategory::Messaging,
                "Team communication notifications",
            ),
            (
                "PostgreSQL",
                "postgresql",
                IntegrationCategory::DataWarehouse,
                "PostgreSQL database connector",
            ),
            (
                "MongoDB",
                "mongodb",
                IntegrationCategory::DataWarehouse,
                "MongoDB document database connector",
            ),
            (
                "Intercom",
                "intercom",
                IntegrationCategory::Messaging,
                "Customer messaging platform",
            ),
            (
                "Braze",
                "braze",
                IntegrationCategory::Messaging,
                "Customer engagement platform (migration)",
            ),
            (
                "Mailchimp",
                "mailchimp",
                IntegrationCategory::Messaging,
                "Email marketing platform",
            ),
            (
                "Klaviyo",
                "klaviyo",
                IntegrationCategory::Messaging,
                "E-commerce email and SMS",
            ),
            (
                "TikTok Ads",
                "tiktok-ads",
                IntegrationCategory::Advertising,
                "TikTok advertising platform",
            ),
        ];
        for (name, slug, category, desc) in integrations {
            let def = IntegrationDefinition {
                id: Uuid::new_v4(),
                name: name.to_string(),
                slug: slug.to_string(),
                description: desc.to_string(),
                category,
                icon_url: None,
                auth_type: AuthType::ApiKey,
                config_schema: serde_json::json!({}),
                supported_actions: vec!["sync_users".to_string(), "export_events".to_string()],
                supported_triggers: vec!["user_created".to_string(), "event_received".to_string()],
                documentation_url: None,
                is_premium: false,
                version: "1.0.0".to_string(),
            };
            self.catalog.insert(def.id, def);
        }
    }

    pub fn list_catalog(&self) -> Vec<IntegrationDefinition> {
        self.catalog.iter().map(|d| d.value().clone()).collect()
    }

    pub fn get_definition(&self, id: &Uuid) -> Option<IntegrationDefinition> {
        self.catalog.get(id).map(|d| d.clone())
    }

    pub fn search_catalog(&self, query: &str) -> Vec<IntegrationDefinition> {
        let q = query.to_lowercase();
        self.catalog
            .iter()
            .filter(|d| {
                d.value().name.to_lowercase().contains(&q)
                    || d.value().description.to_lowercase().contains(&q)
            })
            .map(|d| d.value().clone())
            .collect()
    }

    pub fn install(
        &self,
        integration_id: Uuid,
        tenant_id: Uuid,
        name: String,
        config: serde_json::Value,
    ) -> Option<InstalledIntegration> {
        self.catalog.get(&integration_id)?;
        let now = Utc::now();
        let installed = InstalledIntegration {
            id: Uuid::new_v4(),
            integration_id,
            tenant_id,
            name,
            config,
            enabled: true,
            last_sync: None,
            sync_status: SyncStatus::NeverSynced,
            error_count: 0,
            installed_at: now,
            updated_at: now,
        };
        self.installed.insert(installed.id, installed.clone());
        Some(installed)
    }

    pub fn list_installed(&self, tenant_id: &Uuid) -> Vec<InstalledIntegration> {
        self.installed
            .iter()
            .filter(|i| &i.value().tenant_id == tenant_id)
            .map(|i| i.value().clone())
            .collect()
    }
}

impl Default for IntegrationMarketplace {
    fn default() -> Self {
        Self::new()
    }
}
