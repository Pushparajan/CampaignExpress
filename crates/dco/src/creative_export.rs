//! Creative export contract — standardized format for exporting creatives
//! to external platforms, placement validation, and creative lineage tracking.
//!
//! Addresses FR-CREX-001 through FR-CREX-003.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ─── Export Contract (FR-CREX-001) ───────────────────────────────────

/// Standardized creative export spec for external systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeExportContract {
    pub export_id: Uuid,
    pub creative_id: Uuid,
    pub name: String,
    pub version: u32,
    pub format: ExportFormat,
    pub placements: Vec<PlacementExport>,
    pub assets: Vec<AssetReference>,
    pub metadata: ExportMetadata,
    pub exported_by: Uuid,
    pub exported_at: DateTime<Utc>,
}

/// Export format specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Html5,
    StaticImage,
    Video,
    NativeAd,
    VastTag,
    DcmTag,
    Custom(String),
}

/// A placement-specific export variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementExport {
    pub placement_name: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_url: Option<String>,
    pub file_size_bytes: u64,
    pub validated: bool,
    pub validation_errors: Vec<String>,
}

/// Reference to an asset used in the creative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReference {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub role: AssetRole,
    pub version: u32,
    pub rights_valid: bool,
}

/// Role of an asset in the creative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetRole {
    HeroImage,
    Logo,
    BackgroundVideo,
    ProductImage,
    Icon,
    Font,
    Soundtrack,
}

/// Metadata attached to an export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub campaign_id: Option<Uuid>,
    pub brand: String,
    pub target_platforms: Vec<String>,
    pub click_through_url: Option<String>,
    pub tracking_pixels: Vec<String>,
    pub third_party_tags: Vec<String>,
}

// ─── Placement Validation (FR-CREX-002) ──────────────────────────────

/// Rule for validating a creative against a placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementRule {
    pub placement_name: String,
    pub required_width: u32,
    pub required_height: u32,
    pub max_file_size_bytes: u64,
    pub allowed_formats: Vec<String>,
    pub max_animation_seconds: Option<u32>,
    pub require_ssl: bool,
}

/// Result of validating a creative against placement rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementValidationResult {
    pub placement_name: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ─── Creative Lineage (FR-CREX-003) ──────────────────────────────────

/// A lineage event tracking the history of a creative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEvent {
    pub id: Uuid,
    pub creative_id: Uuid,
    pub event_type: LineageEventType,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub details: String,
    pub occurred_at: DateTime<Utc>,
}

/// Types of lineage events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageEventType {
    Created,
    Modified,
    VersionCreated,
    SubmittedForApproval,
    Approved,
    Rejected,
    ExportedToDsp,
    AssignedToCampaign,
    RemovedFromCampaign,
    Archived,
}

/// Full lineage report for a creative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeLineage {
    pub creative_id: Uuid,
    pub events: Vec<LineageEvent>,
    pub campaigns_using: Vec<Uuid>,
    pub total_versions: u32,
    pub current_version: u32,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

// ─── Creative Export Engine ──────────────────────────────────────────

/// Engine for managing creative exports, validation, and lineage.
pub struct CreativeExportEngine {
    exports: DashMap<Uuid, CreativeExportContract>,
    placement_rules: DashMap<String, PlacementRule>,
    lineage: DashMap<Uuid, Vec<LineageEvent>>,
    campaign_assignments: DashMap<Uuid, Vec<Uuid>>,
}

impl CreativeExportEngine {
    pub fn new() -> Self {
        info!("Creative export engine initialized");
        let engine = Self {
            exports: DashMap::new(),
            placement_rules: DashMap::new(),
            lineage: DashMap::new(),
            campaign_assignments: DashMap::new(),
        };
        engine.seed_placement_rules();
        engine
    }

    fn seed_placement_rules(&self) {
        let rules = vec![
            PlacementRule {
                placement_name: "Leaderboard".to_string(),
                required_width: 728,
                required_height: 90,
                max_file_size_bytes: 150_000,
                allowed_formats: vec!["image/png".into(), "image/jpeg".into(), "image/gif".into()],
                max_animation_seconds: Some(30),
                require_ssl: true,
            },
            PlacementRule {
                placement_name: "Medium Rectangle".to_string(),
                required_width: 300,
                required_height: 250,
                max_file_size_bytes: 150_000,
                allowed_formats: vec!["image/png".into(), "image/jpeg".into(), "image/gif".into()],
                max_animation_seconds: Some(30),
                require_ssl: true,
            },
            PlacementRule {
                placement_name: "Mobile Banner".to_string(),
                required_width: 320,
                required_height: 50,
                max_file_size_bytes: 100_000,
                allowed_formats: vec!["image/png".into(), "image/jpeg".into()],
                max_animation_seconds: None,
                require_ssl: true,
            },
            PlacementRule {
                placement_name: "Facebook Feed".to_string(),
                required_width: 1200,
                required_height: 628,
                max_file_size_bytes: 5_000_000,
                allowed_formats: vec!["image/png".into(), "image/jpeg".into(), "video/mp4".into()],
                max_animation_seconds: Some(120),
                require_ssl: true,
            },
            PlacementRule {
                placement_name: "Instagram Story".to_string(),
                required_width: 1080,
                required_height: 1920,
                max_file_size_bytes: 10_000_000,
                allowed_formats: vec!["image/jpeg".into(), "video/mp4".into()],
                max_animation_seconds: Some(15),
                require_ssl: true,
            },
        ];

        for rule in rules {
            self.placement_rules
                .insert(rule.placement_name.clone(), rule);
        }
    }

    /// Export a creative, validating against placement rules.
    #[allow(clippy::too_many_arguments)]
    pub fn export(
        &self,
        creative_id: Uuid,
        name: &str,
        version: u32,
        format: ExportFormat,
        placements: Vec<PlacementExport>,
        assets: Vec<AssetReference>,
        metadata: ExportMetadata,
        exported_by: Uuid,
    ) -> CreativeExportContract {
        let mut validated_placements = Vec::new();

        for mut placement in placements {
            let validation = self.validate_placement(&placement);
            placement.validated = validation.passed;
            placement.validation_errors = validation.errors;
            validated_placements.push(placement);
        }

        let contract = CreativeExportContract {
            export_id: Uuid::new_v4(),
            creative_id,
            name: name.to_string(),
            version,
            format,
            placements: validated_placements,
            assets,
            metadata,
            exported_by,
            exported_at: Utc::now(),
        };

        self.exports.insert(contract.export_id, contract.clone());

        // Record lineage
        self.record_lineage(
            creative_id,
            LineageEventType::ExportedToDsp,
            exported_by,
            "System".to_string(),
            format!("Exported creative v{}", version),
        );

        contract
    }

    /// Validate a placement export against rules.
    pub fn validate_placement(&self, placement: &PlacementExport) -> PlacementValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if let Some(rule) = self.placement_rules.get(&placement.placement_name) {
            if placement.width != rule.required_width {
                errors.push(format!(
                    "Width mismatch: got {} expected {}",
                    placement.width, rule.required_width
                ));
            }
            if placement.height != rule.required_height {
                errors.push(format!(
                    "Height mismatch: got {} expected {}",
                    placement.height, rule.required_height
                ));
            }
            if placement.file_size_bytes > rule.max_file_size_bytes {
                errors.push(format!(
                    "File too large: {} bytes exceeds {} limit",
                    placement.file_size_bytes, rule.max_file_size_bytes
                ));
            }
            if !rule.allowed_formats.contains(&placement.format) {
                errors.push(format!(
                    "Format '{}' not allowed (accepted: {:?})",
                    placement.format, rule.allowed_formats
                ));
            }
        } else {
            warnings.push(format!(
                "No placement rule found for '{}'",
                placement.placement_name
            ));
        }

        PlacementValidationResult {
            placement_name: placement.placement_name.clone(),
            passed: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Record a lineage event for a creative.
    pub fn record_lineage(
        &self,
        creative_id: Uuid,
        event_type: LineageEventType,
        actor_id: Uuid,
        actor_name: String,
        details: String,
    ) -> LineageEvent {
        let event = LineageEvent {
            id: Uuid::new_v4(),
            creative_id,
            event_type,
            actor_id,
            actor_name,
            details,
            occurred_at: Utc::now(),
        };

        self.lineage
            .entry(creative_id)
            .or_default()
            .push(event.clone());

        event
    }

    /// Assign a creative to a campaign.
    pub fn assign_to_campaign(&self, creative_id: Uuid, campaign_id: Uuid, actor_id: Uuid) {
        self.campaign_assignments
            .entry(creative_id)
            .or_default()
            .push(campaign_id);

        self.record_lineage(
            creative_id,
            LineageEventType::AssignedToCampaign,
            actor_id,
            "System".to_string(),
            format!("Assigned to campaign {}", campaign_id),
        );
    }

    /// Get the full lineage report for a creative.
    pub fn get_lineage(&self, creative_id: &Uuid) -> CreativeLineage {
        let events = self
            .lineage
            .get(creative_id)
            .map(|v| v.clone())
            .unwrap_or_default();

        let campaigns_using = self
            .campaign_assignments
            .get(creative_id)
            .map(|v| v.clone())
            .unwrap_or_default();

        let versions_created = events
            .iter()
            .filter(|e| e.event_type == LineageEventType::VersionCreated)
            .count() as u32;

        let created_at = events
            .iter()
            .filter(|e| e.event_type == LineageEventType::Created)
            .map(|e| e.occurred_at)
            .min()
            .unwrap_or_else(Utc::now);

        let last_modified = events
            .iter()
            .map(|e| e.occurred_at)
            .max()
            .unwrap_or_else(Utc::now);

        CreativeLineage {
            creative_id: *creative_id,
            events,
            campaigns_using,
            total_versions: versions_created + 1,
            current_version: versions_created + 1,
            created_at,
            last_modified,
        }
    }

    /// Get export by id.
    pub fn get_export(&self, export_id: &Uuid) -> Option<CreativeExportContract> {
        self.exports.get(export_id).map(|e| e.clone())
    }
}

impl Default for CreativeExportEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placement_validation_pass() {
        let engine = CreativeExportEngine::new();
        let placement = PlacementExport {
            placement_name: "Leaderboard".to_string(),
            width: 728,
            height: 90,
            format: "image/png".to_string(),
            file_url: Some("https://cdn.example.com/ad.png".to_string()),
            file_size_bytes: 120_000,
            validated: false,
            validation_errors: vec![],
        };

        let result = engine.validate_placement(&placement);
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_placement_validation_fail() {
        let engine = CreativeExportEngine::new();
        let placement = PlacementExport {
            placement_name: "Leaderboard".to_string(),
            width: 300,                      // Wrong width
            height: 250,                     // Wrong height
            format: "video/mp4".to_string(), // Not allowed
            file_url: None,
            file_size_bytes: 200_000, // Over limit
            validated: false,
            validation_errors: vec![],
        };

        let result = engine.validate_placement(&placement);
        assert!(!result.passed);
        assert!(result.errors.len() >= 3);
    }

    #[test]
    fn test_creative_export_with_lineage() {
        let engine = CreativeExportEngine::new();
        let creative_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Record creation
        engine.record_lineage(
            creative_id,
            LineageEventType::Created,
            user_id,
            "Alice".to_string(),
            "Created banner creative".to_string(),
        );

        // Export it
        let contract = engine.export(
            creative_id,
            "Summer Banner",
            1,
            ExportFormat::StaticImage,
            vec![PlacementExport {
                placement_name: "Medium Rectangle".to_string(),
                width: 300,
                height: 250,
                format: "image/png".to_string(),
                file_url: Some("https://cdn.example.com/banner.png".to_string()),
                file_size_bytes: 80_000,
                validated: false,
                validation_errors: vec![],
            }],
            vec![AssetReference {
                asset_id: Uuid::new_v4(),
                asset_name: "hero.png".to_string(),
                role: AssetRole::HeroImage,
                version: 1,
                rights_valid: true,
            }],
            ExportMetadata {
                campaign_id: None,
                brand: "Acme".to_string(),
                target_platforms: vec!["google".to_string()],
                click_through_url: Some("https://acme.com/summer".to_string()),
                tracking_pixels: vec![],
                third_party_tags: vec![],
            },
            user_id,
        );

        assert!(contract.placements[0].validated);

        // Check lineage
        let lineage = engine.get_lineage(&creative_id);
        assert_eq!(lineage.events.len(), 2); // Created + ExportedToDsp
    }

    #[test]
    fn test_campaign_assignment_lineage() {
        let engine = CreativeExportEngine::new();
        let creative_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        engine.assign_to_campaign(creative_id, campaign_id, user_id);

        let lineage = engine.get_lineage(&creative_id);
        assert!(lineage.campaigns_using.contains(&campaign_id));
        assert!(lineage
            .events
            .iter()
            .any(|e| e.event_type == LineageEventType::AssignedToCampaign));
    }

    #[test]
    fn test_export_multiple_placements() {
        let engine = CreativeExportEngine::new();
        let creative_id = Uuid::new_v4();

        let contract = engine.export(
            creative_id,
            "Multi-placement Ad",
            1,
            ExportFormat::StaticImage,
            vec![
                PlacementExport {
                    placement_name: "Leaderboard".to_string(),
                    width: 728,
                    height: 90,
                    format: "image/png".to_string(),
                    file_url: None,
                    file_size_bytes: 100_000,
                    validated: false,
                    validation_errors: vec![],
                },
                PlacementExport {
                    placement_name: "Mobile Banner".to_string(),
                    width: 320,
                    height: 50,
                    format: "image/png".to_string(),
                    file_url: None,
                    file_size_bytes: 50_000,
                    validated: false,
                    validation_errors: vec![],
                },
            ],
            vec![],
            ExportMetadata {
                campaign_id: None,
                brand: "Acme".to_string(),
                target_platforms: vec![],
                click_through_url: None,
                tracking_pixels: vec![],
                third_party_tags: vec![],
            },
            Uuid::new_v4(),
        );

        assert_eq!(contract.placements.len(), 2);
        assert!(contract.placements.iter().all(|p| p.validated));
    }
}
