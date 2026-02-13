//! Task management adaptors — connectors for Asana and Jira.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskProvider {
    Asana,
    Jira,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskManagementConfig {
    pub provider: TaskProvider,
    pub api_base_url: String,
    pub api_token: String,
    pub project_id: String,
    pub workspace_id: Option<String>,
    pub field_mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalTask {
    pub id: String,
    pub provider: TaskProvider,
    pub external_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: TaskPriority,
    pub labels: Vec<String>,
    pub campaign_id: Option<Uuid>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSyncResult {
    pub synced: u32,
    pub created: u32,
    pub updated: u32,
    pub errors: Vec<String>,
    pub synced_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Adaptor
// ---------------------------------------------------------------------------

pub struct TaskManagementAdaptor {
    configs: DashMap<String, TaskManagementConfig>,
    tasks: DashMap<String, ExternalTask>,
    campaign_tasks: DashMap<Uuid, Vec<String>>,
}

impl TaskManagementAdaptor {
    pub fn new() -> Self {
        Self {
            configs: DashMap::new(),
            tasks: DashMap::new(),
            campaign_tasks: DashMap::new(),
        }
    }

    /// Register a named provider configuration.
    pub fn register_provider(&self, name: &str, config: TaskManagementConfig) {
        tracing::info!(provider = name, "Registering task management provider");
        self.configs.insert(name.to_string(), config);
    }

    /// Simulate creating a task via the external provider API.
    pub fn create_task(
        &self,
        provider_name: &str,
        title: &str,
        description: &str,
        campaign_id: Option<Uuid>,
        priority: TaskPriority,
    ) -> Option<ExternalTask> {
        let config = self.configs.get(provider_name)?;
        let task_id = Uuid::new_v4().to_string();
        let external_num = self.tasks.len() as u64 + 1;
        let external_id = format!("{external_num}");

        let url = match config.provider {
            TaskProvider::Asana => {
                format!("https://app.asana.com/0/{}/{}", config.project_id, task_id)
            }
            TaskProvider::Jira => {
                format!(
                    "{}/browse/{}-{}",
                    config.api_base_url, config.project_id, external_num
                )
            }
        };

        let now = Utc::now();
        let task = ExternalTask {
            id: task_id.clone(),
            provider: config.provider.clone(),
            external_id,
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Todo,
            assignee: None,
            due_date: None,
            priority,
            labels: Vec::new(),
            campaign_id,
            custom_fields: HashMap::new(),
            created_at: now,
            updated_at: now,
            url,
        };

        self.tasks.insert(task_id.clone(), task.clone());

        if let Some(cid) = campaign_id {
            self.campaign_tasks.entry(cid).or_default().push(task_id);
        }

        tracing::info!(
            provider = provider_name,
            task = &task.id,
            "Created external task"
        );
        Some(task)
    }

    /// Update the status of an existing task.
    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) -> bool {
        if let Some(mut entry) = self.tasks.get_mut(task_id) {
            entry.status = status;
            entry.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Retrieve a single task by ID.
    pub fn get_task(&self, task_id: &str) -> Option<ExternalTask> {
        self.tasks.get(task_id).map(|t| t.clone())
    }

    /// List all tasks linked to a campaign.
    pub fn list_campaign_tasks(&self, campaign_id: &Uuid) -> Vec<ExternalTask> {
        self.campaign_tasks
            .get(campaign_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.tasks.get(id).map(|t| t.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Simulate pulling tasks from an external system.
    pub fn sync_from_provider(&self, provider_name: &str) -> TaskSyncResult {
        let config = match self.configs.get(provider_name) {
            Some(c) => c.clone(),
            None => {
                return TaskSyncResult {
                    synced: 0,
                    created: 0,
                    updated: 0,
                    errors: vec![format!("Provider '{}' not found", provider_name)],
                    synced_at: Utc::now(),
                };
            }
        };

        let mut created = 0u32;
        let titles = [
            "Update landing page copy",
            "Design banner assets",
            "Configure audience segments",
            "Set up A/B test variants",
            "Review analytics dashboard",
        ];

        for title in &titles {
            let task_id = Uuid::new_v4().to_string();
            let external_num = self.tasks.len() as u64 + 1;

            let url = match config.provider {
                TaskProvider::Asana => {
                    format!("https://app.asana.com/0/{}/{}", config.project_id, task_id)
                }
                TaskProvider::Jira => {
                    format!(
                        "{}/browse/{}-{}",
                        config.api_base_url, config.project_id, external_num
                    )
                }
            };

            let now = Utc::now();
            let task = ExternalTask {
                id: task_id.clone(),
                provider: config.provider.clone(),
                external_id: format!("{external_num}"),
                title: title.to_string(),
                description: format!("Synced task: {title}"),
                status: TaskStatus::Todo,
                assignee: None,
                due_date: None,
                priority: TaskPriority::Medium,
                labels: vec!["synced".to_string()],
                campaign_id: None,
                custom_fields: HashMap::new(),
                created_at: now,
                updated_at: now,
                url,
            };

            self.tasks.insert(task_id, task);
            created += 1;
        }

        tracing::info!(
            provider = provider_name,
            created,
            "Synced tasks from external provider"
        );

        TaskSyncResult {
            synced: created,
            created,
            updated: 0,
            errors: Vec::new(),
            synced_at: Utc::now(),
        }
    }

    /// Create the four standard review tasks for a campaign.
    pub fn create_campaign_review_tasks(
        &self,
        campaign_id: Uuid,
        campaign_name: &str,
        provider_name: &str,
    ) -> Vec<ExternalTask> {
        let review_titles = [
            "Review creative assets",
            "Approve targeting",
            "Legal compliance check",
            "Final sign-off",
        ];

        let mut tasks = Vec::new();
        for title in &review_titles {
            let description = format!("{title} for campaign \"{campaign_name}\"");
            if let Some(task) = self.create_task(
                provider_name,
                title,
                &description,
                Some(campaign_id),
                TaskPriority::High,
            ) {
                tasks.push(task);
            }
        }
        tasks
    }
}

impl Default for TaskManagementAdaptor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn jira_config() -> TaskManagementConfig {
        TaskManagementConfig {
            provider: TaskProvider::Jira,
            api_base_url: "https://myteam.atlassian.net".to_string(),
            api_token: "test-token".to_string(),
            project_id: "CAMP".to_string(),
            workspace_id: None,
            field_mappings: HashMap::new(),
        }
    }

    fn asana_config() -> TaskManagementConfig {
        TaskManagementConfig {
            provider: TaskProvider::Asana,
            api_base_url: "https://app.asana.com/api/1.0".to_string(),
            api_token: "test-token".to_string(),
            project_id: "123456".to_string(),
            workspace_id: Some("ws-1".to_string()),
            field_mappings: HashMap::new(),
        }
    }

    #[test]
    fn test_create_task() {
        let adaptor = TaskManagementAdaptor::new();
        adaptor.register_provider("jira", jira_config());

        let task = adaptor
            .create_task(
                "jira",
                "Test task",
                "A description",
                None,
                TaskPriority::High,
            )
            .expect("task should be created");

        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, TaskPriority::High);
        assert!(task.url.contains("atlassian.net/browse/CAMP-"));

        // Asana URL format
        let adaptor2 = TaskManagementAdaptor::new();
        adaptor2.register_provider("asana", asana_config());
        let task2 = adaptor2
            .create_task("asana", "Asana task", "desc", None, TaskPriority::Low)
            .expect("asana task");
        assert!(task2.url.starts_with("https://app.asana.com/0/123456/"));
    }

    #[test]
    fn test_update_status() {
        let adaptor = TaskManagementAdaptor::new();
        adaptor.register_provider("jira", jira_config());

        let task = adaptor
            .create_task("jira", "Status test", "desc", None, TaskPriority::Medium)
            .unwrap();

        assert!(adaptor.update_task_status(&task.id, TaskStatus::InProgress));
        let updated = adaptor.get_task(&task.id).unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);

        assert!(!adaptor.update_task_status("nonexistent", TaskStatus::Done));
    }

    #[test]
    fn test_campaign_task_linking() {
        let adaptor = TaskManagementAdaptor::new();
        adaptor.register_provider("jira", jira_config());

        let cid = Uuid::new_v4();
        adaptor.create_task("jira", "Task A", "a", Some(cid), TaskPriority::Low);
        adaptor.create_task("jira", "Task B", "b", Some(cid), TaskPriority::High);
        adaptor.create_task("jira", "Unlinked", "c", None, TaskPriority::None);

        let campaign_tasks = adaptor.list_campaign_tasks(&cid);
        assert_eq!(campaign_tasks.len(), 2);

        // Review tasks
        let review = adaptor.create_campaign_review_tasks(cid, "Summer Sale", "jira");
        assert_eq!(review.len(), 4);

        let all = adaptor.list_campaign_tasks(&cid);
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_sync_from_provider() {
        let adaptor = TaskManagementAdaptor::new();
        adaptor.register_provider("jira", jira_config());

        let result = adaptor.sync_from_provider("jira");
        assert_eq!(result.synced, 5);
        assert_eq!(result.created, 5);
        assert!(result.errors.is_empty());

        // Unknown provider returns error
        let bad = adaptor.sync_from_provider("unknown");
        assert_eq!(bad.synced, 0);
        assert_eq!(bad.errors.len(), 1);
    }
}
