//! Global system settings and configuration management for the SaaS platform.
//! Controls cluster-wide defaults, maintenance windows, and operational parameters.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;

/// System-wide configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub platform_name: String,
    pub platform_version: String,
    pub environment: Environment,
    pub maintenance_mode: bool,
    pub maintenance_message: Option<String>,
    pub default_rate_limit_rps: u32,
    pub default_rate_limit_rpm: u32,
    pub max_tenants: u32,
    pub session_ttl_hours: u32,
    pub api_key_ttl_days: u32,
    pub data_retention_days: u32,
    pub allow_self_registration: bool,
    pub require_email_verification: bool,
    pub password_min_length: u8,
    pub mfa_required: bool,
    pub updated_at: DateTime<Utc>,
}

/// Deployment environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            platform_name: "Campaign Express".into(),
            platform_version: "0.1.0".into(),
            environment: Environment::Production,
            maintenance_mode: false,
            maintenance_message: None,
            default_rate_limit_rps: 100,
            default_rate_limit_rpm: 3000,
            max_tenants: 10_000,
            session_ttl_hours: 8,
            api_key_ttl_days: 365,
            data_retention_days: 365,
            allow_self_registration: true,
            require_email_verification: true,
            password_min_length: 12,
            mfa_required: false,
            updated_at: Utc::now(),
        }
    }
}

/// Configuration change audit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub old_value: String,
    pub new_value: String,
    pub changed_by: String,
    pub changed_at: DateTime<Utc>,
}

/// System settings manager.
pub struct SystemSettings {
    config: std::sync::RwLock<SystemConfig>,
    change_log: DashMap<String, Vec<ConfigChange>>,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSettings {
    pub fn new() -> Self {
        Self {
            config: std::sync::RwLock::new(SystemConfig::default()),
            change_log: DashMap::new(),
        }
    }

    /// Get the current system configuration.
    pub fn get_config(&self) -> SystemConfig {
        self.config
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Enable maintenance mode with an optional message.
    pub fn enable_maintenance(&self, message: Option<String>, actor: &str) {
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "maintenance_mode",
            &cfg.maintenance_mode.to_string(),
            "true",
            actor,
        );
        cfg.maintenance_mode = true;
        cfg.maintenance_message = message;
        cfg.updated_at = Utc::now();
        info!(actor = actor, "Maintenance mode enabled");
    }

    /// Disable maintenance mode.
    pub fn disable_maintenance(&self, actor: &str) {
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "maintenance_mode",
            &cfg.maintenance_mode.to_string(),
            "false",
            actor,
        );
        cfg.maintenance_mode = false;
        cfg.maintenance_message = None;
        cfg.updated_at = Utc::now();
        info!(actor = actor, "Maintenance mode disabled");
    }

    /// Update rate limit defaults.
    pub fn set_rate_limits(&self, rps: u32, rpm: u32, actor: &str) {
        if rps == 0 || rpm == 0 {
            return;
        }
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "default_rate_limit_rps",
            &cfg.default_rate_limit_rps.to_string(),
            &rps.to_string(),
            actor,
        );
        cfg.default_rate_limit_rps = rps;
        cfg.default_rate_limit_rpm = rpm;
        cfg.updated_at = Utc::now();
    }

    /// Toggle self-registration.
    pub fn set_self_registration(&self, enabled: bool, actor: &str) {
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "allow_self_registration",
            &cfg.allow_self_registration.to_string(),
            &enabled.to_string(),
            actor,
        );
        cfg.allow_self_registration = enabled;
        cfg.updated_at = Utc::now();
    }

    /// Set MFA requirement.
    pub fn set_mfa_required(&self, required: bool, actor: &str) {
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "mfa_required",
            &cfg.mfa_required.to_string(),
            &required.to_string(),
            actor,
        );
        cfg.mfa_required = required;
        cfg.updated_at = Utc::now();
    }

    /// Set password minimum length.
    pub fn set_password_min_length(&self, length: u8, actor: &str) {
        if length < 4 {
            return;
        }
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "password_min_length",
            &cfg.password_min_length.to_string(),
            &length.to_string(),
            actor,
        );
        cfg.password_min_length = length;
        cfg.updated_at = Utc::now();
    }

    /// Set data retention period.
    pub fn set_data_retention_days(&self, days: u32, actor: &str) {
        if days == 0 {
            return;
        }
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        self.log_change(
            "data_retention_days",
            &cfg.data_retention_days.to_string(),
            &days.to_string(),
            actor,
        );
        cfg.data_retention_days = days;
        cfg.updated_at = Utc::now();
    }

    /// Get the change log for a specific configuration key.
    pub fn get_change_log(&self, key: &str) -> Vec<ConfigChange> {
        self.change_log
            .get(key)
            .map(|e| e.value().clone())
            .unwrap_or_default()
    }

    /// Get the full change log across all keys.
    pub fn full_change_log(&self) -> Vec<ConfigChange> {
        let mut all: Vec<_> = self
            .change_log
            .iter()
            .flat_map(|e| e.value().clone())
            .collect();
        all.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));
        all
    }

    fn log_change(&self, key: &str, old: &str, new: &str, actor: &str) {
        let change = ConfigChange {
            key: key.to_string(),
            old_value: old.to_string(),
            new_value: new.to_string(),
            changed_by: actor.to_string(),
            changed_at: Utc::now(),
        };
        self.change_log
            .entry(key.to_string())
            .or_default()
            .push(change);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let settings = SystemSettings::new();
        let cfg = settings.get_config();
        assert_eq!(cfg.platform_name, "Campaign Express");
        assert!(!cfg.maintenance_mode);
        assert!(cfg.allow_self_registration);
        assert!(!cfg.mfa_required);
    }

    #[test]
    fn test_maintenance_mode() {
        let settings = SystemSettings::new();
        settings.enable_maintenance(Some("Upgrading to v0.2.0".into()), "admin");

        let cfg = settings.get_config();
        assert!(cfg.maintenance_mode);
        assert_eq!(cfg.maintenance_message, Some("Upgrading to v0.2.0".into()));

        settings.disable_maintenance("admin");
        let cfg = settings.get_config();
        assert!(!cfg.maintenance_mode);
        assert!(cfg.maintenance_message.is_none());
    }

    #[test]
    fn test_change_log() {
        let settings = SystemSettings::new();
        settings.set_rate_limits(200, 6000, "ops-admin");
        settings.set_rate_limits(150, 4500, "ops-admin");

        let log = settings.get_change_log("default_rate_limit_rps");
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].old_value, "100");
        assert_eq!(log[0].new_value, "200");
        assert_eq!(log[1].old_value, "200");
        assert_eq!(log[1].new_value, "150");

        let full_log = settings.full_change_log();
        assert_eq!(full_log.len(), 2);
    }

    #[test]
    fn test_security_settings() {
        let settings = SystemSettings::new();
        settings.set_mfa_required(true, "security-admin");
        settings.set_password_min_length(16, "security-admin");
        settings.set_self_registration(false, "security-admin");

        let cfg = settings.get_config();
        assert!(cfg.mfa_required);
        assert_eq!(cfg.password_min_length, 16);
        assert!(!cfg.allow_self_registration);
    }
}
