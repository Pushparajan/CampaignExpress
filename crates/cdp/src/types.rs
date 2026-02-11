use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported CDP platforms.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CdpPlatform {
    SalesforceDataCloud,
    AdobeRealTimeCdp,
    TwilioSegment,
    Tealium,
    Hightouch,
}

impl CdpPlatform {
    /// Human-readable display name for this platform.
    pub fn display_name(&self) -> &'static str {
        match self {
            CdpPlatform::SalesforceDataCloud => "Salesforce Data Cloud",
            CdpPlatform::AdobeRealTimeCdp => "Adobe Real-Time CDP",
            CdpPlatform::TwilioSegment => "Twilio Segment",
            CdpPlatform::Tealium => "Tealium",
            CdpPlatform::Hightouch => "Hightouch",
        }
    }

    /// Default batch size for this platform's sync operations.
    pub fn default_batch_size(&self) -> usize {
        match self {
            CdpPlatform::SalesforceDataCloud => 10_000,
            CdpPlatform::AdobeRealTimeCdp => 5_000,
            CdpPlatform::TwilioSegment => 20_000,
            CdpPlatform::Tealium => 8_000,
            CdpPlatform::Hightouch => 15_000,
        }
    }
}

/// Configuration for a CDP platform connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpConfig {
    pub platform: CdpPlatform,
    pub api_endpoint: String,
    pub api_key: String,
    pub api_secret: Option<String>,
    pub enabled: bool,
    pub sync_interval_secs: u64,
    pub batch_size: usize,
    pub field_mappings: HashMap<String, String>,
}

/// A unified customer profile from a CDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpProfile {
    pub external_id: String,
    pub platform: CdpPlatform,
    pub attributes: HashMap<String, serde_json::Value>,
    pub segments: Vec<String>,
    pub consent: ConsentFlags,
    pub last_synced: DateTime<Utc>,
}

/// Privacy and consent flags for a profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsentFlags {
    pub gdpr_consent: bool,
    pub ccpa_opt_out: bool,
    pub email_opt_in: bool,
    pub push_opt_in: bool,
    pub sms_opt_in: bool,
    pub personalization_consent: bool,
}

/// Direction of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

/// Record of a sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub id: Uuid,
    pub platform: CdpPlatform,
    pub direction: SyncDirection,
    pub record_count: u64,
    pub status: SyncStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Status of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    PartialSuccess,
}

/// Incoming webhook payload from a CDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpWebhookPayload {
    pub platform: CdpPlatform,
    pub event_type: String,
    pub profiles: Vec<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

/// An audience export request targeting a CDP platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceExport {
    pub id: Uuid,
    pub name: String,
    pub platform: CdpPlatform,
    pub segment_ids: Vec<u32>,
    pub user_count: u64,
    pub status: SyncStatus,
    pub created_at: DateTime<Utc>,
}
