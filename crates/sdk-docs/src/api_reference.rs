//! API reference engine — REST endpoints, request/response schemas, and examples.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub id: Uuid,
    pub method: HttpMethod,
    pub path: String,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub parameters: Vec<ApiParameter>,
    pub request_body: Option<ApiSchema>,
    pub responses: Vec<ApiResponse>,
    pub auth_required: bool,
    pub rate_limit: Option<String>,
    pub examples: Vec<ApiExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParameter {
    pub name: String,
    pub location: ParameterLocation,
    pub description: String,
    pub required: bool,
    pub param_type: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchema {
    pub content_type: String,
    pub schema: serde_json::Value,
    pub example: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status_code: u16,
    pub description: String,
    pub schema: Option<serde_json::Value>,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiExample {
    pub language: String,
    pub label: String,
    pub code: String,
}

pub struct ApiReferenceEngine {
    endpoints: dashmap::DashMap<Uuid, ApiEndpoint>,
}

impl ApiReferenceEngine {
    pub fn new() -> Self {
        let engine = Self {
            endpoints: dashmap::DashMap::new(),
        };
        engine.seed_endpoints();
        engine
    }

    fn seed_endpoints(&self) {
        let endpoints = vec![
            ("GET", "/api/v1/users/{user_id}", "Get User Profile", "users",
             "Retrieve a user profile by ID including attributes, segments, and engagement data."),
            ("POST", "/api/v1/users/track", "Track User Event", "events",
             "Record a custom event for a user with optional properties and timestamp."),
            ("POST", "/api/v1/users/identify", "Identify User", "users",
             "Create or update a user profile with attributes."),
            ("POST", "/api/v1/campaigns/trigger/send", "Trigger Campaign Send", "campaigns",
             "Trigger an API-triggered campaign send to specified users."),
            ("GET", "/api/v1/campaigns", "List Campaigns", "campaigns",
             "List all campaigns with optional filters and pagination."),
            ("POST", "/api/v1/campaigns", "Create Campaign", "campaigns",
             "Create a new campaign with channel, content, and targeting configuration."),
            ("GET", "/api/v1/segments", "List Segments", "segments",
             "List all audience segments with estimated sizes."),
            ("POST", "/api/v1/messages/send", "Send Message", "messaging",
             "Send a transactional message to one or more users."),
            ("GET", "/api/v1/analytics/campaigns/{campaign_id}", "Campaign Analytics", "analytics",
             "Get detailed analytics for a specific campaign."),
            ("POST", "/api/v1/users/{user_id}/subscription", "Update Subscription", "users",
             "Update a user's subscription/consent preferences."),
            ("GET", "/api/v1/content-cards/{user_id}", "Get Content Cards", "content-cards",
             "Retrieve active content cards for a user."),
            ("POST", "/api/v1/webhooks", "Register Webhook", "webhooks",
             "Register a webhook endpoint for event notifications."),
            ("GET", "/api/v1/plugins", "List Plugins", "plugins",
             "Browse the plugin marketplace with filters and search."),
            ("POST", "/api/v1/plugins/{slug}/install", "Install Plugin", "plugins",
             "Install a plugin into the current workspace."),
        ];

        for (method, path, summary, tag, description) in endpoints {
            let endpoint = ApiEndpoint {
                id: Uuid::new_v4(),
                method: match method {
                    "GET" => HttpMethod::Get,
                    "POST" => HttpMethod::Post,
                    "PUT" => HttpMethod::Put,
                    "PATCH" => HttpMethod::Patch,
                    "DELETE" => HttpMethod::Delete,
                    _ => HttpMethod::Get,
                },
                path: path.to_string(),
                summary: summary.to_string(),
                description: description.to_string(),
                tags: vec![tag.to_string()],
                parameters: Vec::new(),
                request_body: None,
                responses: vec![
                    ApiResponse {
                        status_code: 200,
                        description: "Success".to_string(),
                        schema: None,
                        example: None,
                    },
                    ApiResponse {
                        status_code: 401,
                        description: "Unauthorized".to_string(),
                        schema: None,
                        example: None,
                    },
                ],
                auth_required: true,
                rate_limit: Some("100 req/min".to_string()),
                examples: vec![
                    ApiExample {
                        language: "curl".to_string(),
                        label: "cURL".to_string(),
                        code: format!(
                            "curl -X {} '{}' \\\n  -H 'Authorization: Bearer YOUR_API_KEY'",
                            method,
                            path.replace("{user_id}", "user_123").replace("{campaign_id}", "camp_456").replace("{slug}", "my-plugin")
                        ),
                    },
                    ApiExample {
                        language: "python".to_string(),
                        label: "Python".to_string(),
                        code: format!(
                            "import requests\n\nresponse = requests.{}(\n    '{}',\n    headers={{'Authorization': 'Bearer YOUR_API_KEY'}}\n)\nprint(response.json())",
                            method.to_lowercase(),
                            path.replace("{user_id}", "user_123").replace("{campaign_id}", "camp_456").replace("{slug}", "my-plugin")
                        ),
                    },
                ],
            };
            self.endpoints.insert(endpoint.id, endpoint);
        }
    }

    pub fn list_endpoints(&self) -> Vec<ApiEndpoint> {
        self.endpoints.iter().map(|e| e.value().clone()).collect()
    }

    pub fn get_endpoint(&self, id: &Uuid) -> Option<ApiEndpoint> {
        self.endpoints.get(id).map(|e| e.clone())
    }

    pub fn search_endpoints(&self, query: &str) -> Vec<ApiEndpoint> {
        let q = query.to_lowercase();
        self.endpoints
            .iter()
            .filter(|e| {
                e.value().summary.to_lowercase().contains(&q)
                    || e.value().path.to_lowercase().contains(&q)
                    || e.value().description.to_lowercase().contains(&q)
            })
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<ApiEndpoint> {
        self.endpoints
            .iter()
            .filter(|e| e.value().tags.iter().any(|t| t == tag))
            .map(|e| e.value().clone())
            .collect()
    }
}

impl Default for ApiReferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}
