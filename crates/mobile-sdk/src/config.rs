//! SDK configuration — remote config served to mobile clients.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkPlatform {
    Ios,
    Android,
    ReactNative,
    Flutter,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    pub api_key: String,
    pub api_endpoint: String,
    pub data_flush_interval_seconds: u32,
    pub session_timeout_seconds: u32,
    pub request_processing_interval_seconds: u32,
    pub minimum_trigger_interval_seconds: u32,
    pub enable_sdk_logging: bool,
    pub enable_in_app_messages: bool,
    pub enable_content_cards: bool,
    pub enable_location_tracking: bool,
    pub enable_geofences: bool,
    pub push_token_registration_enabled: bool,
    pub in_app_message_accessibility_enabled: bool,
    pub custom_endpoints: std::collections::HashMap<String, String>,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_endpoint: "https://api.campaignexpress.io".to_string(),
            data_flush_interval_seconds: 10,
            session_timeout_seconds: 300,
            request_processing_interval_seconds: 10,
            minimum_trigger_interval_seconds: 30,
            enable_sdk_logging: false,
            enable_in_app_messages: true,
            enable_content_cards: true,
            enable_location_tracking: false,
            enable_geofences: false,
            push_token_registration_enabled: true,
            in_app_message_accessibility_enabled: true,
            custom_endpoints: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkInitRequest {
    pub api_key: String,
    pub platform: SdkPlatform,
    pub sdk_version: String,
    pub app_version: String,
    pub device_id: String,
    pub os_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkInitResponse {
    pub config: SdkConfig,
    pub server_time: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<Uuid>,
}

pub struct SdkConfigManager {
    configs: std::collections::HashMap<String, SdkConfig>,
}

impl SdkConfigManager {
    pub fn new() -> Self {
        Self {
            configs: std::collections::HashMap::new(),
        }
    }

    pub fn register_app(&mut self, api_key: String, config: SdkConfig) {
        self.configs.insert(api_key, config);
    }

    pub fn get_config(&self, api_key: &str) -> Option<&SdkConfig> {
        self.configs.get(api_key)
    }

    pub fn handle_init(&self, request: &SdkInitRequest) -> SdkInitResponse {
        let config = self
            .configs
            .get(&request.api_key)
            .cloned()
            .unwrap_or_default();
        SdkInitResponse {
            config,
            server_time: chrono::Utc::now(),
            user_id: None,
        }
    }
}

impl Default for SdkConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
