use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::info;
use uuid::Uuid;

use crate::adapters::create_adapter;
use crate::types::{
    AudienceExport, CdpConfig, CdpPlatform, CdpProfile, CdpWebhookPayload, SyncDirection,
    SyncEvent, SyncStatus,
};

/// Engine that orchestrates CDP sync operations across multiple platforms.
pub struct CdpSyncEngine {
    configs: Arc<DashMap<String, CdpConfig>>,
    sync_history: Arc<DashMap<Uuid, SyncEvent>>,
}

impl CdpSyncEngine {
    /// Create a new engine with empty configuration.
    pub fn new() -> Self {
        Self {
            configs: Arc::new(DashMap::new()),
            sync_history: Arc::new(DashMap::new()),
        }
    }

    /// Register a CDP platform with the given configuration.
    pub fn register_platform(&self, name: String, config: CdpConfig) -> Result<()> {
        let adapter = create_adapter(&config.platform);
        adapter.validate_config(&config)?;

        info!(
            platform = config.platform.display_name(),
            name = %name,
            "registered CDP platform"
        );
        self.configs.insert(name, config);
        Ok(())
    }

    /// List all registered platforms and their configs.
    pub fn list_platforms(&self) -> Vec<(String, CdpConfig)> {
        self.configs
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Look up a single platform config by name.
    pub fn get_platform(&self, name: &str) -> Option<CdpConfig> {
        self.configs.get(name).map(|entry| entry.value().clone())
    }

    /// Remove a registered platform.
    pub fn remove_platform(&self, name: &str) -> Result<()> {
        self.configs
            .remove(name)
            .ok_or_else(|| anyhow!("platform '{}' not found", name))?;
        info!(name = %name, "removed CDP platform");
        Ok(())
    }

    /// Process an inbound webhook from a CDP.
    ///
    /// Transforms each raw profile via the platform's adapter and records a
    /// [`SyncEvent`].
    pub fn process_inbound_webhook(&self, payload: &CdpWebhookPayload) -> Result<SyncEvent> {
        let adapter = create_adapter(&payload.platform);
        let started_at = Utc::now();
        let mut record_count: u64 = 0;
        let mut errors: Vec<String> = Vec::new();

        for raw in &payload.profiles {
            match adapter.transform_inbound(raw) {
                Ok(_profile) => {
                    record_count += 1;
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }

        let status = if errors.is_empty() {
            SyncStatus::Completed
        } else if record_count > 0 {
            SyncStatus::PartialSuccess
        } else {
            SyncStatus::Failed
        };

        let event = SyncEvent {
            id: Uuid::new_v4(),
            platform: payload.platform.clone(),
            direction: SyncDirection::Inbound,
            record_count,
            status,
            started_at,
            completed_at: Some(Utc::now()),
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        };

        info!(
            event_id = %event.id,
            platform = payload.platform.display_name(),
            record_count = record_count,
            "processed inbound webhook"
        );

        self.sync_history.insert(event.id, event.clone());
        Ok(event)
    }

    /// Prepare an outbound sync for the given platform and profiles.
    ///
    /// Transforms profiles through the adapter and records a [`SyncEvent`].
    pub fn prepare_outbound_sync(
        &self,
        platform_name: &str,
        profiles: &[CdpProfile],
    ) -> Result<SyncEvent> {
        let config = self
            .configs
            .get(platform_name)
            .ok_or_else(|| anyhow!("platform '{}' not registered", platform_name))?;

        let adapter = create_adapter(&config.platform);
        let started_at = Utc::now();
        let mut record_count: u64 = 0;
        let mut errors: Vec<String> = Vec::new();

        for profile in profiles {
            match adapter.transform_outbound(profile) {
                Ok(_value) => {
                    record_count += 1;
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }

        let status = if errors.is_empty() {
            SyncStatus::Completed
        } else if record_count > 0 {
            SyncStatus::PartialSuccess
        } else {
            SyncStatus::Failed
        };

        let event = SyncEvent {
            id: Uuid::new_v4(),
            platform: config.platform.clone(),
            direction: SyncDirection::Outbound,
            record_count,
            status,
            started_at,
            completed_at: Some(Utc::now()),
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        };

        info!(
            event_id = %event.id,
            platform = config.platform.display_name(),
            record_count = record_count,
            "prepared outbound sync"
        );

        self.sync_history.insert(event.id, event.clone());
        Ok(event)
    }

    /// Store an audience export request and return its id.
    pub fn export_audience(&self, export: AudienceExport) -> Result<Uuid> {
        let id = export.id;
        info!(
            export_id = %id,
            name = %export.name,
            platform = export.platform.display_name(),
            user_count = export.user_count,
            "audience export queued"
        );

        // Record as a sync event for history tracking.
        let event = SyncEvent {
            id,
            platform: export.platform.clone(),
            direction: SyncDirection::Outbound,
            record_count: export.user_count,
            status: export.status.clone(),
            started_at: export.created_at,
            completed_at: None,
            error: None,
        };
        self.sync_history.insert(id, event);

        Ok(id)
    }

    /// Return all recorded sync events.
    pub fn get_sync_history(&self) -> Vec<SyncEvent> {
        self.sync_history
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Seed the engine with demo configurations for all five CDP platforms.
    pub fn seed_demo_configs(&self) {
        let demos: Vec<(&str, CdpConfig)> = vec![
            (
                "salesforce",
                CdpConfig {
                    platform: CdpPlatform::SalesforceDataCloud,
                    api_endpoint: "https://api.salesforce.com/cdp/v1".to_string(),
                    api_key: "demo-sf-key-001".to_string(),
                    api_secret: Some("demo-sf-secret".to_string()),
                    enabled: true,
                    sync_interval_secs: 300,
                    batch_size: CdpPlatform::SalesforceDataCloud.default_batch_size(),
                    field_mappings: HashMap::from([
                        ("email".to_string(), "Email".to_string()),
                        ("first_name".to_string(), "FirstName".to_string()),
                    ]),
                },
            ),
            (
                "adobe",
                CdpConfig {
                    platform: CdpPlatform::AdobeRealTimeCdp,
                    api_endpoint: "https://platform.adobe.io/data/core".to_string(),
                    api_key: "demo-adobe-key-001".to_string(),
                    api_secret: Some("demo-adobe-secret".to_string()),
                    enabled: true,
                    sync_interval_secs: 600,
                    batch_size: CdpPlatform::AdobeRealTimeCdp.default_batch_size(),
                    field_mappings: HashMap::from([
                        ("email".to_string(), "emailAddress".to_string()),
                        ("ecid".to_string(), "experienceCloudId".to_string()),
                    ]),
                },
            ),
            (
                "segment",
                CdpConfig {
                    platform: CdpPlatform::TwilioSegment,
                    api_endpoint: "https://api.segment.io/v1".to_string(),
                    api_key: "demo-segment-write-key".to_string(),
                    api_secret: None,
                    enabled: true,
                    sync_interval_secs: 120,
                    batch_size: CdpPlatform::TwilioSegment.default_batch_size(),
                    field_mappings: HashMap::from([
                        ("user_id".to_string(), "userId".to_string()),
                        ("anonymous_id".to_string(), "anonymousId".to_string()),
                    ]),
                },
            ),
            (
                "tealium",
                CdpConfig {
                    platform: CdpPlatform::Tealium,
                    api_endpoint: "https://collect.tealiumiq.com/event".to_string(),
                    api_key: "demo-tealium-key-001".to_string(),
                    api_secret: None,
                    enabled: true,
                    sync_interval_secs: 180,
                    batch_size: CdpPlatform::Tealium.default_batch_size(),
                    field_mappings: HashMap::from([
                        ("email".to_string(), "email".to_string()),
                        ("visitor_id".to_string(), "tealium_visitor_id".to_string()),
                    ]),
                },
            ),
            (
                "hightouch",
                CdpConfig {
                    platform: CdpPlatform::Hightouch,
                    api_endpoint: "https://api.hightouch.io/v1".to_string(),
                    api_key: "demo-hightouch-key-001".to_string(),
                    api_secret: None,
                    enabled: true,
                    sync_interval_secs: 240,
                    batch_size: CdpPlatform::Hightouch.default_batch_size(),
                    field_mappings: HashMap::from([
                        ("email".to_string(), "email".to_string()),
                        ("id".to_string(), "id".to_string()),
                    ]),
                },
            ),
        ];

        for (name, config) in demos {
            self.configs.insert(name.to_string(), config);
        }
        info!("seeded 5 demo CDP platform configs");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_list() {
        let engine = CdpSyncEngine::new();
        engine.seed_demo_configs();

        let platforms = engine.list_platforms();
        assert_eq!(platforms.len(), 5);

        let sf = engine.get_platform("salesforce");
        assert!(sf.is_some());
        let sf = sf.unwrap();
        assert_eq!(sf.platform, CdpPlatform::SalesforceDataCloud);
        assert!(sf.enabled);

        // Remove and verify
        engine.remove_platform("salesforce").unwrap();
        assert!(engine.get_platform("salesforce").is_none());
        assert_eq!(engine.list_platforms().len(), 4);
    }

    #[test]
    fn test_inbound_webhook() {
        let engine = CdpSyncEngine::new();
        engine.seed_demo_configs();

        let payload = CdpWebhookPayload {
            platform: CdpPlatform::SalesforceDataCloud,
            event_type: "profile_update".to_string(),
            profiles: vec![
                serde_json::json!({
                    "Id": "sf-001",
                    "Email": "alice@example.com",
                    "FirstName": "Alice",
                    "segments": ["high_value", "active"]
                }),
                serde_json::json!({
                    "Id": "sf-002",
                    "Email": "bob@example.com",
                    "FirstName": "Bob",
                    "segments": ["new_user"]
                }),
            ],
            timestamp: Utc::now(),
            signature: None,
        };

        let event = engine.process_inbound_webhook(&payload).unwrap();
        assert_eq!(event.record_count, 2);
        assert_eq!(event.status, SyncStatus::Completed);
        assert_eq!(event.direction, SyncDirection::Inbound);
        assert!(event.error.is_none());

        let history = engine.get_sync_history();
        assert_eq!(history.len(), 1);
    }
}
