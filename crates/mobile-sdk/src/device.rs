//! Device registration and push token management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::SdkPlatform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub device_id: String,
    pub platform: SdkPlatform,
    pub push_token: Option<String>,
    pub push_enabled: bool,
    pub app_version: String,
    pub sdk_version: String,
    pub os_version: String,
    pub device_model: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub ad_tracking_enabled: bool,
    pub idfa: Option<String>,
    pub gaid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushTokenUpdate {
    pub device_id: String,
    pub push_token: String,
    pub platform: SdkPlatform,
    pub provider: PushProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushProvider {
    Apns,
    ApnsSandbox,
    Fcm,
    Hms,
    WebPush,
}

pub struct DeviceRegistry {
    devices: std::collections::HashMap<String, DeviceRegistration>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, device: DeviceRegistration) {
        self.devices.insert(device.device_id.clone(), device);
    }

    pub fn update_push_token(&mut self, update: &PushTokenUpdate) {
        if let Some(device) = self.devices.get_mut(&update.device_id) {
            device.push_token = Some(update.push_token.clone());
            device.push_enabled = true;
            device.updated_at = Utc::now();
        }
    }

    pub fn get_device(&self, device_id: &str) -> Option<&DeviceRegistration> {
        self.devices.get(device_id)
    }

    pub fn get_user_devices(&self, user_id: &Uuid) -> Vec<&DeviceRegistration> {
        self.devices
            .values()
            .filter(|d| d.user_id.as_ref() == Some(user_id))
            .collect()
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
