//! User management — CRUD, team invitations, role assignment, session oversight.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use campaign_platform::auth::{AuthManager, AuthProvider, AuthSession, AuthToken};
use campaign_platform::rbac::{Permission, RbacEngine, Role};

/// User record managed by the admin console.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub status: UserStatus,
    pub auth_provider: AuthProvider,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// User account status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Invited,
    Disabled,
    Locked,
}

/// Pending team invitation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role_id: Uuid,
    pub invited_by: Uuid,
    pub status: InvitationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Invitation lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

/// User management operations.
pub struct UserOps<'a> {
    users: DashMap<Uuid, User>,
    invitations: DashMap<Uuid, Invitation>,
    auth: &'a AuthManager,
    rbac: &'a RbacEngine,
}

impl<'a> UserOps<'a> {
    pub fn new(auth: &'a AuthManager, rbac: &'a RbacEngine) -> Self {
        Self {
            users: DashMap::new(),
            invitations: DashMap::new(),
            auth,
            rbac,
        }
    }

    /// Create a user account.
    pub fn create_user(
        &self,
        tenant_id: Uuid,
        email: String,
        display_name: String,
        provider: AuthProvider,
    ) -> User {
        let user = User {
            id: Uuid::new_v4(),
            tenant_id,
            email,
            display_name: display_name.clone(),
            status: UserStatus::Active,
            auth_provider: provider,
            created_at: Utc::now(),
            last_login: None,
        };
        info!(user_id = %user.id, name = %display_name, "User created");
        self.users.insert(user.id, user.clone());
        user
    }

    /// Get a user by ID.
    pub fn get_user(&self, user_id: Uuid) -> Option<User> {
        self.users.get(&user_id).map(|e| e.value().clone())
    }

    /// List all users for a tenant.
    pub fn list_users(&self, tenant_id: Uuid) -> Vec<User> {
        let mut users: Vec<_> = self
            .users
            .iter()
            .filter(|e| e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect();
        users.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        users
    }

    /// Disable a user account (revoke all sessions).
    pub fn disable_user(&self, user_id: Uuid) -> anyhow::Result<User> {
        let mut entry = self
            .users
            .get_mut(&user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found: {user_id}"))?;
        entry.status = UserStatus::Disabled;

        // Revoke all active sessions
        let sessions = self.auth.list_active_sessions(user_id);
        for session in &sessions {
            self.auth.revoke_session(session.session_id);
        }
        info!(user_id = %user_id, sessions_revoked = sessions.len(), "User disabled");
        Ok(entry.clone())
    }

    /// Re-enable a disabled user.
    pub fn enable_user(&self, user_id: Uuid) -> anyhow::Result<User> {
        let mut entry = self
            .users
            .get_mut(&user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found: {user_id}"))?;
        entry.status = UserStatus::Active;
        info!(user_id = %user_id, "User re-enabled");
        Ok(entry.clone())
    }

    /// Assign a role to a user.
    pub fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> bool {
        self.rbac.assign_role(user_id, role_id)
    }

    /// Revoke a role from a user.
    pub fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> bool {
        self.rbac.revoke_role(user_id, role_id)
    }

    /// Get all permissions for a user.
    pub fn get_user_permissions(&self, user_id: Uuid) -> Vec<Permission> {
        self.rbac.get_user_permissions(user_id)
    }

    /// List available roles.
    pub fn list_roles(&self) -> Vec<Role> {
        self.rbac.list_roles()
    }

    /// Create a team invitation.
    pub fn invite_user(
        &self,
        tenant_id: Uuid,
        email: String,
        role_id: Uuid,
        invited_by: Uuid,
    ) -> Invitation {
        let invitation = Invitation {
            id: Uuid::new_v4(),
            tenant_id,
            email: email.clone(),
            role_id,
            invited_by,
            status: InvitationStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };
        info!(invitation_id = %invitation.id, email = %email, "Invitation sent");
        self.invitations.insert(invitation.id, invitation.clone());
        invitation
    }

    /// Accept an invitation — creates the user and assigns the role.
    pub fn accept_invitation(
        &self,
        invitation_id: Uuid,
        display_name: String,
    ) -> anyhow::Result<User> {
        let mut inv = self
            .invitations
            .get_mut(&invitation_id)
            .ok_or_else(|| anyhow::anyhow!("Invitation not found"))?;

        if inv.status != InvitationStatus::Pending {
            return Err(anyhow::anyhow!("Invitation is {:?}", inv.status));
        }
        if Utc::now() > inv.expires_at {
            inv.status = InvitationStatus::Expired;
            return Err(anyhow::anyhow!("Invitation has expired"));
        }

        inv.status = InvitationStatus::Accepted;
        let user = self.create_user(
            inv.tenant_id,
            inv.email.clone(),
            display_name,
            AuthProvider::Local,
        );
        self.rbac.assign_role(user.id, inv.role_id);
        Ok(user)
    }

    /// List pending invitations for a tenant.
    pub fn list_invitations(&self, tenant_id: Uuid) -> Vec<Invitation> {
        self.invitations
            .iter()
            .filter(|e| e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Get all active sessions for a user.
    pub fn get_user_sessions(&self, user_id: Uuid) -> Vec<AuthSession> {
        self.auth.list_active_sessions(user_id)
    }

    /// Force-revoke a specific session (admin action).
    pub fn revoke_session(&self, session_id: Uuid) -> bool {
        self.auth.revoke_session(session_id)
    }

    /// Generate an API key for a user.
    pub fn generate_api_key(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        roles: Vec<String>,
    ) -> AuthToken {
        self.auth.generate_api_key(user_id, tenant_id, roles)
    }

    /// Seed demo users for a tenant.
    pub fn seed_demo_users(&self, tenant_id: Uuid) {
        let admin = self.create_user(
            tenant_id,
            "admin@example.com".into(),
            "Platform Admin".into(),
            AuthProvider::Local,
        );
        self.create_user(
            tenant_id,
            "manager@example.com".into(),
            "Campaign Manager".into(),
            AuthProvider::OAuth2,
        );
        self.create_user(
            tenant_id,
            "analyst@example.com".into(),
            "Data Analyst".into(),
            AuthProvider::OAuth2,
        );
        self.create_user(
            tenant_id,
            "viewer@example.com".into(),
            "Read-Only Viewer".into(),
            AuthProvider::Local,
        );

        // Assign roles if seeded
        let roles = self.rbac.list_roles();
        if let Some(admin_role) = roles.iter().find(|r| r.name == "Admin") {
            self.rbac.assign_role(admin.id, admin_role.id);
        }
        info!(tenant_id = %tenant_id, "Demo users seeded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (AuthManager, RbacEngine) {
        let auth = AuthManager::new();
        let rbac = RbacEngine::new();
        rbac.seed_default_roles();
        (auth, rbac)
    }

    #[test]
    fn test_create_and_list_users() {
        let (auth, rbac) = setup();
        let ops = UserOps::new(&auth, &rbac);
        let tenant_id = Uuid::new_v4();

        ops.create_user(
            tenant_id,
            "alice@test.com".into(),
            "Alice".into(),
            AuthProvider::Local,
        );
        ops.create_user(
            tenant_id,
            "bob@test.com".into(),
            "Bob".into(),
            AuthProvider::Local,
        );

        let users = ops.list_users(tenant_id);
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].display_name, "Alice");
        assert_eq!(users[1].display_name, "Bob");
    }

    #[test]
    fn test_disable_and_enable_user() {
        let (auth, rbac) = setup();
        let ops = UserOps::new(&auth, &rbac);
        let tenant_id = Uuid::new_v4();

        let user = ops.create_user(
            tenant_id,
            "alice@test.com".into(),
            "Alice".into(),
            AuthProvider::Local,
        );

        // Create a session for the user
        auth.create_session(
            user.id,
            tenant_id,
            AuthProvider::Local,
            vec!["admin".into()],
            "127.0.0.1".into(),
            "test".into(),
        );
        assert_eq!(auth.list_active_sessions(user.id).len(), 1);

        // Disable revokes sessions
        let disabled = ops.disable_user(user.id).unwrap();
        assert_eq!(disabled.status, UserStatus::Disabled);
        assert_eq!(auth.list_active_sessions(user.id).len(), 0);

        // Re-enable
        let enabled = ops.enable_user(user.id).unwrap();
        assert_eq!(enabled.status, UserStatus::Active);
    }

    #[test]
    fn test_invitation_flow() {
        let (auth, rbac) = setup();
        let ops = UserOps::new(&auth, &rbac);
        let tenant_id = Uuid::new_v4();

        let roles = rbac.list_roles();
        let viewer_role = roles.iter().find(|r| r.name == "Viewer").unwrap();

        let inv = ops.invite_user(
            tenant_id,
            "new@test.com".into(),
            viewer_role.id,
            Uuid::new_v4(),
        );
        assert_eq!(inv.status, InvitationStatus::Pending);

        let user = ops.accept_invitation(inv.id, "New User".into()).unwrap();
        assert_eq!(user.email, "new@test.com");
        assert_eq!(user.tenant_id, tenant_id);

        // User should have Viewer permissions
        let perms = ops.get_user_permissions(user.id);
        assert!(perms.contains(&Permission::AnalyticsRead));
        assert!(!perms.contains(&Permission::SystemAdmin));
    }

    #[test]
    fn test_double_accept_fails() {
        let (auth, rbac) = setup();
        let ops = UserOps::new(&auth, &rbac);
        let roles = rbac.list_roles();
        let role = &roles[0];

        let inv = ops.invite_user(Uuid::new_v4(), "x@test.com".into(), role.id, Uuid::new_v4());
        ops.accept_invitation(inv.id, "User".into()).unwrap();
        assert!(ops.accept_invitation(inv.id, "User2".into()).is_err());
    }
}
