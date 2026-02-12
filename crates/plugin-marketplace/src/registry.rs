//! Plugin registry — catalog of plugins with versioning, reviews, and analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCategory {
    Channels,
    Integrations,
    Analytics,
    MachineLearning,
    Automation,
    Data,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPricing {
    Free,
    Paid {
        monthly_price: f64,
        currency: String,
    },
    Freemium {
        free_tier_limit: String,
    },
    UsageBased {
        per_unit_price: f64,
        unit: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    Draft,
    PendingReview,
    Published,
    Deprecated,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Unverified,
    Verified,
    Official,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDefinition {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub category: PluginCategory,
    pub icon_url: Option<String>,
    pub screenshots: Vec<String>,
    pub video_url: Option<String>,
    pub developer_id: Uuid,
    pub developer_name: String,
    pub pricing: PluginPricing,
    pub status: PluginStatus,
    pub verification: VerificationStatus,
    pub permissions: Vec<PluginPermission>,
    pub latest_version: String,
    pub min_platform_version: String,
    pub install_count: u64,
    pub rating_average: f64,
    pub rating_count: u64,
    pub tags: Vec<String>,
    pub documentation_url: Option<String>,
    pub support_email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub scope: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub version: String,
    pub changelog: String,
    pub manifest: serde_json::Value,
    pub package_url: String,
    pub package_checksum: String,
    pub min_platform_version: String,
    pub download_count: u64,
    pub status: PluginStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub rating: u8,
    pub title: String,
    pub body: String,
    pub helpful_count: u32,
    pub verified_install: bool,
    pub status: ReviewStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchQuery {
    pub query: Option<String>,
    pub category: Option<PluginCategory>,
    pub pricing_filter: Option<String>,
    pub verification_filter: Option<VerificationStatus>,
    pub min_rating: Option<f64>,
    pub sort_by: PluginSortBy,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginSortBy {
    #[default]
    Popular,
    Newest,
    TopRated,
    NameAz,
}

pub struct PluginRegistry {
    plugins: dashmap::DashMap<Uuid, PluginDefinition>,
    versions: dashmap::DashMap<Uuid, Vec<PluginVersion>>,
    reviews: dashmap::DashMap<Uuid, Vec<PluginReview>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let registry = Self {
            plugins: dashmap::DashMap::new(),
            versions: dashmap::DashMap::new(),
            reviews: dashmap::DashMap::new(),
        };
        registry.seed_plugins();
        registry
    }

    fn seed_plugins(&self) {
        let plugins = vec![
            (
                "Segment CDP",
                "segment-cdp",
                PluginCategory::Integrations,
                "Sync user profiles and events with Segment",
            ),
            (
                "Salesforce CRM",
                "salesforce-crm",
                PluginCategory::Integrations,
                "Bidirectional sync with Salesforce leads and contacts",
            ),
            (
                "Snowflake Export",
                "snowflake-export",
                PluginCategory::Data,
                "Export campaign data to Snowflake data warehouse",
            ),
            (
                "Custom ML Model",
                "custom-ml",
                PluginCategory::MachineLearning,
                "Bring your own ML model for scoring",
            ),
            (
                "Slack Notifications",
                "slack-notify",
                PluginCategory::Channels,
                "Send campaign alerts to Slack channels",
            ),
            (
                "WhatsApp Business",
                "whatsapp-biz",
                PluginCategory::Channels,
                "WhatsApp Business API messaging",
            ),
            (
                "Google Analytics 4",
                "ga4",
                PluginCategory::Analytics,
                "Export conversion events to GA4",
            ),
            (
                "Mixpanel Sync",
                "mixpanel-sync",
                PluginCategory::Analytics,
                "Bidirectional event sync with Mixpanel",
            ),
            (
                "Shopify E-Commerce",
                "shopify-ecom",
                PluginCategory::Integrations,
                "Product catalog and purchase sync",
            ),
            (
                "Stripe Revenue",
                "stripe-revenue",
                PluginCategory::Integrations,
                "Revenue attribution with Stripe",
            ),
            (
                "SMS Gateway",
                "sms-gateway",
                PluginCategory::Channels,
                "Multi-provider SMS gateway",
            ),
            (
                "Data Anonymizer",
                "data-anon",
                PluginCategory::Data,
                "GDPR-compliant data anonymization",
            ),
            (
                "A/B Test Analyzer",
                "ab-analyzer",
                PluginCategory::Analytics,
                "Advanced statistical analysis for experiments",
            ),
            (
                "Webhook Relay",
                "webhook-relay",
                PluginCategory::Automation,
                "Forward events to external webhooks",
            ),
            (
                "Customer Scorer",
                "customer-scorer",
                PluginCategory::MachineLearning,
                "Predictive customer lifetime value scoring",
            ),
            (
                "Email Validator",
                "email-validator",
                PluginCategory::Data,
                "Real-time email address validation",
            ),
            (
                "Push Rich Media",
                "push-richmedia",
                PluginCategory::Channels,
                "Rich push notifications with images and actions",
            ),
            (
                "Zendesk Support",
                "zendesk-support",
                PluginCategory::Integrations,
                "Create support tickets from campaign interactions",
            ),
            (
                "BigQuery Sync",
                "bigquery-sync",
                PluginCategory::Data,
                "Real-time event streaming to BigQuery",
            ),
            (
                "Custom Dashboard",
                "custom-dash",
                PluginCategory::Analytics,
                "Build custom analytics dashboards",
            ),
        ];

        for (i, (name, slug, category, desc)) in plugins.into_iter().enumerate() {
            let id = Uuid::new_v4();
            let plugin = PluginDefinition {
                id,
                slug: slug.to_string(),
                name: name.to_string(),
                tagline: desc.to_string(),
                description: format!("{}\n\nFull-featured integration with automatic sync, error handling, and monitoring.", desc),
                category,
                icon_url: None,
                screenshots: Vec::new(),
                video_url: None,
                developer_id: Uuid::new_v4(),
                developer_name: if i < 5 { "CampaignExpress".to_string() } else { format!("Partner {}", i) },
                pricing: if i < 10 { PluginPricing::Free } else { PluginPricing::Paid { monthly_price: 29.0, currency: "USD".to_string() } },
                status: PluginStatus::Published,
                verification: if i < 5 { VerificationStatus::Official } else { VerificationStatus::Verified },
                permissions: vec![
                    PluginPermission { scope: "users.read".to_string(), description: "Read user profiles".to_string(), required: true },
                    PluginPermission { scope: "events.read".to_string(), description: "Read event data".to_string(), required: true },
                ],
                latest_version: "1.0.0".to_string(),
                min_platform_version: "0.1.0".to_string(),
                install_count: (20 - i as u64) * 500 + 100,
                rating_average: 4.0 + (i % 10) as f64 * 0.1,
                rating_count: (20 - i as u64) * 10,
                tags: vec![slug.to_string()],
                documentation_url: None,
                support_email: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                published_at: Some(Utc::now()),
            };
            self.plugins.insert(id, plugin);
        }
    }

    pub fn search(&self, query: &PluginSearchQuery) -> Vec<PluginDefinition> {
        let mut results: Vec<_> = self
            .plugins
            .iter()
            .filter(|p| {
                let plugin = p.value();
                if !matches!(plugin.status, PluginStatus::Published) {
                    return false;
                }
                if let Some(q) = &query.query {
                    let q = q.to_lowercase();
                    if !plugin.name.to_lowercase().contains(&q)
                        && !plugin.description.to_lowercase().contains(&q)
                        && !plugin.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    {
                        return false;
                    }
                }
                if let Some(min) = query.min_rating {
                    if plugin.rating_average < min {
                        return false;
                    }
                }
                true
            })
            .map(|p| p.value().clone())
            .collect();

        match query.sort_by {
            PluginSortBy::Popular => results.sort_by(|a, b| b.install_count.cmp(&a.install_count)),
            PluginSortBy::Newest => results.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            PluginSortBy::TopRated => results.sort_by(|a, b| {
                b.rating_average
                    .partial_cmp(&a.rating_average)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            PluginSortBy::NameAz => results.sort_by(|a, b| a.name.cmp(&b.name)),
        }

        let start = (query.page * query.per_page) as usize;
        results
            .into_iter()
            .skip(start)
            .take(query.per_page as usize)
            .collect()
    }

    pub fn get_plugin(&self, slug: &str) -> Option<PluginDefinition> {
        self.plugins
            .iter()
            .find(|p| p.value().slug == slug)
            .map(|p| p.value().clone())
    }

    pub fn get_reviews(&self, plugin_id: &Uuid) -> Vec<PluginReview> {
        self.reviews
            .get(plugin_id)
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    pub fn submit_review(&self, review: PluginReview) {
        self.reviews
            .entry(review.plugin_id)
            .or_default()
            .push(review);
    }

    pub fn publish_version(&self, version: PluginVersion) {
        if let Some(mut plugin) = self.plugins.get_mut(&version.plugin_id) {
            plugin.latest_version = version.version.clone();
            plugin.updated_at = Utc::now();
        }
        self.versions
            .entry(version.plugin_id)
            .or_default()
            .push(version);
    }

    pub fn list_all(&self) -> Vec<PluginDefinition> {
        self.plugins.iter().map(|p| p.value().clone()).collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
