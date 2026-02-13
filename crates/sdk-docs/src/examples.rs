//! Code examples and campaign templates library.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: String,
    pub languages: Vec<LanguageExample>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageExample {
    pub language: String,
    pub code: String,
    pub imports: Option<String>,
    pub expected_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub difficulty: String,
    pub estimated_setup_minutes: u32,
    pub journey_json: serde_json::Value,
    pub expected_metrics: TemplateMetrics,
    pub customization_tips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetrics {
    pub expected_open_rate: String,
    pub expected_click_rate: String,
    pub expected_conversion_rate: String,
}

pub struct ExampleLibrary {
    examples: dashmap::DashMap<Uuid, CodeExample>,
    templates: dashmap::DashMap<Uuid, CampaignTemplate>,
}

impl ExampleLibrary {
    pub fn new() -> Self {
        let lib = Self {
            examples: dashmap::DashMap::new(),
            templates: dashmap::DashMap::new(),
        };
        lib.seed();
        lib
    }

    fn seed(&self) {
        let code_examples = vec![
            (
                "Track Custom Event",
                "events",
                "Record a custom event with properties",
            ),
            ("Identify User", "users", "Create or update a user profile"),
            (
                "Log Purchase",
                "events",
                "Track a purchase event with revenue",
            ),
            (
                "Send Transactional Email",
                "messaging",
                "Send a one-off email via API",
            ),
            (
                "Register Push Token",
                "push",
                "Register a device push token",
            ),
            (
                "Fetch Content Cards",
                "content-cards",
                "Get content cards for a user",
            ),
            (
                "Create Segment",
                "segments",
                "Create a dynamic segment via API",
            ),
            (
                "Trigger Campaign",
                "campaigns",
                "Trigger an API-triggered campaign",
            ),
            (
                "Handle Webhook",
                "webhooks",
                "Process incoming webhook events",
            ),
            ("Bulk User Import", "users", "Import users from CSV via API"),
            (
                "Submit for Approval",
                "workflows",
                "Submit a campaign for multi-step approval workflow",
            ),
            (
                "Validate Brand Guidelines",
                "brand",
                "Check creative content against brand guidelines",
            ),
            (
                "Get Budget Status",
                "reporting",
                "Retrieve budget pacing and ROAS for a campaign",
            ),
            (
                "Generate Report",
                "reporting",
                "Build and export a custom report with filters",
            ),
            (
                "Get Recommendations",
                "personalization",
                "Fetch personalized recommendations using collaborative filtering",
            ),
            (
                "Create Suppression Entry",
                "delivery",
                "Add a user to the global suppression list",
            ),
            (
                "Search DAM Assets",
                "integrations",
                "Search for creative assets across connected DAM platforms",
            ),
            (
                "Push to Power BI",
                "integrations",
                "Push campaign analytics data to Power BI",
            ),
            (
                "Create Jira Task",
                "integrations",
                "Create a Jira issue for campaign review tracking",
            ),
            (
                "OfferFit Recommendation",
                "ml",
                "Get an RL-optimized offer recommendation from OfferFit",
            ),
        ];

        for (title, category, desc) in code_examples {
            let example = CodeExample {
                id: Uuid::new_v4(),
                title: title.to_string(),
                description: desc.to_string(),
                category: category.to_string(),
                languages: vec![
                    LanguageExample {
                        language: "python".to_string(),
                        code: format!("# {} in Python\nimport campaignexpress as ce\n\nclient = ce.Client('YOUR_API_KEY')\n# ...", title),
                        imports: Some("pip install campaignexpress".to_string()),
                        expected_output: Some("{'success': true}".to_string()),
                    },
                    LanguageExample {
                        language: "javascript".to_string(),
                        code: format!("// {} in JavaScript\nimport CampaignExpress from 'campaignexpress';\n\nconst client = new CampaignExpress('YOUR_API_KEY');\n// ...", title),
                        imports: Some("npm install campaignexpress".to_string()),
                        expected_output: Some("{ success: true }".to_string()),
                    },
                    LanguageExample {
                        language: "curl".to_string(),
                        code: format!("# {}\ncurl -X POST 'https://api.campaignexpress.io/v1/...' \\\n  -H 'Authorization: Bearer YOUR_API_KEY' \\\n  -H 'Content-Type: application/json'", title),
                        imports: None,
                        expected_output: None,
                    },
                ],
                tags: vec![category.to_string()],
            };
            self.examples.insert(example.id, example);
        }

        let templates_data = vec![
            (
                "Welcome Series",
                "onboarding",
                "3-email welcome series for new users",
                15,
                "45%",
                "8%",
                "3%",
            ),
            (
                "Cart Abandonment",
                "ecommerce",
                "Multi-channel cart recovery flow",
                30,
                "50%",
                "12%",
                "8%",
            ),
            (
                "Win-Back Campaign",
                "retention",
                "Re-engage dormant users",
                25,
                "30%",
                "5%",
                "2%",
            ),
            (
                "Post-Purchase Follow-Up",
                "ecommerce",
                "Thank you + review + cross-sell",
                20,
                "55%",
                "10%",
                "5%",
            ),
            (
                "Birthday Campaign",
                "loyalty",
                "Personalized birthday discount",
                10,
                "60%",
                "15%",
                "12%",
            ),
            (
                "Subscription Renewal",
                "retention",
                "Renewal reminders to reduce churn",
                20,
                "65%",
                "8%",
                "4%",
            ),
            (
                "Referral Program",
                "growth",
                "Viral referral loop automation",
                40,
                "40%",
                "20%",
                "8%",
            ),
            (
                "Onboarding Checklist",
                "onboarding",
                "App onboarding milestone tracking",
                30,
                "55%",
                "12%",
                "6%",
            ),
            (
                "Re-Engagement",
                "retention",
                "Multi-channel lapsed user recovery",
                25,
                "35%",
                "6%",
                "3%",
            ),
            (
                "Free-to-Paid Upsell",
                "monetization",
                "Convert free users with feature education",
                35,
                "40%",
                "8%",
                "4%",
            ),
            (
                "Approval Workflow",
                "governance",
                "Multi-step campaign approval flow",
                25,
                "N/A",
                "N/A",
                "95%",
            ),
            (
                "Budget-Aware Campaign",
                "finance",
                "Campaign with budget pacing and ROAS alerts",
                20,
                "50%",
                "10%",
                "6%",
            ),
            (
                "RL-Optimized Offers",
                "ml",
                "OfferFit reinforcement learning powered offers",
                35,
                "55%",
                "15%",
                "10%",
            ),
        ];

        for (name, cat, desc, minutes, open, click, conv) in templates_data {
            let template = CampaignTemplate {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: desc.to_string(),
                category: cat.to_string(),
                difficulty: if minutes <= 15 {
                    "beginner"
                } else if minutes <= 30 {
                    "intermediate"
                } else {
                    "advanced"
                }
                .to_string(),
                estimated_setup_minutes: minutes,
                journey_json: serde_json::json!({
                    "name": name,
                    "steps": [
                        {"type": "trigger", "event": "user_created"},
                        {"type": "delay", "duration": "1d"},
                        {"type": "send", "channel": "email"},
                    ]
                }),
                expected_metrics: TemplateMetrics {
                    expected_open_rate: open.to_string(),
                    expected_click_rate: click.to_string(),
                    expected_conversion_rate: conv.to_string(),
                },
                customization_tips: vec![
                    "Adjust delay times based on your audience".to_string(),
                    "A/B test subject lines for best results".to_string(),
                ],
            };
            self.templates.insert(template.id, template);
        }
    }

    pub fn list_examples(&self) -> Vec<CodeExample> {
        self.examples.iter().map(|e| e.value().clone()).collect()
    }

    pub fn list_templates(&self) -> Vec<CampaignTemplate> {
        self.templates.iter().map(|t| t.value().clone()).collect()
    }

    pub fn search_examples(&self, query: &str) -> Vec<CodeExample> {
        let q = query.to_lowercase();
        self.examples
            .iter()
            .filter(|e| {
                e.value().title.to_lowercase().contains(&q)
                    || e.value().description.to_lowercase().contains(&q)
            })
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn get_template(&self, name: &str) -> Option<CampaignTemplate> {
        let q = name.to_lowercase();
        self.templates
            .iter()
            .find(|t| t.value().name.to_lowercase().contains(&q))
            .map(|t| t.value().clone())
    }
}

impl Default for ExampleLibrary {
    fn default() -> Self {
        Self::new()
    }
}
