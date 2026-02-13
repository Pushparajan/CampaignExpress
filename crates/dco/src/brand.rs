//! Asset library and brand guidelines enforcement for Dynamic Creative Optimization.
//!
//! Provides [`AssetLibrary`] for managing versioned creative assets (images, videos,
//! logos, fonts, etc.) and [`BrandGuidelinesEngine`] for validating content
//! submissions against configurable brand rules.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Asset types
// ---------------------------------------------------------------------------

/// The kind of creative asset stored in the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Image,
    Video,
    Logo,
    Font,
    ColorPalette,
    Template,
    Document,
    Audio,
    Icon,
    Animation,
}

/// Lifecycle status of an [`Asset`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetStatus {
    Active,
    Archived,
    PendingReview,
    Rejected,
}

/// A single creative asset stored in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub file_size_bytes: u64,
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_seconds: Option<f64>,
    pub tags: Vec<String>,
    /// Virtual folder path, e.g. `"/brand/logos"`.
    pub folder: String,
    pub uploaded_by: Uuid,
    pub version: u32,
    pub status: AssetStatus,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A historical snapshot of a single asset version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetVersion {
    pub version: u32,
    pub url: String,
    pub uploaded_by: Uuid,
    pub changelog: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Asset Library
// ---------------------------------------------------------------------------

/// Thread-safe asset library backed by [`DashMap`].
pub struct AssetLibrary {
    assets: DashMap<Uuid, Asset>,
    versions: DashMap<Uuid, Vec<AssetVersion>>,
    folders: DashMap<String, Vec<Uuid>>,
}

impl AssetLibrary {
    /// Create an empty asset library.
    pub fn new() -> Self {
        Self {
            assets: DashMap::new(),
            versions: DashMap::new(),
            folders: DashMap::new(),
        }
    }

    /// Store a new asset and index it under its folder.
    pub fn upload(&self, asset: Asset) {
        let id = asset.id;
        let folder = asset.folder.clone();
        self.assets.insert(id, asset);
        self.folders.entry(folder).or_default().push(id);
    }

    /// Retrieve an asset by id.
    pub fn get(&self, id: &Uuid) -> Option<Asset> {
        self.assets.get(id).map(|r| r.clone())
    }

    /// Bump the version of an existing asset, recording the old URL in history.
    ///
    /// Returns the new version number, or `None` if the asset does not exist.
    pub fn update_version(
        &self,
        id: Uuid,
        new_url: String,
        uploaded_by: Uuid,
        changelog: String,
    ) -> Option<u32> {
        let mut asset = self.assets.get_mut(&id)?;
        let new_version = asset.version + 1;
        asset.version = new_version;
        asset.url = new_url.clone();
        asset.updated_at = Utc::now();

        self.versions.entry(id).or_default().push(AssetVersion {
            version: new_version,
            url: new_url,
            uploaded_by,
            changelog,
            created_at: Utc::now(),
        });

        Some(new_version)
    }

    /// Search assets by name or tag, optionally filtering by type and folder.
    pub fn search(
        &self,
        query: &str,
        asset_type: Option<AssetType>,
        folder: Option<&str>,
    ) -> Vec<Asset> {
        let query_lower = query.to_lowercase();
        self.assets
            .iter()
            .filter(|entry| {
                let a = entry.value();
                let matches_query = a.name.to_lowercase().contains(&query_lower)
                    || a.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower));
                let matches_type = asset_type.as_ref().is_none_or(|at| a.asset_type == *at);
                let matches_folder = folder.is_none_or(|f| a.folder == f);
                matches_query && matches_type && matches_folder
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// List every asset in the given folder.
    pub fn list_folder(&self, folder: &str) -> Vec<Asset> {
        let ids = match self.folders.get(folder) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        ids.iter()
            .filter_map(|id| self.assets.get(id).map(|r| r.clone()))
            .collect()
    }

    /// Return the full version history for an asset.
    pub fn get_version_history(&self, id: &Uuid) -> Vec<AssetVersion> {
        self.versions.get(id).map(|r| r.clone()).unwrap_or_default()
    }

    /// Archive an asset. Returns `true` if the asset existed.
    pub fn archive(&self, id: &Uuid) -> bool {
        if let Some(mut asset) = self.assets.get_mut(id) {
            asset.status = AssetStatus::Archived;
            asset.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Append tags to an asset. Returns `true` if the asset existed.
    pub fn tag(&self, id: &Uuid, tags: Vec<String>) -> bool {
        if let Some(mut asset) = self.assets.get_mut(id) {
            asset.tags.extend(tags);
            asset.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
}

impl Default for AssetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Brand guideline types
// ---------------------------------------------------------------------------

/// Severity level for brand rule violations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Warn,
    Block,
}

/// Category of a brand rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrandRuleType {
    ColorCompliance,
    TypographyCompliance,
    LogoPlacement,
    ToneCompliance,
    ImageQuality,
    MinImageResolution,
    MaxFileSize,
}

/// A single enforceable brand rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandRule {
    pub id: Uuid,
    pub rule_type: BrandRuleType,
    pub description: String,
    pub severity: Severity,
}

/// A named brand color with its intended usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandColor {
    pub name: String,
    pub hex: String,
    pub rgb: (u8, u8, u8),
    pub usage: String,
}

/// Typography constraints for a given usage context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontRule {
    pub font_family: String,
    pub usage: String,
    pub min_size_px: u32,
    pub max_size_px: u32,
    pub allowed_weights: Vec<u32>,
}

/// Logo placement and usage restrictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoUsageRules {
    pub min_clear_space_px: u32,
    pub min_width_px: u32,
    pub allowed_backgrounds: Vec<String>,
    pub prohibited_modifications: Vec<String>,
}

/// Voice and tone guidelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneGuide {
    pub voice_attributes: Vec<String>,
    pub prohibited_words: Vec<String>,
    pub max_sentence_length: u32,
    pub required_cta_patterns: Vec<String>,
}

/// A complete set of brand guidelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandGuideline {
    pub id: Uuid,
    pub name: String,
    pub rules: Vec<BrandRule>,
    pub color_palette: Vec<BrandColor>,
    pub typography: Vec<FontRule>,
    pub logo_usage: LogoUsageRules,
    pub tone_of_voice: ToneGuide,
    pub updated_at: DateTime<Utc>,
}

/// A content submission to be validated against brand guidelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSubmission {
    pub text: Option<String>,
    pub colors_used: Vec<String>,
    /// `(font_family, size_px)` pairs.
    pub fonts_used: Vec<(String, u32)>,
    /// `(width, height)` in pixels.
    pub image_dimensions: Option<(u32, u32)>,
    pub file_size_bytes: Option<u64>,
    pub has_logo: bool,
    pub logo_background: Option<String>,
}

/// A single violation produced by brand-guideline validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandViolation {
    pub rule_id: Uuid,
    pub rule_type: BrandRuleType,
    pub severity: Severity,
    pub message: String,
    pub field: String,
}

// ---------------------------------------------------------------------------
// Brand Guidelines Engine
// ---------------------------------------------------------------------------

/// Validates content submissions against stored [`BrandGuideline`]s.
pub struct BrandGuidelinesEngine {
    guidelines: DashMap<Uuid, BrandGuideline>,
}

impl BrandGuidelinesEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self {
            guidelines: DashMap::new(),
        }
    }

    /// Store a brand guideline.
    pub fn create_guideline(&self, guideline: BrandGuideline) {
        self.guidelines.insert(guideline.id, guideline);
    }

    /// Retrieve a guideline by id.
    pub fn get_guideline(&self, id: &Uuid) -> Option<BrandGuideline> {
        self.guidelines.get(id).map(|r| r.clone())
    }

    /// Validate a [`ContentSubmission`] against every rule in the specified guideline.
    pub fn validate_content(
        &self,
        guideline_id: &Uuid,
        content: &ContentSubmission,
    ) -> Vec<BrandViolation> {
        let guideline = match self.guidelines.get(guideline_id) {
            Some(g) => g.clone(),
            None => return Vec::new(),
        };

        let mut violations = Vec::new();

        for rule in &guideline.rules {
            match rule.rule_type {
                BrandRuleType::ColorCompliance => {
                    self.check_colors(rule, &guideline.color_palette, content, &mut violations);
                }
                BrandRuleType::TypographyCompliance => {
                    self.check_typography(rule, &guideline.typography, content, &mut violations);
                }
                BrandRuleType::MinImageResolution => {
                    self.check_image_resolution(rule, content, &mut violations);
                }
                BrandRuleType::MaxFileSize => {
                    self.check_file_size(rule, content, &mut violations);
                }
                BrandRuleType::LogoPlacement => {
                    self.check_logo(rule, &guideline.logo_usage, content, &mut violations);
                }
                BrandRuleType::ToneCompliance => {
                    self.check_tone(rule, &guideline.tone_of_voice, content, &mut violations);
                }
                BrandRuleType::ImageQuality => {
                    // Image-quality checks (e.g. DPI, compression artefacts) require
                    // binary inspection which is out of scope here. Placeholder for
                    // future implementation.
                }
            }
        }

        violations
    }

    // -- private validation helpers -----------------------------------------

    fn check_colors(
        &self,
        rule: &BrandRule,
        palette: &[BrandColor],
        content: &ContentSubmission,
        violations: &mut Vec<BrandViolation>,
    ) {
        let allowed: Vec<String> = palette.iter().map(|c| c.hex.to_lowercase()).collect();
        for color in &content.colors_used {
            if !allowed.contains(&color.to_lowercase()) {
                violations.push(BrandViolation {
                    rule_id: rule.id,
                    rule_type: BrandRuleType::ColorCompliance,
                    severity: rule.severity.clone(),
                    message: format!("Color {} is not in the approved brand palette", color),
                    field: "colors_used".to_string(),
                });
            }
        }
    }

    fn check_typography(
        &self,
        rule: &BrandRule,
        typography: &[FontRule],
        content: &ContentSubmission,
        violations: &mut Vec<BrandViolation>,
    ) {
        let allowed_families: Vec<String> = typography
            .iter()
            .map(|f| f.font_family.to_lowercase())
            .collect();

        for (family, size) in &content.fonts_used {
            if !allowed_families.contains(&family.to_lowercase()) {
                violations.push(BrandViolation {
                    rule_id: rule.id,
                    rule_type: BrandRuleType::TypographyCompliance,
                    severity: rule.severity.clone(),
                    message: format!(
                        "Font family '{}' is not in the approved typography rules",
                        family
                    ),
                    field: "fonts_used".to_string(),
                });
                continue;
            }

            // Check size against matching font rules — valid if ANY rule allows it.
            let matching_rules: Vec<_> = typography
                .iter()
                .filter(|fr| fr.font_family.to_lowercase() == family.to_lowercase())
                .collect();
            let size_ok = matching_rules
                .iter()
                .any(|fr| *size >= fr.min_size_px && *size <= fr.max_size_px);
            if !matching_rules.is_empty() && !size_ok {
                let ranges: Vec<String> = matching_rules
                    .iter()
                    .map(|fr| format!("{}-{}px", fr.min_size_px, fr.max_size_px))
                    .collect();
                violations.push(BrandViolation {
                    rule_id: rule.id,
                    rule_type: BrandRuleType::TypographyCompliance,
                    severity: rule.severity.clone(),
                    message: format!(
                        "Font '{}' at {}px is outside allowed ranges ({})",
                        family,
                        size,
                        ranges.join(", ")
                    ),
                    field: "fonts_used".to_string(),
                });
            }
        }
    }

    fn check_image_resolution(
        &self,
        rule: &BrandRule,
        content: &ContentSubmission,
        violations: &mut Vec<BrandViolation>,
    ) {
        if let Some((w, h)) = content.image_dimensions {
            // Minimum 800x600.
            if w < 800 || h < 600 {
                violations.push(BrandViolation {
                    rule_id: rule.id,
                    rule_type: BrandRuleType::MinImageResolution,
                    severity: rule.severity.clone(),
                    message: format!("Image resolution {}x{} is below minimum 800x600", w, h),
                    field: "image_dimensions".to_string(),
                });
            }
        }
    }

    fn check_file_size(
        &self,
        rule: &BrandRule,
        content: &ContentSubmission,
        violations: &mut Vec<BrandViolation>,
    ) {
        if let Some(size) = content.file_size_bytes {
            // 10 MB limit.
            let max_bytes: u64 = 10 * 1024 * 1024;
            if size > max_bytes {
                violations.push(BrandViolation {
                    rule_id: rule.id,
                    rule_type: BrandRuleType::MaxFileSize,
                    severity: rule.severity.clone(),
                    message: format!(
                        "File size {} bytes exceeds maximum {} bytes",
                        size, max_bytes
                    ),
                    field: "file_size_bytes".to_string(),
                });
            }
        }
    }

    fn check_logo(
        &self,
        rule: &BrandRule,
        logo_rules: &LogoUsageRules,
        content: &ContentSubmission,
        violations: &mut Vec<BrandViolation>,
    ) {
        if content.has_logo {
            if let Some(ref bg) = content.logo_background {
                let allowed: Vec<String> = logo_rules
                    .allowed_backgrounds
                    .iter()
                    .map(|b| b.to_lowercase())
                    .collect();
                if !allowed.contains(&bg.to_lowercase()) {
                    violations.push(BrandViolation {
                        rule_id: rule.id,
                        rule_type: BrandRuleType::LogoPlacement,
                        severity: rule.severity.clone(),
                        message: format!("Logo background '{}' is not in the allowed list", bg),
                        field: "logo_background".to_string(),
                    });
                }
            }
        }
    }

    fn check_tone(
        &self,
        rule: &BrandRule,
        tone: &ToneGuide,
        content: &ContentSubmission,
        violations: &mut Vec<BrandViolation>,
    ) {
        if let Some(ref text) = content.text {
            let text_lower = text.to_lowercase();

            // Prohibited words.
            for word in &tone.prohibited_words {
                if text_lower.contains(&word.to_lowercase()) {
                    violations.push(BrandViolation {
                        rule_id: rule.id,
                        rule_type: BrandRuleType::ToneCompliance,
                        severity: rule.severity.clone(),
                        message: format!("Text contains prohibited word: '{}'", word),
                        field: "text".to_string(),
                    });
                }
            }

            // Max sentence length.
            for sentence in text.split(['.', '!', '?']) {
                let trimmed = sentence.trim();
                if !trimmed.is_empty() {
                    let word_count = trimmed.split_whitespace().count() as u32;
                    if word_count > tone.max_sentence_length {
                        violations.push(BrandViolation {
                            rule_id: rule.id,
                            rule_type: BrandRuleType::ToneCompliance,
                            severity: rule.severity.clone(),
                            message: format!(
                                "Sentence exceeds max length of {} words: '{}'",
                                tone.max_sentence_length,
                                if trimmed.len() > 60 {
                                    format!("{}...", &trimmed[..60])
                                } else {
                                    trimmed.to_string()
                                }
                            ),
                            field: "text".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Seed a complete default brand guideline with representative values.
    pub fn seed_default_guidelines(&self) -> Uuid {
        let id = Uuid::new_v4();

        let color_palette = vec![
            BrandColor {
                name: "Primary Blue".to_string(),
                hex: "#0052CC".to_string(),
                rgb: (0, 82, 204),
                usage: "primary".to_string(),
            },
            BrandColor {
                name: "Secondary Teal".to_string(),
                hex: "#00B8D9".to_string(),
                rgb: (0, 184, 217),
                usage: "secondary".to_string(),
            },
            BrandColor {
                name: "Accent Orange".to_string(),
                hex: "#FF5630".to_string(),
                rgb: (255, 86, 48),
                usage: "accent".to_string(),
            },
            BrandColor {
                name: "Neutral Dark".to_string(),
                hex: "#172B4D".to_string(),
                rgb: (23, 43, 77),
                usage: "text".to_string(),
            },
            BrandColor {
                name: "Neutral Light".to_string(),
                hex: "#F4F5F7".to_string(),
                rgb: (244, 245, 247),
                usage: "background".to_string(),
            },
            BrandColor {
                name: "White".to_string(),
                hex: "#FFFFFF".to_string(),
                rgb: (255, 255, 255),
                usage: "background".to_string(),
            },
        ];

        let typography = vec![
            FontRule {
                font_family: "Inter".to_string(),
                usage: "heading".to_string(),
                min_size_px: 18,
                max_size_px: 72,
                allowed_weights: vec![600, 700, 800],
            },
            FontRule {
                font_family: "Inter".to_string(),
                usage: "body".to_string(),
                min_size_px: 14,
                max_size_px: 20,
                allowed_weights: vec![400, 500],
            },
            FontRule {
                font_family: "JetBrains Mono".to_string(),
                usage: "caption".to_string(),
                min_size_px: 10,
                max_size_px: 14,
                allowed_weights: vec![400],
            },
        ];

        let logo_usage = LogoUsageRules {
            min_clear_space_px: 24,
            min_width_px: 120,
            allowed_backgrounds: vec![
                "#FFFFFF".to_string(),
                "#F4F5F7".to_string(),
                "#172B4D".to_string(),
            ],
            prohibited_modifications: vec![
                "rotate".to_string(),
                "stretch".to_string(),
                "recolor".to_string(),
                "crop".to_string(),
            ],
        };

        let tone_of_voice = ToneGuide {
            voice_attributes: vec![
                "professional".to_string(),
                "warm".to_string(),
                "concise".to_string(),
                "inclusive".to_string(),
                "confident".to_string(),
            ],
            prohibited_words: vec![
                "cheap".to_string(),
                "free".to_string(),
                "guarantee".to_string(),
                "spam".to_string(),
                "urgent".to_string(),
                "limited".to_string(),
                "act now".to_string(),
                "no obligation".to_string(),
                "risk-free".to_string(),
                "winner".to_string(),
            ],
            max_sentence_length: 25,
            required_cta_patterns: vec![
                "Shop Now".to_string(),
                "Learn More".to_string(),
                "Get Started".to_string(),
                "Contact Us".to_string(),
            ],
        };

        let rules = vec![
            BrandRule {
                id: Uuid::new_v4(),
                rule_type: BrandRuleType::ColorCompliance,
                description: "All colours must belong to the approved palette".to_string(),
                severity: Severity::Block,
            },
            BrandRule {
                id: Uuid::new_v4(),
                rule_type: BrandRuleType::TypographyCompliance,
                description: "Only approved font families and sizes may be used".to_string(),
                severity: Severity::Block,
            },
            BrandRule {
                id: Uuid::new_v4(),
                rule_type: BrandRuleType::LogoPlacement,
                description: "Logo must appear on an approved background".to_string(),
                severity: Severity::Block,
            },
            BrandRule {
                id: Uuid::new_v4(),
                rule_type: BrandRuleType::ToneCompliance,
                description: "Text must follow tone-of-voice guidelines".to_string(),
                severity: Severity::Warn,
            },
            BrandRule {
                id: Uuid::new_v4(),
                rule_type: BrandRuleType::MinImageResolution,
                description: "Images must be at least 800x600 pixels".to_string(),
                severity: Severity::Block,
            },
        ];

        let guideline = BrandGuideline {
            id,
            name: "Campaign Express Default Brand".to_string(),
            rules,
            color_palette,
            typography,
            logo_usage,
            tone_of_voice,
            updated_at: Utc::now(),
        };

        self.create_guideline(guideline);
        id
    }
}

impl Default for BrandGuidelinesEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_asset(name: &str, folder: &str, asset_type: AssetType) -> Asset {
        Asset {
            id: Uuid::new_v4(),
            name: name.to_string(),
            asset_type,
            url: format!("https://cdn.example.com/{}", name),
            thumbnail_url: None,
            file_size_bytes: 1024,
            mime_type: "image/png".to_string(),
            width: Some(1920),
            height: Some(1080),
            duration_seconds: None,
            tags: vec!["brand".to_string(), "2024".to_string()],
            folder: folder.to_string(),
            uploaded_by: Uuid::new_v4(),
            version: 1,
            status: AssetStatus::Active,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 1. Asset upload and search
    #[test]
    fn test_asset_upload_and_search() {
        let lib = AssetLibrary::new();

        let logo = sample_asset("acme_logo", "/brand/logos", AssetType::Logo);
        let banner = sample_asset("summer_banner", "/brand/banners", AssetType::Image);
        lib.upload(logo.clone());
        lib.upload(banner.clone());

        // Search by name.
        let results = lib.search("acme", None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "acme_logo");

        // Search by type.
        let results = lib.search("", Some(AssetType::Image), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "summer_banner");

        // Search by folder.
        let folder_assets = lib.list_folder("/brand/logos");
        assert_eq!(folder_assets.len(), 1);
        assert_eq!(folder_assets[0].id, logo.id);
    }

    // 2. Asset version history
    #[test]
    fn test_asset_version_history() {
        let lib = AssetLibrary::new();

        let asset = sample_asset("hero_image", "/campaign/hero", AssetType::Image);
        let id = asset.id;
        lib.upload(asset);

        let v2 = lib.update_version(
            id,
            "https://cdn.example.com/hero_v2.png".to_string(),
            Uuid::new_v4(),
            "Updated colour grading".to_string(),
        );
        assert_eq!(v2, Some(2));

        let v3 = lib.update_version(
            id,
            "https://cdn.example.com/hero_v3.png".to_string(),
            Uuid::new_v4(),
            "Cropped for mobile".to_string(),
        );
        assert_eq!(v3, Some(3));

        let history = lib.get_version_history(&id);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 2);
        assert_eq!(history[1].version, 3);

        let current = lib.get(&id).unwrap();
        assert_eq!(current.version, 3);
    }

    // 3. Brand validation — all compliant
    #[test]
    fn test_brand_validation_pass() {
        let engine = BrandGuidelinesEngine::new();
        let gid = engine.seed_default_guidelines();

        let submission = ContentSubmission {
            text: Some("Discover our latest collection today.".to_string()),
            colors_used: vec!["#0052CC".to_string(), "#FFFFFF".to_string()],
            fonts_used: vec![("Inter".to_string(), 16)],
            image_dimensions: Some((1920, 1080)),
            file_size_bytes: Some(500_000),
            has_logo: true,
            logo_background: Some("#FFFFFF".to_string()),
        };

        let violations = engine.validate_content(&gid, &submission);
        assert!(
            violations.is_empty(),
            "Expected no violations but got: {:?}",
            violations
        );
    }

    // 4. Brand validation — color violation
    #[test]
    fn test_brand_validation_color_violation() {
        let engine = BrandGuidelinesEngine::new();
        let gid = engine.seed_default_guidelines();

        let submission = ContentSubmission {
            text: None,
            colors_used: vec!["#FF00FF".to_string()], // magenta — not in palette
            fonts_used: vec![],
            image_dimensions: None,
            file_size_bytes: None,
            has_logo: false,
            logo_background: None,
        };

        let violations = engine.validate_content(&gid, &submission);
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.rule_type == BrandRuleType::ColorCompliance));
    }

    // 5. Brand validation — prohibited word in content
    #[test]
    fn test_brand_validation_prohibited_word() {
        let engine = BrandGuidelinesEngine::new();
        let gid = engine.seed_default_guidelines();

        let submission = ContentSubmission {
            text: Some("Act now and get this cheap deal.".to_string()),
            colors_used: vec![],
            fonts_used: vec![],
            image_dimensions: None,
            file_size_bytes: None,
            has_logo: false,
            logo_background: None,
        };

        let violations = engine.validate_content(&gid, &submission);
        let tone_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_type == BrandRuleType::ToneCompliance)
            .collect();
        // Should catch at least "cheap" and "act now".
        assert!(
            tone_violations.len() >= 2,
            "Expected at least 2 tone violations, got {}",
            tone_violations.len()
        );
    }

    // 6. Brand validation — image too small
    #[test]
    fn test_brand_validation_image_too_small() {
        let engine = BrandGuidelinesEngine::new();
        let gid = engine.seed_default_guidelines();

        let submission = ContentSubmission {
            text: None,
            colors_used: vec![],
            fonts_used: vec![],
            image_dimensions: Some((400, 300)), // below 800x600
            file_size_bytes: None,
            has_logo: false,
            logo_background: None,
        };

        let violations = engine.validate_content(&gid, &submission);
        assert!(violations
            .iter()
            .any(|v| v.rule_type == BrandRuleType::MinImageResolution));
    }
}
