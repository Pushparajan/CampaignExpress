//! Plugin store — installation, configuration, and lifecycle management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationStatus {
    Installing,
    Active,
    Disabled,
    Error,
    Uninstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallation {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub plugin_slug: String,
    pub workspace_id: Uuid,
    pub installed_by: Uuid,
    pub version: String,
    pub config: serde_json::Value,
    pub status: InstallationStatus,
    pub error_message: Option<String>,
    pub error_count: u32,
    pub installed_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    pub plugin_id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub config: serde_json::Value,
    pub accepted_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAnalytics {
    pub plugin_id: Uuid,
    pub total_installs: u64,
    pub active_installs: u64,
    pub total_api_calls: u64,
    pub error_rate: f64,
    pub avg_response_ms: f64,
}

pub struct PluginStore {
    installations: dashmap::DashMap<Uuid, PluginInstallation>,
}

impl PluginStore {
    pub fn new() -> Self {
        Self {
            installations: dashmap::DashMap::new(),
        }
    }

    pub fn install(&self, request: InstallRequest) -> anyhow::Result<PluginInstallation> {
        if !request.accepted_permissions {
            anyhow::bail!("Must accept plugin permissions before installation");
        }

        let now = Utc::now();
        let installation = PluginInstallation {
            id: Uuid::new_v4(),
            plugin_id: request.plugin_id,
            plugin_slug: String::new(),
            workspace_id: request.workspace_id,
            installed_by: request.user_id,
            version: "1.0.0".to_string(),
            config: request.config,
            status: InstallationStatus::Active,
            error_message: None,
            error_count: 0,
            installed_at: now,
            last_used: None,
            updated_at: now,
        };

        self.installations
            .insert(installation.id, installation.clone());
        tracing::info!(
            plugin_id = %request.plugin_id,
            workspace_id = %request.workspace_id,
            "Plugin installed successfully"
        );
        Ok(installation)
    }

    pub fn uninstall(&self, installation_id: &Uuid) -> bool {
        if let Some(mut inst) = self.installations.get_mut(installation_id) {
            inst.status = InstallationStatus::Uninstalled;
            inst.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn update_config(&self, installation_id: &Uuid, config: serde_json::Value) -> bool {
        if let Some(mut inst) = self.installations.get_mut(installation_id) {
            inst.config = config;
            inst.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn toggle_status(&self, installation_id: &Uuid, enabled: bool) -> bool {
        if let Some(mut inst) = self.installations.get_mut(installation_id) {
            inst.status = if enabled {
                InstallationStatus::Active
            } else {
                InstallationStatus::Disabled
            };
            inst.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn list_installed(&self, workspace_id: &Uuid) -> Vec<PluginInstallation> {
        self.installations
            .iter()
            .filter(|i| {
                &i.value().workspace_id == workspace_id
                    && !matches!(i.value().status, InstallationStatus::Uninstalled)
            })
            .map(|i| i.value().clone())
            .collect()
    }

    pub fn get_installation(&self, id: &Uuid) -> Option<PluginInstallation> {
        self.installations.get(id).map(|i| i.clone())
    }
}

impl Default for PluginStore {
    fn default() -> Self {
        Self::new()
    }
}
