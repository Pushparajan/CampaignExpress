use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::types::{CdpConfig, CdpPlatform, CdpProfile, ConsentFlags};

/// Trait for transforming data between the internal representation and a
/// specific CDP platform's format.
pub trait CdpAdapter: Send + Sync {
    /// The CDP platform this adapter handles.
    fn platform(&self) -> CdpPlatform;

    /// Transform raw CDP data into an internal profile.
    fn transform_inbound(&self, raw: &serde_json::Value) -> Result<CdpProfile>;

    /// Transform an internal profile into the CDP platform's format.
    fn transform_outbound(&self, profile: &CdpProfile) -> Result<serde_json::Value>;

    /// Validate that the given config is well-formed for this platform.
    fn validate_config(&self, config: &CdpConfig) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Salesforce Data Cloud
// ---------------------------------------------------------------------------

pub struct SalesforceAdapter;

impl CdpAdapter for SalesforceAdapter {
    fn platform(&self) -> CdpPlatform {
        CdpPlatform::SalesforceDataCloud
    }

    fn transform_inbound(&self, raw: &serde_json::Value) -> Result<CdpProfile> {
        let external_id = raw
            .get("Id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing Id field in Salesforce payload"))?
            .to_string();

        let mut attributes = HashMap::new();
        if let Some(email) = raw.get("Email") {
            attributes.insert("email".to_string(), email.clone());
        }
        if let Some(first_name) = raw.get("FirstName") {
            attributes.insert("first_name".to_string(), first_name.clone());
        }

        let segments = raw
            .get("segments")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(CdpProfile {
            external_id,
            platform: CdpPlatform::SalesforceDataCloud,
            attributes,
            segments,
            consent: ConsentFlags::default(),
            last_synced: Utc::now(),
        })
    }

    fn transform_outbound(&self, profile: &CdpProfile) -> Result<serde_json::Value> {
        let mut out = serde_json::Map::new();
        out.insert("Id".to_string(), serde_json::Value::String(profile.external_id.clone()));
        if let Some(email) = profile.attributes.get("email") {
            out.insert("Email".to_string(), email.clone());
        }
        if let Some(first_name) = profile.attributes.get("first_name") {
            out.insert("FirstName".to_string(), first_name.clone());
        }
        out.insert(
            "segments".to_string(),
            serde_json::Value::Array(
                profile.segments.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            ),
        );
        Ok(serde_json::Value::Object(out))
    }

    fn validate_config(&self, config: &CdpConfig) -> Result<()> {
        if config.api_endpoint.is_empty() {
            return Err(anyhow!("Salesforce api_endpoint must not be empty"));
        }
        if config.api_key.is_empty() {
            return Err(anyhow!("Salesforce api_key must not be empty"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Adobe Real-Time CDP
// ---------------------------------------------------------------------------

pub struct AdobeAdapter;

impl CdpAdapter for AdobeAdapter {
    fn platform(&self) -> CdpPlatform {
        CdpPlatform::AdobeRealTimeCdp
    }

    fn transform_inbound(&self, raw: &serde_json::Value) -> Result<CdpProfile> {
        let external_id = raw
            .get("adobeId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing adobeId field in Adobe payload"))?
            .to_string();

        let mut attributes = HashMap::new();
        if let Some(email) = raw.get("emailAddress") {
            attributes.insert("email".to_string(), email.clone());
        }
        if let Some(ecid) = raw.get("experienceCloudId") {
            attributes.insert("experience_cloud_id".to_string(), ecid.clone());
        }

        let segments = raw
            .get("segments")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(CdpProfile {
            external_id,
            platform: CdpPlatform::AdobeRealTimeCdp,
            attributes,
            segments,
            consent: ConsentFlags::default(),
            last_synced: Utc::now(),
        })
    }

    fn transform_outbound(&self, profile: &CdpProfile) -> Result<serde_json::Value> {
        let mut out = serde_json::Map::new();
        out.insert("adobeId".to_string(), serde_json::Value::String(profile.external_id.clone()));
        if let Some(email) = profile.attributes.get("email") {
            out.insert("emailAddress".to_string(), email.clone());
        }
        if let Some(ecid) = profile.attributes.get("experience_cloud_id") {
            out.insert("experienceCloudId".to_string(), ecid.clone());
        }
        out.insert(
            "segments".to_string(),
            serde_json::Value::Array(
                profile.segments.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            ),
        );
        Ok(serde_json::Value::Object(out))
    }

    fn validate_config(&self, config: &CdpConfig) -> Result<()> {
        if config.api_endpoint.is_empty() {
            return Err(anyhow!("Adobe api_endpoint must not be empty"));
        }
        if config.api_key.is_empty() {
            return Err(anyhow!("Adobe api_key must not be empty"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Twilio Segment
// ---------------------------------------------------------------------------

pub struct SegmentAdapter;

impl CdpAdapter for SegmentAdapter {
    fn platform(&self) -> CdpPlatform {
        CdpPlatform::TwilioSegment
    }

    fn transform_inbound(&self, raw: &serde_json::Value) -> Result<CdpProfile> {
        let external_id = raw
            .get("userId")
            .or_else(|| raw.get("anonymousId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing userId or anonymousId in Segment payload"))?
            .to_string();

        let mut attributes = HashMap::new();
        if let Some(traits) = raw.get("traits").and_then(|v| v.as_object()) {
            for (k, v) in traits {
                attributes.insert(k.clone(), v.clone());
            }
        }

        let segments = raw
            .get("segments")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(CdpProfile {
            external_id,
            platform: CdpPlatform::TwilioSegment,
            attributes,
            segments,
            consent: ConsentFlags::default(),
            last_synced: Utc::now(),
        })
    }

    fn transform_outbound(&self, profile: &CdpProfile) -> Result<serde_json::Value> {
        let mut out = serde_json::Map::new();
        out.insert("userId".to_string(), serde_json::Value::String(profile.external_id.clone()));
        let traits_map: serde_json::Map<String, serde_json::Value> = profile
            .attributes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        out.insert("traits".to_string(), serde_json::Value::Object(traits_map));
        out.insert(
            "segments".to_string(),
            serde_json::Value::Array(
                profile.segments.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            ),
        );
        Ok(serde_json::Value::Object(out))
    }

    fn validate_config(&self, config: &CdpConfig) -> Result<()> {
        if config.api_key.is_empty() {
            return Err(anyhow!("Segment api_key (write key) must not be empty"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tealium
// ---------------------------------------------------------------------------

pub struct TealiumAdapter;

impl CdpAdapter for TealiumAdapter {
    fn platform(&self) -> CdpPlatform {
        CdpPlatform::Tealium
    }

    fn transform_inbound(&self, raw: &serde_json::Value) -> Result<CdpProfile> {
        let external_id = raw
            .get("tealium_visitor_id")
            .or_else(|| raw.get("visitor_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing tealium_visitor_id or visitor_id in Tealium payload"))?
            .to_string();

        let mut attributes = HashMap::new();
        if let Some(email) = raw.get("email") {
            attributes.insert("email".to_string(), email.clone());
        }
        if let Some(vid) = raw.get("visitor_id") {
            attributes.insert("visitor_id".to_string(), vid.clone());
        }

        let segments = raw
            .get("audiences")
            .or_else(|| raw.get("segments"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(CdpProfile {
            external_id,
            platform: CdpPlatform::Tealium,
            attributes,
            segments,
            consent: ConsentFlags::default(),
            last_synced: Utc::now(),
        })
    }

    fn transform_outbound(&self, profile: &CdpProfile) -> Result<serde_json::Value> {
        let mut out = serde_json::Map::new();
        out.insert(
            "tealium_visitor_id".to_string(),
            serde_json::Value::String(profile.external_id.clone()),
        );
        if let Some(email) = profile.attributes.get("email") {
            out.insert("email".to_string(), email.clone());
        }
        if let Some(vid) = profile.attributes.get("visitor_id") {
            out.insert("visitor_id".to_string(), vid.clone());
        }
        out.insert(
            "audiences".to_string(),
            serde_json::Value::Array(
                profile.segments.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            ),
        );
        Ok(serde_json::Value::Object(out))
    }

    fn validate_config(&self, config: &CdpConfig) -> Result<()> {
        if config.api_endpoint.is_empty() {
            return Err(anyhow!("Tealium api_endpoint must not be empty"));
        }
        if config.api_key.is_empty() {
            return Err(anyhow!("Tealium api_key must not be empty"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Hightouch
// ---------------------------------------------------------------------------

pub struct HightouchAdapter;

impl CdpAdapter for HightouchAdapter {
    fn platform(&self) -> CdpPlatform {
        CdpPlatform::Hightouch
    }

    fn transform_inbound(&self, raw: &serde_json::Value) -> Result<CdpProfile> {
        let external_id = raw
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing id field in Hightouch payload"))?
            .to_string();

        let mut attributes = HashMap::new();
        if let Some(email) = raw.get("email") {
            attributes.insert("email".to_string(), email.clone());
        }
        if let Some(props) = raw.get("properties").and_then(|v| v.as_object()) {
            for (k, v) in props {
                attributes.insert(k.clone(), v.clone());
            }
        }

        let segments = raw
            .get("segments")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(CdpProfile {
            external_id,
            platform: CdpPlatform::Hightouch,
            attributes,
            segments,
            consent: ConsentFlags::default(),
            last_synced: Utc::now(),
        })
    }

    fn transform_outbound(&self, profile: &CdpProfile) -> Result<serde_json::Value> {
        let mut out = serde_json::Map::new();
        out.insert("id".to_string(), serde_json::Value::String(profile.external_id.clone()));
        if let Some(email) = profile.attributes.get("email") {
            out.insert("email".to_string(), email.clone());
        }
        let props: serde_json::Map<String, serde_json::Value> = profile
            .attributes
            .iter()
            .filter(|(k, _)| k.as_str() != "email")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        out.insert("properties".to_string(), serde_json::Value::Object(props));
        out.insert(
            "segments".to_string(),
            serde_json::Value::Array(
                profile.segments.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            ),
        );
        Ok(serde_json::Value::Object(out))
    }

    fn validate_config(&self, config: &CdpConfig) -> Result<()> {
        if config.api_key.is_empty() {
            return Err(anyhow!("Hightouch api_key must not be empty"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Create the appropriate adapter for the given CDP platform.
pub fn create_adapter(platform: &CdpPlatform) -> Box<dyn CdpAdapter + Send + Sync> {
    match platform {
        CdpPlatform::SalesforceDataCloud => Box::new(SalesforceAdapter),
        CdpPlatform::AdobeRealTimeCdp => Box::new(AdobeAdapter),
        CdpPlatform::TwilioSegment => Box::new(SegmentAdapter),
        CdpPlatform::Tealium => Box::new(TealiumAdapter),
        CdpPlatform::Hightouch => Box::new(HightouchAdapter),
    }
}
