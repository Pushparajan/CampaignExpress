//! Use-case guides — step-by-step tutorials for common campaign patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuideDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guide {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub difficulty: GuideDifficulty,
    pub estimated_minutes: u32,
    pub tags: Vec<String>,
    pub steps: Vec<GuideStep>,
    pub prerequisites: Vec<String>,
    pub business_objective: String,
    pub expected_results: String,
    pub common_pitfalls: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideStep {
    pub number: u32,
    pub title: String,
    pub description: String,
    pub code_snippets: Vec<CodeSnippet>,
    pub screenshot_url: Option<String>,
    pub tips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub language: String,
    pub code: String,
    pub description: Option<String>,
}

pub struct GuideEngine {
    guides: dashmap::DashMap<Uuid, Guide>,
}

impl GuideEngine {
    pub fn new() -> Self {
        let engine = Self {
            guides: dashmap::DashMap::new(),
        };
        engine.seed_guides();
        engine
    }

    fn seed_guides(&self) {
        let guide_defs = vec![
            (
                "welcome-series",
                "Building a Welcome Email Series",
                GuideDifficulty::Beginner,
                30,
                "Set up automated welcome emails for new users",
                "Increase day-7 retention by 25%",
            ),
            (
                "cart-abandonment",
                "Cart Abandonment Recovery",
                GuideDifficulty::Intermediate,
                45,
                "Recover lost revenue with multi-channel cart reminders",
                "Recover 15-20% of abandoned carts",
            ),
            (
                "win-back-campaign",
                "Win-Back Dormant Users",
                GuideDifficulty::Intermediate,
                40,
                "Re-engage users who haven't been active in 60+ days",
                "Reactivate 10-15% of dormant users",
            ),
            (
                "post-purchase",
                "Post-Purchase Follow-Up",
                GuideDifficulty::Beginner,
                25,
                "Automate thank you, review request, and cross-sell after purchase",
                "Increase repeat purchase rate by 20%",
            ),
            (
                "birthday-campaign",
                "Birthday Campaign",
                GuideDifficulty::Beginner,
                20,
                "Delight customers with personalized birthday offers",
                "45% open rate, 12% redemption rate",
            ),
            (
                "subscription-renewal",
                "Subscription Renewal Reminders",
                GuideDifficulty::Intermediate,
                35,
                "Reduce involuntary churn with renewal reminders",
                "Reduce churn by 30%",
            ),
            (
                "referral-program",
                "Referral Program Automation",
                GuideDifficulty::Advanced,
                60,
                "Build a viral referral loop with automated rewards",
                "20% of new signups from referrals",
            ),
            (
                "onboarding-checklist",
                "Mobile App Onboarding",
                GuideDifficulty::Intermediate,
                40,
                "Guide new users through key activation milestones",
                "Increase activation rate by 35%",
            ),
            (
                "re-engagement",
                "App Re-Engagement Campaign",
                GuideDifficulty::Intermediate,
                35,
                "Multi-channel re-engagement for lapsed app users",
                "Bring back 25% of lapsed users",
            ),
            (
                "upsell-campaign",
                "Free-to-Paid Upsell",
                GuideDifficulty::Advanced,
                50,
                "Convert free users to paid with targeted feature education",
                "15% free-to-paid conversion rate",
            ),
            (
                "ab-testing",
                "A/B Testing Best Practices",
                GuideDifficulty::Intermediate,
                30,
                "Design and analyze A/B tests for campaign optimization",
                "Achieve statistical significance in 7 days",
            ),
            (
                "multi-channel-journey",
                "Multi-Channel Journey",
                GuideDifficulty::Advanced,
                60,
                "Orchestrate email → SMS → push sequences with branching logic",
                "3x engagement vs single channel",
            ),
            (
                "real-time-personalization",
                "Real-Time Personalization",
                GuideDifficulty::Advanced,
                45,
                "Personalize content with Liquid templates and connected content",
                "25% CTR improvement with personalization",
            ),
            (
                "gdpr-compliance",
                "GDPR Data Export & Deletion",
                GuideDifficulty::Intermediate,
                30,
                "Implement GDPR data subject request handling",
                "Full GDPR compliance with automated DSR processing",
            ),
            (
                "high-volume-optimization",
                "High-Volume Campaign Optimization",
                GuideDifficulty::Advanced,
                45,
                "Optimize send performance for 1M+ recipient campaigns",
                "Deliver 1M emails in under 30 minutes",
            ),
            (
                "product-recommendations",
                "Product Recommendations",
                GuideDifficulty::Intermediate,
                40,
                "Add personalized product recommendations to emails and in-app",
                "15% revenue lift from recommendations",
            ),
            (
                "customer-feedback",
                "Customer Feedback Collection",
                GuideDifficulty::Beginner,
                25,
                "Automate NPS and CSAT surveys after key interactions",
                "30% survey response rate",
            ),
            (
                "seasonal-planning",
                "Seasonal Campaign Planning",
                GuideDifficulty::Intermediate,
                35,
                "Plan and execute holiday and seasonal campaigns at scale",
                "2x revenue during peak seasons",
            ),
            (
                "b2b-lead-nurture",
                "B2B Lead Nurturing",
                GuideDifficulty::Advanced,
                50,
                "Build multi-touch nurture sequences for B2B prospects",
                "40% increase in marketing qualified leads",
            ),
            (
                "event-triggered",
                "Event-Triggered Campaigns",
                GuideDifficulty::Beginner,
                30,
                "Set up real-time campaigns triggered by user actions",
                "5x engagement vs scheduled campaigns",
            ),
            (
                "campaign-workflows",
                "Campaign Approval Workflows",
                GuideDifficulty::Intermediate,
                35,
                "Set up multi-step approval workflows with role-based review",
                "100% compliance with governance requirements",
            ),
            (
                "brand-guidelines",
                "Brand Guidelines Enforcement",
                GuideDifficulty::Intermediate,
                30,
                "Configure and enforce brand color, font, tone, and logo guidelines",
                "Zero off-brand creative launches",
            ),
            (
                "budget-tracking",
                "Campaign Budget Tracking & Alerts",
                GuideDifficulty::Beginner,
                25,
                "Monitor campaign spend with pacing alerts and ROAS tracking",
                "15% improvement in budget utilization",
            ),
            (
                "report-builder",
                "Custom Report Builder",
                GuideDifficulty::Intermediate,
                30,
                "Create scheduled reports with CSV/JSON export and custom filters",
                "Automated daily performance reports",
            ),
            (
                "offerfit-integration",
                "OfferFit RL Engine Integration",
                GuideDifficulty::Advanced,
                50,
                "Connect to OfferFit for reinforcement learning optimization with Thompson Sampling fallback",
                "25% improvement in offer conversion rate",
            ),
            (
                "inference-providers",
                "Hardware-Agnostic Inference Setup",
                GuideDifficulty::Advanced,
                45,
                "Configure inference backends for Groq LPU, AWS Inferentia, Ampere ARM, or Tenstorrent RISC-V",
                "3x inference throughput with hardware acceleration",
            ),
            (
                "dam-integration",
                "Digital Asset Management Integration",
                GuideDifficulty::Intermediate,
                40,
                "Connect AEM Assets, Bynder, or Aprimo for centralized creative management",
                "50% reduction in asset search time",
            ),
            (
                "task-management",
                "Task Management Integration",
                GuideDifficulty::Beginner,
                20,
                "Create Asana or Jira tasks automatically from campaign approval workflows",
                "Complete visibility into campaign review pipeline",
            ),
            (
                "suppression-lists",
                "Global Suppression List Management",
                GuideDifficulty::Beginner,
                20,
                "Manage per-channel suppression lists with automatic expiry for compliance",
                "Full CAN-SPAM and GDPR compliance",
            ),
        ];

        for (slug, title, difficulty, minutes, objective, results) in guide_defs {
            let guide = Guide {
                id: Uuid::new_v4(),
                slug: slug.to_string(),
                title: title.to_string(),
                description: objective.to_string(),
                difficulty,
                estimated_minutes: minutes,
                tags: vec![slug.split('-').next().unwrap_or("general").to_string()],
                steps: vec![
                    GuideStep {
                        number: 1,
                        title: "Setup prerequisites".to_string(),
                        description: "Ensure your environment is configured".to_string(),
                        code_snippets: Vec::new(),
                        screenshot_url: None,
                        tips: vec!["Check API key is valid".to_string()],
                    },
                    GuideStep {
                        number: 2,
                        title: "Configure the campaign".to_string(),
                        description: "Set up targeting, content, and triggers".to_string(),
                        code_snippets: Vec::new(),
                        screenshot_url: None,
                        tips: Vec::new(),
                    },
                    GuideStep {
                        number: 3,
                        title: "Test and launch".to_string(),
                        description: "Validate with test users and go live".to_string(),
                        code_snippets: Vec::new(),
                        screenshot_url: None,
                        tips: vec!["Always test with a small segment first".to_string()],
                    },
                ],
                prerequisites: vec![
                    "API key configured".to_string(),
                    "SDK installed".to_string(),
                ],
                business_objective: objective.to_string(),
                expected_results: results.to_string(),
                common_pitfalls: vec![
                    "Not segmenting properly".to_string(),
                    "Missing event tracking".to_string(),
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.guides.insert(guide.id, guide);
        }
    }

    pub fn list_guides(&self) -> Vec<Guide> {
        self.guides.iter().map(|g| g.value().clone()).collect()
    }

    pub fn get_guide(&self, slug: &str) -> Option<Guide> {
        self.guides
            .iter()
            .find(|g| g.value().slug == slug)
            .map(|g| g.value().clone())
    }

    pub fn search_guides(&self, query: &str) -> Vec<Guide> {
        let q = query.to_lowercase();
        self.guides
            .iter()
            .filter(|g| {
                g.value().title.to_lowercase().contains(&q)
                    || g.value().description.to_lowercase().contains(&q)
            })
            .map(|g| g.value().clone())
            .collect()
    }

    pub fn list_by_difficulty(&self, difficulty: &GuideDifficulty) -> Vec<Guide> {
        let target = serde_json::to_string(difficulty).unwrap_or_default();
        self.guides
            .iter()
            .filter(|g| serde_json::to_string(&g.value().difficulty).unwrap_or_default() == target)
            .map(|g| g.value().clone())
            .collect()
    }
}

impl Default for GuideEngine {
    fn default() -> Self {
        Self::new()
    }
}
