//! Documentation search — full-text search across API reference, guides, and examples.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub section: DocSection,
    pub url: String,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSection {
    GettingStarted,
    ApiReference,
    SdkIos,
    SdkAndroid,
    SdkReactNative,
    SdkFlutter,
    SdkWeb,
    SdkServer,
    Guides,
    Integrations,
    Plugins,
    Changelog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub section_filter: Option<DocSection>,
    pub limit: usize,
}

pub struct DocSearchEngine {
    documents: dashmap::DashMap<Uuid, SearchResult>,
}

impl DocSearchEngine {
    pub fn new() -> Self {
        let engine = Self {
            documents: dashmap::DashMap::new(),
        };
        engine.seed_index();
        engine
    }

    fn seed_index(&self) {
        let docs = vec![
            (
                "Quick Start Guide",
                "Get started with CampaignExpress in 5 minutes",
                DocSection::GettingStarted,
                "/docs/quickstart",
            ),
            (
                "Authentication",
                "API keys, OAuth, and SDK initialization",
                DocSection::GettingStarted,
                "/docs/auth",
            ),
            (
                "iOS SDK Installation",
                "Install via CocoaPods or Swift Package Manager",
                DocSection::SdkIos,
                "/docs/sdk/ios/install",
            ),
            (
                "Android SDK Installation",
                "Install via Maven Central",
                DocSection::SdkAndroid,
                "/docs/sdk/android/install",
            ),
            (
                "React Native SDK",
                "Cross-platform SDK for React Native apps",
                DocSection::SdkReactNative,
                "/docs/sdk/react-native",
            ),
            (
                "Flutter SDK",
                "Cross-platform SDK for Flutter apps",
                DocSection::SdkFlutter,
                "/docs/sdk/flutter",
            ),
            (
                "Web SDK",
                "JavaScript SDK for web applications",
                DocSection::SdkWeb,
                "/docs/sdk/web",
            ),
            (
                "User Tracking",
                "Track user events, purchases, and attributes",
                DocSection::ApiReference,
                "/docs/api/tracking",
            ),
            (
                "Push Notifications",
                "Setup and send push notifications",
                DocSection::ApiReference,
                "/docs/api/push",
            ),
            (
                "In-App Messages",
                "Configure and display in-app messages",
                DocSection::ApiReference,
                "/docs/api/in-app",
            ),
            (
                "Content Cards",
                "Persistent content card feed management",
                DocSection::ApiReference,
                "/docs/api/content-cards",
            ),
            (
                "Segments API",
                "Create and manage audience segments",
                DocSection::ApiReference,
                "/docs/api/segments",
            ),
            (
                "Campaigns API",
                "Campaign CRUD and triggering",
                DocSection::ApiReference,
                "/docs/api/campaigns",
            ),
            (
                "Webhooks",
                "Receive real-time event notifications",
                DocSection::ApiReference,
                "/docs/api/webhooks",
            ),
            (
                "Segment Integration",
                "Connect with Segment CDP",
                DocSection::Integrations,
                "/docs/integrations/segment",
            ),
            (
                "Salesforce Integration",
                "Sync with Salesforce CRM",
                DocSection::Integrations,
                "/docs/integrations/salesforce",
            ),
            (
                "Plugin Development",
                "Build and publish marketplace plugins",
                DocSection::Plugins,
                "/docs/plugins/getting-started",
            ),
            (
                "Cart Abandonment Guide",
                "Build a cart abandonment recovery flow",
                DocSection::Guides,
                "/docs/guides/cart-abandonment",
            ),
            (
                "Welcome Series Guide",
                "Automated onboarding email series",
                DocSection::Guides,
                "/docs/guides/welcome-series",
            ),
            (
                "GDPR Compliance",
                "Data deletion and export guides",
                DocSection::Guides,
                "/docs/guides/gdpr",
            ),
            (
                "Campaign Workflows",
                "Multi-step approval workflows with role-based review",
                DocSection::ApiReference,
                "/docs/api/workflows",
            ),
            (
                "Brand Guidelines",
                "Enforce brand colors, fonts, tone, and logo usage",
                DocSection::ApiReference,
                "/docs/api/brand",
            ),
            (
                "Budget Tracking",
                "Campaign budget pacing, ROAS, and spend alerts",
                DocSection::ApiReference,
                "/docs/api/budget",
            ),
            (
                "Report Builder",
                "Custom reports with CSV/JSON export and scheduling",
                DocSection::ApiReference,
                "/docs/api/reports",
            ),
            (
                "Recommendations",
                "Personalized recommendations via CF, content-based, and trending strategies",
                DocSection::ApiReference,
                "/docs/api/recommendations",
            ),
            (
                "Suppression Lists",
                "Global per-channel suppression with automatic expiry",
                DocSection::ApiReference,
                "/docs/api/suppression",
            ),
            (
                "OfferFit Integration",
                "Reinforcement learning optimization via OfferFit connector",
                DocSection::Integrations,
                "/docs/integrations/offerfit",
            ),
            (
                "Asana Integration",
                "Create tasks in Asana from campaign workflows",
                DocSection::Integrations,
                "/docs/integrations/asana",
            ),
            (
                "Jira Integration",
                "Create issues in Jira from campaign workflows",
                DocSection::Integrations,
                "/docs/integrations/jira",
            ),
            (
                "DAM Integration",
                "Connect AEM Assets, Bynder, or Aprimo for asset management",
                DocSection::Integrations,
                "/docs/integrations/dam",
            ),
            (
                "Power BI Integration",
                "Push campaign data to Power BI dashboards",
                DocSection::Integrations,
                "/docs/integrations/powerbi",
            ),
            (
                "Excel Export",
                "Generate Excel reports from campaign data",
                DocSection::Integrations,
                "/docs/integrations/excel",
            ),
            (
                "Inference Providers",
                "Hardware-agnostic inference with Groq, Inferentia, Ampere, Tenstorrent",
                DocSection::ApiReference,
                "/docs/api/inference",
            ),
            (
                "Inference Provider Guide",
                "Configure hardware backends for optimal inference performance",
                DocSection::Guides,
                "/docs/guides/inference-providers",
            ),
            (
                "Workflow Guide",
                "Set up campaign approval workflows",
                DocSection::Guides,
                "/docs/guides/workflows",
            ),
            (
                "Brand Guidelines Guide",
                "Configure brand enforcement for creatives",
                DocSection::Guides,
                "/docs/guides/brand",
            ),
        ];

        for (title, desc, section, url) in docs {
            let result = SearchResult {
                id: Uuid::new_v4(),
                title: title.to_string(),
                description: desc.to_string(),
                section,
                url: url.to_string(),
                relevance_score: 0.0,
            };
            self.documents.insert(result.id, result);
        }
    }

    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let q = query.query.to_lowercase();
        let words: Vec<&str> = q.split_whitespace().collect();

        let mut results: Vec<_> = self
            .documents
            .iter()
            .filter_map(|entry| {
                let doc = entry.value();
                if let Some(ref filter) = query.section_filter {
                    let filter_str = serde_json::to_string(filter).unwrap_or_default();
                    let doc_str = serde_json::to_string(&doc.section).unwrap_or_default();
                    if filter_str != doc_str {
                        return None;
                    }
                }

                let title_lower = doc.title.to_lowercase();
                let desc_lower = doc.description.to_lowercase();

                let mut score = 0.0;
                for word in &words {
                    if title_lower.contains(word) {
                        score += 10.0;
                    }
                    if desc_lower.contains(word) {
                        score += 5.0;
                    }
                    if doc.url.contains(word) {
                        score += 3.0;
                    }
                }

                if score > 0.0 {
                    let mut result = doc.clone();
                    result.relevance_score = score;
                    Some(result)
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(query.limit);
        results
    }

    pub fn list_all(&self) -> Vec<SearchResult> {
        self.documents.iter().map(|d| d.value().clone()).collect()
    }
}

impl Default for DocSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
