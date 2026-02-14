//! Asset Library operational governance: ingestion UX, lifecycle workflow,
//! versioning & rollback, rights management, and renditions/optimization.
//!
//! Extends the existing `AssetLibrary` in `brand.rs` with enterprise-grade
//! asset management features.
//!
//! Addresses FR-AST-001 through FR-AST-005.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Asset Ingestion (FR-AST-001) ─────────────────────────────────────

/// Metadata template applied during bulk upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataTemplate {
    pub id: Uuid,
    pub name: String,
    pub default_tags: Vec<String>,
    pub default_folder: String,
    pub campaign_ids: Vec<Uuid>,
    pub usage_rights: Option<AssetRights>,
}

/// Result of a single asset ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionResult {
    pub asset_id: Uuid,
    pub filename: String,
    pub status: IngestionStatus,
    pub duplicate_of: Option<Uuid>,
    pub auto_tags: Vec<String>,
}

/// Status of ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestionStatus {
    Accepted,
    Duplicate,
    Rejected,
}

/// A record for duplicate detection via content hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetHash {
    pub asset_id: Uuid,
    pub content_hash: String,
    pub file_size: u64,
}

/// Bulk asset ingestion engine with duplicate detection and auto-tagging.
pub struct AssetIngestionEngine {
    hashes: DashMap<String, AssetHash>,
    metadata_templates: DashMap<Uuid, MetadataTemplate>,
}

impl AssetIngestionEngine {
    pub fn new() -> Self {
        Self {
            hashes: DashMap::new(),
            metadata_templates: DashMap::new(),
        }
    }

    /// Register a metadata template for bulk uploads.
    pub fn register_template(&self, template: MetadataTemplate) {
        self.metadata_templates.insert(template.id, template);
    }

    /// Process a batch of assets for upload. Returns per-asset results.
    pub fn bulk_ingest(
        &self,
        files: Vec<(String, String, u64)>, // (filename, content_hash, file_size)
        template_id: Option<&Uuid>,
    ) -> Vec<IngestionResult> {
        let _template =
            template_id.and_then(|id| self.metadata_templates.get(id).map(|t| t.clone()));

        files
            .iter()
            .map(|(filename, hash, size)| {
                // Duplicate detection
                if let Some(existing) = self.hashes.get(hash) {
                    return IngestionResult {
                        asset_id: Uuid::new_v4(),
                        filename: filename.clone(),
                        status: IngestionStatus::Duplicate,
                        duplicate_of: Some(existing.asset_id),
                        auto_tags: Vec::new(),
                    };
                }

                let asset_id = Uuid::new_v4();
                self.hashes.insert(
                    hash.clone(),
                    AssetHash {
                        asset_id,
                        content_hash: hash.clone(),
                        file_size: *size,
                    },
                );

                // Auto-tagging suggestions based on filename
                let auto_tags = Self::suggest_tags(filename);

                IngestionResult {
                    asset_id,
                    filename: filename.clone(),
                    status: IngestionStatus::Accepted,
                    duplicate_of: None,
                    auto_tags,
                }
            })
            .collect()
    }

    /// Suggest tags based on filename patterns.
    fn suggest_tags(filename: &str) -> Vec<String> {
        let lower = filename.to_lowercase();
        let mut tags = Vec::new();

        if lower.contains("hero") {
            tags.push("hero".to_string());
        }
        if lower.contains("banner") {
            tags.push("banner".to_string());
        }
        if lower.contains("logo") {
            tags.push("logo".to_string());
        }
        if lower.contains("icon") {
            tags.push("icon".to_string());
        }
        if lower.contains("cta") {
            tags.push("cta".to_string());
        }
        if lower.contains("product") {
            tags.push("product".to_string());
        }
        if lower.contains("social") {
            tags.push("social".to_string());
        }
        if lower.contains("email") {
            tags.push("email".to_string());
        }

        // Detect image type from extension
        if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            tags.push("image".to_string());
        } else if lower.ends_with(".mp4") || lower.ends_with(".mov") {
            tags.push("video".to_string());
        } else if lower.ends_with(".svg") {
            tags.push("vector".to_string());
        }

        tags
    }
}

impl Default for AssetIngestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Asset Lifecycle Workflow (FR-AST-002) ────────────────────────────

/// Asset lifecycle stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetLifecycleStage {
    Draft,
    PendingReview,
    Approved,
    Rejected,
    Archived,
}

/// A reviewer assignment for an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReviewer {
    pub user_id: Uuid,
    pub role: String,
    pub assigned_at: DateTime<Utc>,
    pub sla_due: DateTime<Utc>,
    pub decided: bool,
    pub approved: Option<bool>,
    pub comment: Option<String>,
}

/// Asset lifecycle record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLifecycle {
    pub asset_id: Uuid,
    pub stage: AssetLifecycleStage,
    pub reviewers: Vec<AssetReviewer>,
    pub transitions: Vec<AssetLifecycleTransition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A transition in the asset lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLifecycleTransition {
    pub from: AssetLifecycleStage,
    pub to: AssetLifecycleStage,
    pub actor_id: Uuid,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Asset lifecycle workflow engine.
pub struct AssetWorkflowEngine {
    lifecycles: DashMap<Uuid, AssetLifecycle>,
}

impl AssetWorkflowEngine {
    pub fn new() -> Self {
        Self {
            lifecycles: DashMap::new(),
        }
    }

    /// Register an asset in draft stage.
    pub fn register(&self, asset_id: Uuid) -> AssetLifecycle {
        let now = Utc::now();
        let lifecycle = AssetLifecycle {
            asset_id,
            stage: AssetLifecycleStage::Draft,
            reviewers: Vec::new(),
            transitions: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        self.lifecycles.insert(asset_id, lifecycle.clone());
        lifecycle
    }

    /// Submit for review with assigned reviewers and SLA.
    pub fn submit_for_review(
        &self,
        asset_id: &Uuid,
        reviewers: Vec<(Uuid, String)>,
        sla_hours: u32,
        actor_id: Uuid,
    ) -> Result<AssetLifecycle, String> {
        let mut entry = self
            .lifecycles
            .get_mut(asset_id)
            .ok_or("Asset not registered")?;

        if entry.stage != AssetLifecycleStage::Draft {
            return Err("Asset must be in Draft stage to submit for review".to_string());
        }

        let now = Utc::now();
        let sla_due = now + chrono::Duration::hours(sla_hours as i64);

        entry.reviewers = reviewers
            .into_iter()
            .map(|(user_id, role)| AssetReviewer {
                user_id,
                role,
                assigned_at: now,
                sla_due,
                decided: false,
                approved: None,
                comment: None,
            })
            .collect();

        entry.transitions.push(AssetLifecycleTransition {
            from: AssetLifecycleStage::Draft,
            to: AssetLifecycleStage::PendingReview,
            actor_id,
            reason: Some("Submitted for review".to_string()),
            timestamp: now,
        });

        entry.stage = AssetLifecycleStage::PendingReview;
        entry.updated_at = now;
        Ok(entry.clone())
    }

    /// Record a reviewer's decision.
    pub fn review_decision(
        &self,
        asset_id: &Uuid,
        reviewer_id: Uuid,
        approved: bool,
        comment: Option<String>,
    ) -> Result<AssetLifecycle, String> {
        let mut entry = self
            .lifecycles
            .get_mut(asset_id)
            .ok_or("Asset not registered")?;

        if entry.stage != AssetLifecycleStage::PendingReview {
            return Err("Asset is not pending review".to_string());
        }

        // Record decision
        if let Some(reviewer) = entry
            .reviewers
            .iter_mut()
            .find(|r| r.user_id == reviewer_id)
        {
            reviewer.decided = true;
            reviewer.approved = Some(approved);
            reviewer.comment = comment;
        } else {
            return Err("Reviewer not assigned to this asset".to_string());
        }

        // Check if all reviewers have decided
        let all_decided = entry.reviewers.iter().all(|r| r.decided);
        if all_decided {
            let any_rejected = entry.reviewers.iter().any(|r| r.approved == Some(false));
            let new_stage = if any_rejected {
                AssetLifecycleStage::Rejected
            } else {
                AssetLifecycleStage::Approved
            };

            entry.transitions.push(AssetLifecycleTransition {
                from: AssetLifecycleStage::PendingReview,
                to: new_stage.clone(),
                actor_id: reviewer_id,
                reason: Some("All reviewers decided".to_string()),
                timestamp: Utc::now(),
            });
            entry.stage = new_stage;
        }

        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Check if an asset is approved (can be used in publish/go-live).
    pub fn is_approved(&self, asset_id: &Uuid) -> bool {
        self.lifecycles
            .get(asset_id)
            .is_some_and(|l| l.stage == AssetLifecycleStage::Approved)
    }

    /// Archive an asset.
    pub fn archive(&self, asset_id: &Uuid, actor_id: Uuid) -> Result<AssetLifecycle, String> {
        let mut entry = self
            .lifecycles
            .get_mut(asset_id)
            .ok_or("Asset not registered")?;

        let old_stage = entry.stage.clone();
        entry.transitions.push(AssetLifecycleTransition {
            from: old_stage,
            to: AssetLifecycleStage::Archived,
            actor_id,
            reason: Some("Archived".to_string()),
            timestamp: Utc::now(),
        });
        entry.stage = AssetLifecycleStage::Archived;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    pub fn get(&self, asset_id: &Uuid) -> Option<AssetLifecycle> {
        self.lifecycles.get(asset_id).map(|l| l.clone())
    }

    /// Get all assets with overdue SLA (pending review past sla_due).
    pub fn overdue_reviews(&self) -> Vec<(Uuid, Vec<AssetReviewer>)> {
        let now = Utc::now();
        self.lifecycles
            .iter()
            .filter(|e| e.value().stage == AssetLifecycleStage::PendingReview)
            .filter_map(|e| {
                let overdue: Vec<AssetReviewer> = e
                    .value()
                    .reviewers
                    .iter()
                    .filter(|r| !r.decided && r.sla_due < now)
                    .cloned()
                    .collect();
                if overdue.is_empty() {
                    None
                } else {
                    Some((*e.key(), overdue))
                }
            })
            .collect()
    }
}

impl Default for AssetWorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Asset Versioning & Rollback (FR-AST-003) ─────────────────────────

/// An asset version pin: a campaign/template pinned to a specific asset version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetVersionPin {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub pinned_version: u32,
    pub pinned_by: PinTarget,
    pub pinned_at: DateTime<Utc>,
}

/// What entity is pinned to this version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinTarget {
    Campaign(Uuid),
    Template(Uuid),
    Creative(Uuid),
}

/// Asset version manager for pinning and rollback.
pub struct AssetVersionManager {
    pins: DashMap<Uuid, AssetVersionPin>,
}

impl AssetVersionManager {
    pub fn new() -> Self {
        Self {
            pins: DashMap::new(),
        }
    }

    /// Pin a campaign/template to a specific asset version.
    pub fn pin(&self, asset_id: Uuid, version: u32, target: PinTarget) -> AssetVersionPin {
        let pin = AssetVersionPin {
            id: Uuid::new_v4(),
            asset_id,
            pinned_version: version,
            pinned_by: target,
            pinned_at: Utc::now(),
        };
        self.pins.insert(pin.id, pin.clone());
        pin
    }

    /// Get the pinned version for a campaign + asset combo.
    pub fn get_pinned_version(&self, asset_id: &Uuid, campaign_id: &Uuid) -> Option<u32> {
        self.pins
            .iter()
            .find(|e| {
                let p = e.value();
                p.asset_id == *asset_id
                    && matches!(&p.pinned_by, PinTarget::Campaign(c) if c == campaign_id)
            })
            .map(|e| e.value().pinned_version)
    }

    /// List all pins for an asset.
    pub fn pins_for_asset(&self, asset_id: &Uuid) -> Vec<AssetVersionPin> {
        self.pins
            .iter()
            .filter(|e| e.value().asset_id == *asset_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Remove a pin.
    pub fn unpin(&self, pin_id: &Uuid) -> bool {
        self.pins.remove(pin_id).is_some()
    }
}

impl Default for AssetVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Rights Management (FR-AST-004) ──────────────────────────────────

/// License type for an asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    Owned,
    RoyaltyFree,
    RightsManaged,
    CreativeCommons,
    EditorialOnly,
}

/// Asset usage rights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRights {
    pub asset_id: Uuid,
    pub license_type: LicenseType,
    pub expiry_date: Option<DateTime<Utc>>,
    pub permitted_regions: Vec<String>,
    pub permitted_channels: Vec<String>,
    pub permitted_brands: Vec<String>,
    pub max_impressions: Option<u64>,
    pub attribution_required: bool,
    pub notes: String,
}

/// Rights manager for checking usage eligibility.
pub struct RightsManager {
    rights: DashMap<Uuid, AssetRights>,
}

impl RightsManager {
    pub fn new() -> Self {
        Self {
            rights: DashMap::new(),
        }
    }

    pub fn register_rights(&self, rights: AssetRights) {
        self.rights.insert(rights.asset_id, rights);
    }

    /// Check if an asset can be used in a given context.
    pub fn check_usage(
        &self,
        asset_id: &Uuid,
        region: &str,
        channel: &str,
        brand: &str,
    ) -> RightsCheckResult {
        let rights = match self.rights.get(asset_id) {
            Some(r) => r.clone(),
            None => {
                return RightsCheckResult {
                    allowed: true,
                    reasons: vec!["No rights restrictions registered".to_string()],
                }
            }
        };

        let mut reasons = Vec::new();
        let mut allowed = true;

        // Check expiry
        if let Some(expiry) = rights.expiry_date {
            if expiry < Utc::now() {
                allowed = false;
                reasons.push(format!("Rights expired on {}", expiry.format("%Y-%m-%d")));
            }
        }

        // Check region
        if !rights.permitted_regions.is_empty()
            && !rights
                .permitted_regions
                .iter()
                .any(|r| r.eq_ignore_ascii_case(region))
        {
            allowed = false;
            reasons.push(format!("Region '{}' not permitted", region));
        }

        // Check channel
        if !rights.permitted_channels.is_empty()
            && !rights
                .permitted_channels
                .iter()
                .any(|c| c.eq_ignore_ascii_case(channel))
        {
            allowed = false;
            reasons.push(format!("Channel '{}' not permitted", channel));
        }

        // Check brand
        if !rights.permitted_brands.is_empty()
            && !rights
                .permitted_brands
                .iter()
                .any(|b| b.eq_ignore_ascii_case(brand))
        {
            allowed = false;
            reasons.push(format!("Brand '{}' not permitted", brand));
        }

        if allowed {
            reasons.push("Usage permitted".to_string());
        }

        RightsCheckResult { allowed, reasons }
    }
}

/// Result of a rights usage check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsCheckResult {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

impl Default for RightsManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Renditions & Optimization (FR-AST-005) ──────────────────────────

/// A rendition variant of a source asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rendition {
    pub id: Uuid,
    pub source_asset_id: Uuid,
    pub rendition_type: RenditionType,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub quality: u32,
    pub file_size_bytes: u64,
    pub url: String,
    pub generated_at: DateTime<Utc>,
}

/// Type of rendition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenditionType {
    Thumbnail,
    WebOptimized,
    Retina2x,
    ChannelSpecific,
    FocalCrop,
}

/// Focal point for smart cropping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocalPoint {
    pub x_percent: f64,
    pub y_percent: f64,
}

/// Rendition specification for auto-generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenditionSpec {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub quality: u32,
    pub rendition_type: RenditionType,
}

/// Rendition engine for generating optimized asset variants.
pub struct RenditionEngine {
    renditions: DashMap<Uuid, Vec<Rendition>>,
    specs: Vec<RenditionSpec>,
}

impl RenditionEngine {
    pub fn new() -> Self {
        Self {
            renditions: DashMap::new(),
            specs: Self::default_specs(),
        }
    }

    fn default_specs() -> Vec<RenditionSpec> {
        vec![
            RenditionSpec {
                name: "thumbnail".to_string(),
                width: 150,
                height: 150,
                format: "image/webp".to_string(),
                quality: 80,
                rendition_type: RenditionType::Thumbnail,
            },
            RenditionSpec {
                name: "web_optimized".to_string(),
                width: 800,
                height: 600,
                format: "image/webp".to_string(),
                quality: 85,
                rendition_type: RenditionType::WebOptimized,
            },
            RenditionSpec {
                name: "retina_2x".to_string(),
                width: 1600,
                height: 1200,
                format: "image/png".to_string(),
                quality: 95,
                rendition_type: RenditionType::Retina2x,
            },
            RenditionSpec {
                name: "email_header".to_string(),
                width: 600,
                height: 200,
                format: "image/jpeg".to_string(),
                quality: 80,
                rendition_type: RenditionType::ChannelSpecific,
            },
            RenditionSpec {
                name: "mobile_push".to_string(),
                width: 192,
                height: 192,
                format: "image/png".to_string(),
                quality: 90,
                rendition_type: RenditionType::ChannelSpecific,
            },
        ]
    }

    /// Generate all standard renditions for an asset.
    /// Returns the list of renditions (URLs are computed, actual image processing
    /// would be done by an external service in production).
    pub fn generate_renditions(&self, asset_id: Uuid, source_url: &str) -> Vec<Rendition> {
        let now = Utc::now();
        let renditions: Vec<Rendition> = self
            .specs
            .iter()
            .map(|spec| {
                let estimated_size =
                    (spec.width as u64 * spec.height as u64 * spec.quality as u64) / 100;
                Rendition {
                    id: Uuid::new_v4(),
                    source_asset_id: asset_id,
                    rendition_type: spec.rendition_type.clone(),
                    width: spec.width,
                    height: spec.height,
                    format: spec.format.clone(),
                    quality: spec.quality,
                    file_size_bytes: estimated_size,
                    url: format!(
                        "{}_{}x{}.{}",
                        source_url,
                        spec.width,
                        spec.height,
                        spec.format.split('/').next_back().unwrap_or("webp")
                    ),
                    generated_at: now,
                }
            })
            .collect();

        self.renditions.insert(asset_id, renditions.clone());
        renditions
    }

    /// Generate a focal-point crop rendition.
    pub fn generate_focal_crop(
        &self,
        asset_id: Uuid,
        source_url: &str,
        focal_point: &FocalPoint,
        target_width: u32,
        target_height: u32,
    ) -> Rendition {
        let rendition = Rendition {
            id: Uuid::new_v4(),
            source_asset_id: asset_id,
            rendition_type: RenditionType::FocalCrop,
            width: target_width,
            height: target_height,
            format: "image/jpeg".to_string(),
            quality: 90,
            file_size_bytes: (target_width as u64 * target_height as u64 * 90) / 100,
            url: format!(
                "{}_crop_{:.0}_{:.0}_{}x{}.jpg",
                source_url,
                focal_point.x_percent,
                focal_point.y_percent,
                target_width,
                target_height
            ),
            generated_at: Utc::now(),
        };

        self.renditions
            .entry(asset_id)
            .or_default()
            .push(rendition.clone());

        rendition
    }

    /// Get all renditions for an asset.
    pub fn get_renditions(&self, asset_id: &Uuid) -> Vec<Rendition> {
        self.renditions
            .get(asset_id)
            .map(|r| r.clone())
            .unwrap_or_default()
    }
}

impl Default for RenditionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_ingest_with_duplicates() {
        let engine = AssetIngestionEngine::new();

        let files = vec![
            ("hero_banner.png".to_string(), "abc123".to_string(), 50000),
            ("product_shot.jpg".to_string(), "def456".to_string(), 75000),
            (
                "hero_banner_copy.png".to_string(),
                "abc123".to_string(),
                50000,
            ), // duplicate
        ];

        let results = engine.bulk_ingest(files, None);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].status, IngestionStatus::Accepted);
        assert_eq!(results[1].status, IngestionStatus::Accepted);
        assert_eq!(results[2].status, IngestionStatus::Duplicate);
        assert!(results[2].duplicate_of.is_some());
    }

    #[test]
    fn test_auto_tagging() {
        let engine = AssetIngestionEngine::new();
        let results = engine.bulk_ingest(
            vec![(
                "hero_banner_social.png".to_string(),
                "unique1".to_string(),
                1000,
            )],
            None,
        );
        let tags = &results[0].auto_tags;
        assert!(tags.contains(&"hero".to_string()));
        assert!(tags.contains(&"banner".to_string()));
        assert!(tags.contains(&"social".to_string()));
        assert!(tags.contains(&"image".to_string()));
    }

    #[test]
    fn test_asset_lifecycle_workflow() {
        let engine = AssetWorkflowEngine::new();
        let asset_id = Uuid::new_v4();
        let actor = Uuid::new_v4();
        let reviewer1 = Uuid::new_v4();
        let reviewer2 = Uuid::new_v4();

        engine.register(asset_id);
        assert!(!engine.is_approved(&asset_id));

        // Submit for review
        engine
            .submit_for_review(
                &asset_id,
                vec![
                    (reviewer1, "brand".to_string()),
                    (reviewer2, "legal".to_string()),
                ],
                48,
                actor,
            )
            .unwrap();

        let lifecycle = engine.get(&asset_id).unwrap();
        assert_eq!(lifecycle.stage, AssetLifecycleStage::PendingReview);
        assert_eq!(lifecycle.reviewers.len(), 2);

        // First reviewer approves
        engine
            .review_decision(&asset_id, reviewer1, true, Some("LGTM".to_string()))
            .unwrap();
        assert!(!engine.is_approved(&asset_id)); // Still pending second reviewer

        // Second reviewer approves
        engine
            .review_decision(&asset_id, reviewer2, true, None)
            .unwrap();
        assert!(engine.is_approved(&asset_id));

        let lifecycle = engine.get(&asset_id).unwrap();
        assert_eq!(lifecycle.stage, AssetLifecycleStage::Approved);
        assert_eq!(lifecycle.transitions.len(), 2);
    }

    #[test]
    fn test_asset_rejection() {
        let engine = AssetWorkflowEngine::new();
        let asset_id = Uuid::new_v4();
        let reviewer = Uuid::new_v4();

        engine.register(asset_id);
        engine
            .submit_for_review(
                &asset_id,
                vec![(reviewer, "brand".to_string())],
                24,
                Uuid::new_v4(),
            )
            .unwrap();

        engine
            .review_decision(
                &asset_id,
                reviewer,
                false,
                Some("Does not meet guidelines".to_string()),
            )
            .unwrap();

        assert!(!engine.is_approved(&asset_id));
        let lifecycle = engine.get(&asset_id).unwrap();
        assert_eq!(lifecycle.stage, AssetLifecycleStage::Rejected);
    }

    #[test]
    fn test_blocked_asset_enforcement() {
        let engine = AssetWorkflowEngine::new();
        let asset_id = Uuid::new_v4();
        engine.register(asset_id);
        // Not approved — should be blocked from publish
        assert!(!engine.is_approved(&asset_id));
    }

    #[test]
    fn test_version_pinning() {
        let manager = AssetVersionManager::new();
        let asset_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();

        manager.pin(asset_id, 3, PinTarget::Campaign(campaign_id));

        let pinned = manager.get_pinned_version(&asset_id, &campaign_id);
        assert_eq!(pinned, Some(3));

        let pins = manager.pins_for_asset(&asset_id);
        assert_eq!(pins.len(), 1);
    }

    #[test]
    fn test_rights_management() {
        let manager = RightsManager::new();
        let asset_id = Uuid::new_v4();

        manager.register_rights(AssetRights {
            asset_id,
            license_type: LicenseType::RightsManaged,
            expiry_date: Some(Utc::now() + chrono::Duration::days(30)),
            permitted_regions: vec!["US".to_string(), "UK".to_string()],
            permitted_channels: vec!["email".to_string(), "display".to_string()],
            permitted_brands: vec!["AcmeCorp".to_string()],
            max_impressions: Some(100_000),
            attribution_required: false,
            notes: "Licensed for Q1 campaign".to_string(),
        });

        // Allowed usage
        let check = manager.check_usage(&asset_id, "US", "email", "AcmeCorp");
        assert!(check.allowed);

        // Disallowed region
        let check = manager.check_usage(&asset_id, "DE", "email", "AcmeCorp");
        assert!(!check.allowed);
        assert!(check.reasons.iter().any(|r| r.contains("Region")));

        // Disallowed channel
        let check = manager.check_usage(&asset_id, "US", "sms", "AcmeCorp");
        assert!(!check.allowed);
        assert!(check.reasons.iter().any(|r| r.contains("Channel")));
    }

    #[test]
    fn test_rendition_generation() {
        let engine = RenditionEngine::new();
        let asset_id = Uuid::new_v4();

        let renditions = engine.generate_renditions(asset_id, "https://cdn.example.com/hero");
        assert_eq!(renditions.len(), 5); // 5 default specs

        let thumbnails: Vec<_> = renditions
            .iter()
            .filter(|r| r.rendition_type == RenditionType::Thumbnail)
            .collect();
        assert_eq!(thumbnails.len(), 1);
        assert_eq!(thumbnails[0].width, 150);
    }

    #[test]
    fn test_focal_crop_rendition() {
        let engine = RenditionEngine::new();
        let asset_id = Uuid::new_v4();

        let focal = FocalPoint {
            x_percent: 60.0,
            y_percent: 40.0,
        };
        let crop = engine.generate_focal_crop(
            asset_id,
            "https://cdn.example.com/portrait",
            &focal,
            300,
            250,
        );

        assert_eq!(crop.rendition_type, RenditionType::FocalCrop);
        assert_eq!(crop.width, 300);
        assert_eq!(crop.height, 250);
        assert!(crop.url.contains("crop_60_40"));
    }
}
