//! Authentication: OAuth2, SAML, API key, and local session management.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

/// Authentication provider type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    Local,
    OAuth2,
    Saml,
    ApiKey,
}

/// OAuth2 provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub provider_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

/// SAML identity-provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    pub idp_entity_id: String,
    pub idp_sso_url: String,
    pub idp_certificate: String,
    pub sp_entity_id: String,
    pub sp_acs_url: String,
}

/// Bearer / API-key token issued after authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub provider: AuthProvider,
    pub roles: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

/// A user session backed by an auth token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: Uuid,
    pub token: AuthToken,
    pub refresh_token: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

/// Central authentication manager holding sessions and provider configs.
pub struct AuthManager {
    sessions: DashMap<Uuid, AuthSession>,
    oauth2_providers: DashMap<String, OAuth2Config>,
    saml_providers: DashMap<String, SamlConfig>,
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthManager {
    /// Create a new, empty `AuthManager`.
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            oauth2_providers: DashMap::new(),
            saml_providers: DashMap::new(),
        }
    }

    /// Register an OAuth2 identity provider.
    pub fn register_oauth2_provider(&self, name: String, config: OAuth2Config) {
        info!(provider = %name, "Registered OAuth2 provider");
        self.oauth2_providers.insert(name, config);
    }

    /// Register a SAML identity provider.
    pub fn register_saml_provider(&self, name: String, config: SamlConfig) {
        info!(provider = %name, "Registered SAML provider");
        self.saml_providers.insert(name, config);
    }

    /// Create a new authenticated session and return it.
    pub fn create_session(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        provider: AuthProvider,
        roles: Vec<String>,
        ip: String,
        user_agent: String,
    ) -> AuthSession {
        let now = Utc::now();
        let token = AuthToken {
            token_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            provider,
            roles,
            issued_at: now,
            expires_at: now + Duration::hours(8),
            scopes: vec!["read".into(), "write".into()],
        };

        let session = AuthSession {
            session_id: Uuid::new_v4(),
            token,
            refresh_token: Some(Uuid::new_v4().to_string()),
            ip_address: ip,
            user_agent,
            created_at: now,
            last_active: now,
        };

        info!(
            session_id = %session.session_id,
            user_id = %user_id,
            "Session created"
        );
        self.sessions.insert(session.session_id, session.clone());
        session
    }

    /// Validate a token by its id; returns `None` when expired or missing.
    pub fn validate_token(&self, token_id: Uuid) -> Option<AuthToken> {
        for entry in self.sessions.iter() {
            if entry.value().token.token_id == token_id {
                let token = &entry.value().token;
                if Utc::now() < token.expires_at {
                    return Some(token.clone());
                }
                return None; // expired
            }
        }
        None
    }

    /// Revoke (delete) a session. Returns `true` when the session existed.
    pub fn revoke_session(&self, session_id: Uuid) -> bool {
        let removed = self.sessions.remove(&session_id).is_some();
        if removed {
            info!(session_id = %session_id, "Session revoked");
        }
        removed
    }

    /// List all active (non-expired) sessions for a given user.
    pub fn list_active_sessions(&self, user_id: Uuid) -> Vec<AuthSession> {
        let now = Utc::now();
        self.sessions
            .iter()
            .filter(|e| e.value().token.user_id == user_id && now < e.value().token.expires_at)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Generate a long-lived API key token (1 year).
    pub fn generate_api_key(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        roles: Vec<String>,
    ) -> AuthToken {
        let now = Utc::now();
        let token = AuthToken {
            token_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            provider: AuthProvider::ApiKey,
            roles,
            issued_at: now,
            expires_at: now + Duration::days(365),
            scopes: vec!["api".into()],
        };

        // Store in a pseudo-session so validate_token still works.
        let session = AuthSession {
            session_id: Uuid::new_v4(),
            token: token.clone(),
            refresh_token: None,
            ip_address: "api-key".into(),
            user_agent: "api-key".into(),
            created_at: now,
            last_active: now,
        };
        self.sessions.insert(session.session_id, session);

        info!(token_id = %token.token_id, user_id = %user_id, "API key generated");
        token
    }

    /// Seed demo OAuth2 ("okta") and SAML ("azure_ad") providers.
    pub fn seed_demo_providers(&self) {
        self.register_oauth2_provider(
            "okta".into(),
            OAuth2Config {
                provider_name: "Okta".into(),
                client_id: "demo-client-id".into(),
                client_secret: "demo-client-secret".into(),
                auth_url: "https://dev-123456.okta.com/oauth2/v1/authorize".into(),
                token_url: "https://dev-123456.okta.com/oauth2/v1/token".into(),
                scopes: vec!["openid".into(), "profile".into(), "email".into()],
                redirect_uri: "https://app.campaignexpress.io/auth/callback".into(),
            },
        );

        self.register_saml_provider(
            "azure_ad".into(),
            SamlConfig {
                idp_entity_id: "https://sts.windows.net/demo-tenant-id/".into(),
                idp_sso_url: "https://login.microsoftonline.com/demo-tenant-id/saml2".into(),
                idp_certificate: "MIIC8DCC...demo-cert...".into(),
                sp_entity_id: "https://app.campaignexpress.io".into(),
                sp_acs_url: "https://app.campaignexpress.io/auth/saml/acs".into(),
            },
        );

        info!("Demo auth providers seeded (okta, azure_ad)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_session() {
        let mgr = AuthManager::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let session = mgr.create_session(
            user_id,
            tenant_id,
            AuthProvider::OAuth2,
            vec!["admin".into()],
            "127.0.0.1".into(),
            "test-agent".into(),
        );

        // Token should be valid.
        let token = mgr.validate_token(session.token.token_id);
        assert!(token.is_some());
        let token = token.unwrap();
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.tenant_id, tenant_id);
        assert_eq!(token.provider, AuthProvider::OAuth2);
        assert_eq!(token.roles, vec!["admin".to_string()]);

        // Active sessions list should contain this session.
        let active = mgr.list_active_sessions(user_id);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].session_id, session.session_id);
    }

    #[test]
    fn test_revoke_session() {
        let mgr = AuthManager::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let session = mgr.create_session(
            user_id,
            tenant_id,
            AuthProvider::Local,
            vec!["viewer".into()],
            "10.0.0.1".into(),
            "browser".into(),
        );

        assert!(mgr.revoke_session(session.session_id));
        // Token should no longer validate.
        assert!(mgr.validate_token(session.token.token_id).is_none());
        // Revoking again returns false.
        assert!(!mgr.revoke_session(session.session_id));
    }
}
