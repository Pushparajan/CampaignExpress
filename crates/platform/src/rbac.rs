//! Role-Based Access Control (RBAC) engine with hierarchical permissions.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// Fine-grained permission for platform resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    CampaignRead,
    CampaignWrite,
    CampaignDelete,
    CreativeRead,
    CreativeWrite,
    CreativeDelete,
    JourneyRead,
    JourneyWrite,
    ExperimentRead,
    ExperimentWrite,
    DcoRead,
    DcoWrite,
    CdpRead,
    CdpWrite,
    AnalyticsRead,
    BillingRead,
    BillingWrite,
    UserManage,
    TenantAdmin,
    SystemAdmin,
}

impl Permission {
    /// All permission variants.
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::CampaignRead,
            Permission::CampaignWrite,
            Permission::CampaignDelete,
            Permission::CreativeRead,
            Permission::CreativeWrite,
            Permission::CreativeDelete,
            Permission::JourneyRead,
            Permission::JourneyWrite,
            Permission::ExperimentRead,
            Permission::ExperimentWrite,
            Permission::DcoRead,
            Permission::DcoWrite,
            Permission::CdpRead,
            Permission::CdpWrite,
            Permission::AnalyticsRead,
            Permission::BillingRead,
            Permission::BillingWrite,
            Permission::UserManage,
            Permission::TenantAdmin,
            Permission::SystemAdmin,
        ]
    }
}

/// A named role containing a set of permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// RBAC engine backed by DashMap stores.
pub struct RbacEngine {
    roles: DashMap<Uuid, Role>,
    user_roles: DashMap<Uuid, Vec<Uuid>>,
}

impl Default for RbacEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RbacEngine {
    /// Create a new, empty RBAC engine.
    pub fn new() -> Self {
        Self {
            roles: DashMap::new(),
            user_roles: DashMap::new(),
        }
    }

    /// Create a new role and return it.
    pub fn create_role(
        &self,
        name: String,
        description: String,
        permissions: Vec<Permission>,
        is_system: bool,
    ) -> Role {
        let role = Role {
            id: Uuid::new_v4(),
            name,
            description,
            permissions,
            is_system,
            created_at: Utc::now(),
        };
        info!(role_id = %role.id, role_name = %role.name, "Role created");
        self.roles.insert(role.id, role.clone());
        role
    }

    /// Assign an existing role to a user. Returns `true` when newly assigned.
    pub fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> bool {
        if !self.roles.contains_key(&role_id) {
            return false;
        }
        let mut entry = self.user_roles.entry(user_id).or_default();
        if entry.contains(&role_id) {
            return false;
        }
        entry.push(role_id);
        info!(user_id = %user_id, role_id = %role_id, "Role assigned");
        true
    }

    /// Remove a role from a user. Returns `true` when the role was actually removed.
    pub fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> bool {
        if let Some(mut entry) = self.user_roles.get_mut(&user_id) {
            let before = entry.len();
            entry.retain(|r| *r != role_id);
            let removed = entry.len() < before;
            if removed {
                info!(user_id = %user_id, role_id = %role_id, "Role revoked");
            }
            removed
        } else {
            false
        }
    }

    /// Check whether a user holds a specific permission through any assigned role.
    pub fn check_permission(&self, user_id: Uuid, permission: &Permission) -> bool {
        if let Some(role_ids) = self.user_roles.get(&user_id) {
            for role_id in role_ids.iter() {
                if let Some(role) = self.roles.get(role_id) {
                    if role.permissions.contains(permission) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Collect every permission a user holds (deduplicated).
    pub fn get_user_permissions(&self, user_id: Uuid) -> Vec<Permission> {
        let mut perms = Vec::new();
        if let Some(role_ids) = self.user_roles.get(&user_id) {
            for role_id in role_ids.iter() {
                if let Some(role) = self.roles.get(role_id) {
                    for p in &role.permissions {
                        if !perms.contains(p) {
                            perms.push(p.clone());
                        }
                    }
                }
            }
        }
        perms
    }

    /// List all defined roles.
    pub fn list_roles(&self) -> Vec<Role> {
        self.roles.iter().map(|e| e.value().clone()).collect()
    }

    /// Seed the four default system roles:
    /// Admin, Campaign Manager, Analyst, Viewer.
    pub fn seed_default_roles(&self) {
        // Admin -- every permission
        self.create_role(
            "Admin".into(),
            "Full system administrator".into(),
            Permission::all(),
            true,
        );

        // Campaign Manager
        self.create_role(
            "Campaign Manager".into(),
            "Manages campaigns, creatives, and journeys with analytics access".into(),
            vec![
                Permission::CampaignRead,
                Permission::CampaignWrite,
                Permission::CampaignDelete,
                Permission::CreativeRead,
                Permission::CreativeWrite,
                Permission::CreativeDelete,
                Permission::JourneyRead,
                Permission::JourneyWrite,
                Permission::AnalyticsRead,
            ],
            true,
        );

        // Analyst
        self.create_role(
            "Analyst".into(),
            "Read-only analytics and experiment access".into(),
            vec![
                Permission::AnalyticsRead,
                Permission::ExperimentRead,
                Permission::ExperimentWrite,
            ],
            true,
        );

        // Viewer
        self.create_role(
            "Viewer".into(),
            "Read-only access to all resources".into(),
            vec![
                Permission::CampaignRead,
                Permission::CreativeRead,
                Permission::JourneyRead,
                Permission::ExperimentRead,
                Permission::DcoRead,
                Permission::CdpRead,
                Permission::AnalyticsRead,
                Permission::BillingRead,
            ],
            true,
        );

        info!("Default RBAC roles seeded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_assignment() {
        let engine = RbacEngine::new();
        let role = engine.create_role(
            "Editor".into(),
            "Can edit campaigns".into(),
            vec![Permission::CampaignRead, Permission::CampaignWrite],
            false,
        );

        let user_id = Uuid::new_v4();
        assert!(engine.assign_role(user_id, role.id));
        // Duplicate assignment returns false.
        assert!(!engine.assign_role(user_id, role.id));

        // Revoke
        assert!(engine.revoke_role(user_id, role.id));
        assert!(!engine.revoke_role(user_id, role.id));
    }

    #[test]
    fn test_permission_check() {
        let engine = RbacEngine::new();
        let role = engine.create_role(
            "Analyst".into(),
            "Read analytics".into(),
            vec![Permission::AnalyticsRead, Permission::ExperimentRead],
            false,
        );

        let user_id = Uuid::new_v4();
        engine.assign_role(user_id, role.id);

        assert!(engine.check_permission(user_id, &Permission::AnalyticsRead));
        assert!(engine.check_permission(user_id, &Permission::ExperimentRead));
        assert!(!engine.check_permission(user_id, &Permission::CampaignWrite));

        let perms = engine.get_user_permissions(user_id);
        assert_eq!(perms.len(), 2);
    }
}
