//! Plugin sandboxing — security isolation, permission enforcement, and resource limits.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_cpu_cores: u32,
    pub max_memory_mb: u64,
    pub max_disk_mb: u64,
    pub max_network_requests_per_min: u32,
    pub allowed_outbound_domains: Vec<String>,
    pub filesystem_read_only: bool,
    pub temp_directory_mb: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_cpu_cores: 1,
            max_memory_mb: 512,
            max_disk_mb: 100,
            max_network_requests_per_min: 1000,
            allowed_outbound_domains: Vec::new(),
            filesystem_read_only: true,
            temp_directory_mb: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScope {
    pub scope: String,
    pub description: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub workspace_id: Uuid,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub result: AuditResult,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Allowed,
    Denied,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub plugin_id: Uuid,
    pub version: String,
    pub passed: bool,
    pub findings: Vec<SecurityFinding>,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: FindingSeverity,
    pub category: String,
    pub description: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

pub struct PluginSandbox {
    granted_permissions: dashmap::DashMap<(Uuid, Uuid), Vec<String>>,
    audit_log: dashmap::DashMap<Uuid, Vec<AuditEntry>>,
    scan_results: dashmap::DashMap<Uuid, SecurityScanResult>,
}

impl PluginSandbox {
    pub fn new() -> Self {
        Self {
            granted_permissions: dashmap::DashMap::new(),
            audit_log: dashmap::DashMap::new(),
            scan_results: dashmap::DashMap::new(),
        }
    }

    pub fn grant_permissions(&self, plugin_id: Uuid, workspace_id: Uuid, scopes: Vec<String>) {
        self.granted_permissions
            .insert((plugin_id, workspace_id), scopes);
    }

    pub fn check_permission(&self, plugin_id: &Uuid, workspace_id: &Uuid, scope: &str) -> bool {
        self.granted_permissions
            .get(&(*plugin_id, *workspace_id))
            .map(|perms| perms.iter().any(|p| p == scope || p == "*"))
            .unwrap_or(false)
    }

    pub fn log_access(
        &self,
        plugin_id: Uuid,
        workspace_id: Uuid,
        action: String,
        resource: String,
        resource_id: Option<String>,
        allowed: bool,
    ) {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            plugin_id,
            workspace_id,
            action,
            resource,
            resource_id,
            result: if allowed {
                AuditResult::Allowed
            } else {
                AuditResult::Denied
            },
            timestamp: Utc::now(),
        };
        self.audit_log.entry(plugin_id).or_default().push(entry);
    }

    pub fn get_audit_log(&self, plugin_id: &Uuid) -> Vec<AuditEntry> {
        self.audit_log
            .get(plugin_id)
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }

    pub fn record_scan(&self, result: SecurityScanResult) {
        self.scan_results.insert(result.plugin_id, result);
    }

    pub fn get_scan_result(&self, plugin_id: &Uuid) -> Option<SecurityScanResult> {
        self.scan_results.get(plugin_id).map(|r| r.clone())
    }

    pub fn available_scopes() -> Vec<PermissionScope> {
        vec![
            PermissionScope {
                scope: "campaigns.read".to_string(),
                description: "View campaigns".to_string(),
                risk_level: RiskLevel::Low,
            },
            PermissionScope {
                scope: "campaigns.write".to_string(),
                description: "Create/edit campaigns".to_string(),
                risk_level: RiskLevel::Medium,
            },
            PermissionScope {
                scope: "users.read".to_string(),
                description: "View user profiles".to_string(),
                risk_level: RiskLevel::Medium,
            },
            PermissionScope {
                scope: "users.write".to_string(),
                description: "Update user attributes".to_string(),
                risk_level: RiskLevel::High,
            },
            PermissionScope {
                scope: "users.pii".to_string(),
                description: "Access PII (email, phone, name)".to_string(),
                risk_level: RiskLevel::Critical,
            },
            PermissionScope {
                scope: "users.delete".to_string(),
                description: "Delete user data".to_string(),
                risk_level: RiskLevel::Critical,
            },
            PermissionScope {
                scope: "events.read".to_string(),
                description: "Query event data".to_string(),
                risk_level: RiskLevel::Low,
            },
            PermissionScope {
                scope: "events.write".to_string(),
                description: "Track custom events".to_string(),
                risk_level: RiskLevel::Medium,
            },
            PermissionScope {
                scope: "analytics.read".to_string(),
                description: "View reports".to_string(),
                risk_level: RiskLevel::Low,
            },
            PermissionScope {
                scope: "analytics.export".to_string(),
                description: "Export raw data".to_string(),
                risk_level: RiskLevel::High,
            },
            PermissionScope {
                scope: "channels.send".to_string(),
                description: "Send messages".to_string(),
                risk_level: RiskLevel::High,
            },
            PermissionScope {
                scope: "integrations.configure".to_string(),
                description: "Manage integrations".to_string(),
                risk_level: RiskLevel::Medium,
            },
            PermissionScope {
                scope: "network.outbound".to_string(),
                description: "Make HTTP requests to external APIs".to_string(),
                risk_level: RiskLevel::High,
            },
        ]
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}
