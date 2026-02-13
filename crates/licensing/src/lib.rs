//! Campaign Express Licensing — module-gated license generation, signing, and verification.
//!
//! Licenses are HMAC-SHA256 signed JSON payloads encoded as `<base64-payload>.<base64-signature>`.
//! An admin tool generates license files; the runtime `LicenseGuard` validates and gates modules.

pub mod dashboard;

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("invalid license format: expected <payload>.<signature>")]
    InvalidFormat,
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("signature verification failed — license may be tampered")]
    SignatureInvalid,
    #[error("license expired at {0}")]
    Expired(DateTime<Utc>),
    #[error("module `{0}` is not included in this license")]
    ModuleNotLicensed(String),
    #[error("node limit exceeded: licensed for {licensed}, requested {requested}")]
    NodeLimitExceeded { licensed: u32, requested: u32 },
    #[error("throughput limit exceeded: licensed for {licensed} offers/hr, requested {requested}")]
    ThroughputExceeded { licensed: u64, requested: u64 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Licensed modules
// ---------------------------------------------------------------------------

/// Every licensable module in Campaign Express.
/// Core infrastructure (core, cache, analytics, api-server, agents, npu-engine, platform)
/// is always included. Everything else requires a license grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicensedModule {
    Loyalty,
    Dsp,
    Channels,
    Management,
    Journey,
    Dco,
    Cdp,
    Billing,
    Ops,
    Personalization,
    Segmentation,
    Reporting,
    Integrations,
    IntelligentDelivery,
    RlEngine,
    MobileSdk,
    PluginMarketplace,
    SdkDocs,
    WasmEdge,
}

impl LicensedModule {
    /// All possible licensable modules.
    pub const ALL: &'static [LicensedModule] = &[
        Self::Loyalty,
        Self::Dsp,
        Self::Channels,
        Self::Management,
        Self::Journey,
        Self::Dco,
        Self::Cdp,
        Self::Billing,
        Self::Ops,
        Self::Personalization,
        Self::Segmentation,
        Self::Reporting,
        Self::Integrations,
        Self::IntelligentDelivery,
        Self::RlEngine,
        Self::MobileSdk,
        Self::PluginMarketplace,
        Self::SdkDocs,
        Self::WasmEdge,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loyalty => "loyalty",
            Self::Dsp => "dsp",
            Self::Channels => "channels",
            Self::Management => "management",
            Self::Journey => "journey",
            Self::Dco => "dco",
            Self::Cdp => "cdp",
            Self::Billing => "billing",
            Self::Ops => "ops",
            Self::Personalization => "personalization",
            Self::Segmentation => "segmentation",
            Self::Reporting => "reporting",
            Self::Integrations => "integrations",
            Self::IntelligentDelivery => "intelligent_delivery",
            Self::RlEngine => "rl_engine",
            Self::MobileSdk => "mobile_sdk",
            Self::PluginMarketplace => "plugin_marketplace",
            Self::SdkDocs => "sdk_docs",
            Self::WasmEdge => "wasm_edge",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Loyalty => "3-tier loyalty program engine",
            Self::Dsp => "DSP integration router (TTD, DV360, Xandr, Amazon)",
            Self::Channels => "Omnichannel ingest and activation",
            Self::Management => "Campaign management dashboard & workflows",
            Self::Journey => "Journey orchestration with branching & A/B splits",
            Self::Dco => "Dynamic Creative Optimization & brand guidelines",
            Self::Cdp => "Customer Data Platform adapters",
            Self::Billing => "Usage metering & subscription management",
            Self::Ops => "Operations: backup, SLA, incident management",
            Self::Personalization => "Recommendation engine & catalog",
            Self::Segmentation => "Real-time segmentation & predictive segments",
            Self::Reporting => "Analytics, dashboards, funnels, attribution",
            Self::Integrations => "Integration marketplace (Asana, Jira, DAM, BI)",
            Self::IntelligentDelivery => "Send-time optimization & suppression",
            Self::RlEngine => "Reinforcement learning & OfferFit connector",
            Self::MobileSdk => "Mobile SDK server-side support",
            Self::PluginMarketplace => "Plugin marketplace & sandboxing",
            Self::SdkDocs => "SDK documentation server",
            Self::WasmEdge => "Edge worker for bid preprocessing",
        }
    }
}

impl std::fmt::Display for LicensedModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// License tiers (convenience presets)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTier {
    Starter,
    Professional,
    Enterprise,
}

impl LicenseTier {
    /// Default modules included in each tier.
    pub fn default_modules(&self) -> Vec<LicensedModule> {
        match self {
            Self::Starter => vec![
                LicensedModule::Management,
                LicensedModule::Channels,
                LicensedModule::Reporting,
                LicensedModule::SdkDocs,
            ],
            Self::Professional => vec![
                LicensedModule::Management,
                LicensedModule::Channels,
                LicensedModule::Reporting,
                LicensedModule::SdkDocs,
                LicensedModule::Loyalty,
                LicensedModule::Journey,
                LicensedModule::Segmentation,
                LicensedModule::Personalization,
                LicensedModule::MobileSdk,
                LicensedModule::Cdp,
                LicensedModule::Billing,
            ],
            Self::Enterprise => LicensedModule::ALL.to_vec(),
        }
    }

    pub fn default_max_nodes(&self) -> u32 {
        match self {
            Self::Starter => 3,
            Self::Professional => 10,
            Self::Enterprise => 100,
        }
    }

    pub fn default_max_offers_per_hour(&self) -> u64 {
        match self {
            Self::Starter => 1_000_000,
            Self::Professional => 10_000_000,
            Self::Enterprise => 100_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LicenseType {
    #[default]
    Commercial,
    Trial,
    Internal,
}

// ---------------------------------------------------------------------------
// License payload
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub license_id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub license_type: LicenseType,
    pub tier: LicenseTier,
    pub modules: Vec<LicensedModule>,
    pub max_nodes: u32,
    pub max_offers_per_hour: u64,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub issued_by: String,
}

impl License {
    /// Check whether a specific module is granted by this license.
    pub fn has_module(&self, module: LicensedModule) -> bool {
        self.modules.contains(&module)
    }

    /// Check whether the license has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Validate the license is not expired and a given module is included.
    pub fn check_module(&self, module: LicensedModule) -> Result<(), LicenseError> {
        if self.is_expired() {
            return Err(LicenseError::Expired(self.expires_at));
        }
        if !self.has_module(module) {
            return Err(LicenseError::ModuleNotLicensed(module.to_string()));
        }
        Ok(())
    }

    /// Validate node count against the license limit.
    pub fn check_nodes(&self, node_count: u32) -> Result<(), LicenseError> {
        if node_count > self.max_nodes {
            return Err(LicenseError::NodeLimitExceeded {
                licensed: self.max_nodes,
                requested: node_count,
            });
        }
        Ok(())
    }

    /// Validate throughput against the license limit.
    pub fn check_throughput(&self, offers_per_hour: u64) -> Result<(), LicenseError> {
        if offers_per_hour > self.max_offers_per_hour {
            return Err(LicenseError::ThroughputExceeded {
                licensed: self.max_offers_per_hour,
                requested: offers_per_hour,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Signing key
// ---------------------------------------------------------------------------

/// A 256-bit HMAC-SHA256 signing key.
#[derive(Clone)]
pub struct LicenseKey {
    bytes: Vec<u8>,
}

impl LicenseKey {
    /// Create a key from raw bytes (must be at least 32 bytes).
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Generate a new random 256-bit key.
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self { bytes: key }
    }

    /// Encode the key to base64 for storage.
    pub fn to_base64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&self.bytes)
    }

    /// Decode a key from base64.
    pub fn from_base64(encoded: &str) -> Result<Self, LicenseError> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(encoded)?;
        Ok(Self { bytes })
    }

    /// Write the key to a file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), LicenseError> {
        std::fs::write(path, self.to_base64())?;
        Ok(())
    }

    /// Read the key from a file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, LicenseError> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_base64(contents.trim())
    }
}

// ---------------------------------------------------------------------------
// Signing & verification
// ---------------------------------------------------------------------------

/// Sign a license, returning the complete license file contents: `<base64-payload>.<base64-sig>`.
pub fn sign_license(license: &License, key: &LicenseKey) -> Result<String, LicenseError> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;

    let payload_json = serde_json::to_vec(license)?;
    let payload_b64 = engine.encode(&payload_json);

    let mut mac =
        HmacSha256::new_from_slice(&key.bytes).expect("HMAC accepts any key length");
    mac.update(payload_json.as_slice());
    let signature = mac.finalize().into_bytes();
    let sig_b64 = engine.encode(signature);

    Ok(format!("{payload_b64}.{sig_b64}"))
}

/// Parse and verify a signed license file. Returns the `License` if valid.
pub fn verify_license(license_file: &str, key: &LicenseKey) -> Result<License, LicenseError> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;

    let parts: Vec<&str> = license_file.trim().splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(LicenseError::InvalidFormat);
    }

    let payload_json = engine.decode(parts[0])?;
    let signature = engine.decode(parts[1])?;

    // Verify HMAC
    let mut mac =
        HmacSha256::new_from_slice(&key.bytes).expect("HMAC accepts any key length");
    mac.update(&payload_json);
    mac.verify_slice(&signature)
        .map_err(|_| LicenseError::SignatureInvalid)?;

    let license: License = serde_json::from_slice(&payload_json)?;
    Ok(license)
}

/// Load a license file from disk and verify it.
pub fn load_license_file(
    path: &std::path::Path,
    key: &LicenseKey,
) -> Result<License, LicenseError> {
    let contents = std::fs::read_to_string(path)?;
    verify_license(&contents, key)
}

// ---------------------------------------------------------------------------
// Runtime license guard
// ---------------------------------------------------------------------------

/// Runtime license guard that validates module access.
/// Intended to be initialized once at startup and queried throughout the application lifetime.
pub struct LicenseGuard {
    license: License,
    modules: dashmap::DashMap<LicensedModule, ()>,
}

impl LicenseGuard {
    /// Create a new guard from a verified license.
    pub fn new(license: License) -> Self {
        let modules = dashmap::DashMap::new();
        for m in &license.modules {
            modules.insert(*m, ());
        }
        Self { license, modules }
    }

    /// Load and verify a license file, then create a guard.
    pub fn from_file(
        license_path: &std::path::Path,
        key: &LicenseKey,
    ) -> Result<Self, LicenseError> {
        let license = load_license_file(license_path, key)?;
        if license.is_expired() {
            return Err(LicenseError::Expired(license.expires_at));
        }
        Ok(Self::new(license))
    }

    /// Check if a module is licensed. Returns `Ok(())` or an error.
    pub fn require_module(&self, module: LicensedModule) -> Result<(), LicenseError> {
        if self.license.is_expired() {
            return Err(LicenseError::Expired(self.license.expires_at));
        }
        if self.modules.contains_key(&module) {
            Ok(())
        } else {
            Err(LicenseError::ModuleNotLicensed(module.to_string()))
        }
    }

    /// Check if a module is licensed (bool convenience).
    pub fn is_module_licensed(&self, module: LicensedModule) -> bool {
        self.modules.contains_key(&module)
    }

    /// Get the underlying license.
    pub fn license(&self) -> &License {
        &self.license
    }

    /// List all licensed modules.
    pub fn licensed_modules(&self) -> Vec<LicensedModule> {
        self.license.modules.clone()
    }

    /// Check node count limit.
    pub fn check_nodes(&self, count: u32) -> Result<(), LicenseError> {
        self.license.check_nodes(count)
    }

    /// Check throughput limit.
    pub fn check_throughput(&self, offers_per_hour: u64) -> Result<(), LicenseError> {
        self.license.check_throughput(offers_per_hour)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_test_license() -> License {
        License {
            license_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            tenant_name: "Acme Corp".into(),
            license_type: LicenseType::Commercial,
            tier: LicenseTier::Professional,
            modules: vec![
                LicensedModule::Management,
                LicensedModule::Channels,
                LicensedModule::Reporting,
                LicensedModule::Loyalty,
                LicensedModule::Journey,
            ],
            max_nodes: 10,
            max_offers_per_hour: 10_000_000,
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(365),
            issued_by: "license-admin".into(),
        }
    }

    #[test]
    fn test_sign_and_verify() {
        let key = LicenseKey::generate();
        let license = make_test_license();

        let signed = sign_license(&license, &key).unwrap();
        assert!(signed.contains('.'));

        let verified = verify_license(&signed, &key).unwrap();
        assert_eq!(verified.license_id, license.license_id);
        assert_eq!(verified.tenant_name, "Acme Corp");
        assert_eq!(verified.modules.len(), 5);
    }

    #[test]
    fn test_tampered_payload_fails() {
        let key = LicenseKey::generate();
        let license = make_test_license();
        let signed = sign_license(&license, &key).unwrap();

        // Tamper with the payload portion
        let mut tampered = signed.clone();
        let dot = tampered.find('.').unwrap();
        // Flip a character in the payload
        unsafe {
            let bytes = tampered.as_bytes_mut();
            bytes[dot - 1] = if bytes[dot - 1] == b'A' { b'B' } else { b'A' };
        }

        let result = verify_license(&tampered, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = LicenseKey::generate();
        let key2 = LicenseKey::generate();
        let license = make_test_license();
        let signed = sign_license(&license, &key1).unwrap();

        let result = verify_license(&signed, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_module_check() {
        let license = make_test_license();
        assert!(license.has_module(LicensedModule::Management));
        assert!(!license.has_module(LicensedModule::Dsp));

        assert!(license.check_module(LicensedModule::Loyalty).is_ok());
        assert!(license.check_module(LicensedModule::Dsp).is_err());
    }

    #[test]
    fn test_expired_license() {
        let mut license = make_test_license();
        license.expires_at = Utc::now() - Duration::days(1);

        assert!(license.is_expired());
        let result = license.check_module(LicensedModule::Management);
        assert!(matches!(result, Err(LicenseError::Expired(_))));
    }

    #[test]
    fn test_node_limit() {
        let license = make_test_license();
        assert!(license.check_nodes(10).is_ok());
        assert!(license.check_nodes(11).is_err());
    }

    #[test]
    fn test_throughput_limit() {
        let license = make_test_license();
        assert!(license.check_throughput(10_000_000).is_ok());
        assert!(license.check_throughput(10_000_001).is_err());
    }

    #[test]
    fn test_license_guard() {
        let license = make_test_license();
        let guard = LicenseGuard::new(license);

        assert!(guard.is_module_licensed(LicensedModule::Management));
        assert!(!guard.is_module_licensed(LicensedModule::Dsp));
        assert!(guard.require_module(LicensedModule::Loyalty).is_ok());
        assert!(guard.require_module(LicensedModule::RlEngine).is_err());
    }

    #[test]
    fn test_key_base64_roundtrip() {
        let key = LicenseKey::generate();
        let b64 = key.to_base64();
        let restored = LicenseKey::from_base64(&b64).unwrap();
        assert_eq!(key.bytes, restored.bytes);
    }

    #[test]
    fn test_tier_presets() {
        let starter = LicenseTier::Starter.default_modules();
        assert_eq!(starter.len(), 4);

        let pro = LicenseTier::Professional.default_modules();
        assert_eq!(pro.len(), 11);

        let enterprise = LicenseTier::Enterprise.default_modules();
        assert_eq!(enterprise.len(), LicensedModule::ALL.len());
    }

    #[test]
    fn test_license_file_roundtrip() {
        let key = LicenseKey::generate();
        let license = make_test_license();
        let signed = sign_license(&license, &key).unwrap();

        let tmp = std::env::temp_dir().join("test_license.lic");
        std::fs::write(&tmp, &signed).unwrap();
        let loaded = load_license_file(&tmp, &key).unwrap();
        assert_eq!(loaded.license_id, license.license_id);
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_all_modules_listed() {
        // Ensure ALL constant has every variant
        assert_eq!(LicensedModule::ALL.len(), 19);
        for m in LicensedModule::ALL {
            assert!(!m.as_str().is_empty());
            assert!(!m.description().is_empty());
        }
    }
}
