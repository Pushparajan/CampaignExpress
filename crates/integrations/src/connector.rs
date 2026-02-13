//! Integration connector — base trait and common logic for all integrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub direction: SyncDirection,
    pub frequency_seconds: u64,
    pub batch_size: u32,
    pub field_mappings: Vec<FieldMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub source_field: String,
    pub destination_field: String,
    pub transform: Option<FieldTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldTransform {
    ToString,
    ToNumber,
    ToBoolean,
    ToDate,
    Lowercase,
    Uppercase,
    HashSha256,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub records_processed: u64,
    pub records_created: u64,
    pub records_updated: u64,
    pub records_failed: u64,
    pub errors: Vec<SyncError>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub record_id: Option<String>,
    pub error: String,
    pub field: Option<String>,
}

pub struct IntegrationConnector {
    id: Uuid,
    name: String,
    #[allow(dead_code)]
    config: SyncConfig,
}

impl IntegrationConnector {
    pub fn new(name: String, config: SyncConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            config,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn sync(&self) -> anyhow::Result<SyncResult> {
        let started = Utc::now();
        tracing::info!(connector = &self.name, "Starting integration sync");
        let completed = Utc::now();
        let duration = (completed - started).num_milliseconds().max(0) as u64;
        Ok(SyncResult {
            id: Uuid::new_v4(),
            integration_id: self.id,
            records_processed: 0,
            records_created: 0,
            records_updated: 0,
            records_failed: 0,
            errors: Vec::new(),
            started_at: started,
            completed_at: completed,
            duration_ms: duration,
        })
    }
}
