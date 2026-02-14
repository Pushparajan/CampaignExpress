//! Creative Ops Studio: DCO template management, placement-aware constraints,
//! creative performance insights, and creative-level approval workflows.
//!
//! Addresses FR-DCO-001 through FR-DCO-004.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── DCO Template Studio (FR-DCO-001) ─────────────────────────────────

/// A named slot in a DCO template (headline, hero, CTA, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSlot {
    pub id: Uuid,
    pub name: String,
    pub slot_type: SlotType,
    pub required: bool,
    pub max_variants: u32,
    pub constraints: SlotConstraints,
}

/// Type of component slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Headline,
    SubHeadline,
    HeroImage,
    ProductImage,
    Cta,
    DiscountBadge,
    Logo,
    BodyCopy,
    BackgroundColor,
    BackgroundImage,
}

/// Constraints for a component slot (text limits, image sizes, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotConstraints {
    pub max_text_length: Option<u32>,
    pub min_image_width: Option<u32>,
    pub min_image_height: Option<u32>,
    pub max_file_size_bytes: Option<u64>,
    pub allowed_mime_types: Vec<String>,
}

/// A variant assigned to a slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotVariant {
    pub id: Uuid,
    pub slot_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub text_content: Option<String>,
    pub color_value: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// A full DCO template in the studio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub slots: Vec<ComponentSlot>,
    pub variants: Vec<SlotVariant>,
    pub max_combinations: u32,
    pub version: u32,
    pub status: StudioTemplateStatus,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Template status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioTemplateStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

/// Validation result for a studio template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub total_combinations: u64,
}

/// Studio for managing DCO templates.
pub struct DcoStudio {
    templates: DashMap<Uuid, StudioTemplate>,
}

impl DcoStudio {
    pub fn new() -> Self {
        Self {
            templates: DashMap::new(),
        }
    }

    pub fn create_template(
        &self,
        name: String,
        description: String,
        max_combinations: u32,
        user_id: Uuid,
    ) -> StudioTemplate {
        let now = Utc::now();
        let template = StudioTemplate {
            id: Uuid::new_v4(),
            name,
            description,
            slots: Vec::new(),
            variants: Vec::new(),
            max_combinations,
            version: 1,
            status: StudioTemplateStatus::Draft,
            created_by: user_id,
            created_at: now,
            updated_at: now,
        };
        self.templates.insert(template.id, template.clone());
        template
    }

    /// Add a component slot to a template.
    pub fn add_slot(
        &self,
        template_id: &Uuid,
        slot: ComponentSlot,
    ) -> Result<StudioTemplate, String> {
        let mut entry = self
            .templates
            .get_mut(template_id)
            .ok_or("Template not found")?;
        entry.slots.push(slot);
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Add a variant to a slot.
    pub fn add_variant(
        &self,
        template_id: &Uuid,
        variant: SlotVariant,
    ) -> Result<StudioTemplate, String> {
        let mut entry = self
            .templates
            .get_mut(template_id)
            .ok_or("Template not found")?;

        // Check that the slot exists
        let slot = entry
            .slots
            .iter()
            .find(|s| s.id == variant.slot_id)
            .ok_or("Slot not found")?;

        // Check max variants for this slot
        let current_count = entry
            .variants
            .iter()
            .filter(|v| v.slot_id == variant.slot_id)
            .count() as u32;
        if current_count >= slot.max_variants {
            return Err(format!(
                "Slot '{}' already has {}/{} variants",
                slot.name, current_count, slot.max_variants
            ));
        }

        entry.variants.push(variant);
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Validate a template: required slots, max combinations, variant constraints.
    pub fn validate(&self, template_id: &Uuid) -> Result<TemplateValidation, String> {
        let entry = self
            .templates
            .get(template_id)
            .ok_or("Template not found")?;

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required slots have at least one variant
        for slot in &entry.slots {
            let variant_count = entry
                .variants
                .iter()
                .filter(|v| v.slot_id == slot.id)
                .count();

            if slot.required && variant_count == 0 {
                errors.push(format!("Required slot '{}' has no variants", slot.name));
            }

            // Check text length constraints
            if let Some(max_len) = slot.constraints.max_text_length {
                for variant in entry.variants.iter().filter(|v| v.slot_id == slot.id) {
                    if let Some(ref text) = variant.text_content {
                        if text.len() as u32 > max_len {
                            errors.push(format!(
                                "Variant in slot '{}' exceeds max text length ({} > {})",
                                slot.name,
                                text.len(),
                                max_len
                            ));
                        }
                    }
                }
            }
        }

        // Compute total combinations
        let mut total_combinations: u64 = 1;
        for slot in &entry.slots {
            let count = entry
                .variants
                .iter()
                .filter(|v| v.slot_id == slot.id)
                .count()
                .max(1) as u64;
            total_combinations = total_combinations.saturating_mul(count);
        }

        if total_combinations > entry.max_combinations as u64 {
            warnings.push(format!(
                "Total combinations ({}) exceeds max ({})",
                total_combinations, entry.max_combinations
            ));
        }

        Ok(TemplateValidation {
            valid: errors.is_empty(),
            errors,
            warnings,
            total_combinations,
        })
    }

    pub fn get_template(&self, id: &Uuid) -> Option<StudioTemplate> {
        self.templates.get(id).map(|t| t.clone())
    }

    /// Preview the top K assembled creatives.
    pub fn preview_top_k(
        &self,
        template_id: &Uuid,
        k: usize,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let entry = self
            .templates
            .get(template_id)
            .ok_or("Template not found")?;

        let mut previews = Vec::new();

        // Build one creative per combination (limited to k)
        let slot_variants: Vec<Vec<&SlotVariant>> = entry
            .slots
            .iter()
            .map(|slot| {
                entry
                    .variants
                    .iter()
                    .filter(|v| v.slot_id == slot.id)
                    .collect()
            })
            .collect();

        // Simple: take first variant from each slot, cycling
        for i in 0..k {
            let mut preview: HashMap<String, String> = HashMap::new();
            for (si, slot) in entry.slots.iter().enumerate() {
                if let Some(variants) = slot_variants.get(si) {
                    if !variants.is_empty() {
                        let variant = &variants[i % variants.len()];
                        let value = variant
                            .text_content
                            .clone()
                            .or_else(|| variant.color_value.clone())
                            .or_else(|| variant.asset_id.map(|id| id.to_string()))
                            .unwrap_or_default();
                        preview.insert(slot.name.clone(), value);
                    }
                }
            }
            if !preview.is_empty() {
                previews.push(preview);
            }
        }

        Ok(previews)
    }
}

impl Default for DcoStudio {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Placement Constraints (FR-DCO-002) ───────────────────────────────

/// A placement definition with size, channel, and DSP requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementSpec {
    pub id: Uuid,
    pub name: String,
    pub channel: String,
    pub dsp: Option<String>,
    pub width: u32,
    pub height: u32,
    pub safe_area_margin: u32,
    pub max_text_chars: Option<u32>,
    pub max_file_size_bytes: u64,
    pub allowed_formats: Vec<String>,
    pub min_cta_size_px: Option<u32>,
}

/// Result of validating a creative against a placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementValidation {
    pub placement_name: String,
    pub passed: bool,
    pub violations: Vec<String>,
}

/// Registry of placement constraints for size/format validation.
pub struct PlacementRegistry {
    placements: DashMap<Uuid, PlacementSpec>,
}

impl PlacementRegistry {
    pub fn new() -> Self {
        let registry = Self {
            placements: DashMap::new(),
        };
        registry.seed_standard_placements();
        registry
    }

    fn seed_standard_placements(&self) {
        let specs = vec![
            PlacementSpec {
                id: Uuid::new_v4(),
                name: "Leaderboard".to_string(),
                channel: "display".to_string(),
                dsp: None,
                width: 728,
                height: 90,
                safe_area_margin: 5,
                max_text_chars: Some(90),
                max_file_size_bytes: 150_000,
                allowed_formats: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/gif".to_string(),
                ],
                min_cta_size_px: Some(44),
            },
            PlacementSpec {
                id: Uuid::new_v4(),
                name: "Medium Rectangle".to_string(),
                channel: "display".to_string(),
                dsp: None,
                width: 300,
                height: 250,
                safe_area_margin: 10,
                max_text_chars: Some(150),
                max_file_size_bytes: 150_000,
                allowed_formats: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/gif".to_string(),
                ],
                min_cta_size_px: Some(44),
            },
            PlacementSpec {
                id: Uuid::new_v4(),
                name: "Mobile Banner".to_string(),
                channel: "mobile".to_string(),
                dsp: None,
                width: 320,
                height: 50,
                safe_area_margin: 3,
                max_text_chars: Some(40),
                max_file_size_bytes: 100_000,
                allowed_formats: vec!["image/jpeg".to_string(), "image/png".to_string()],
                min_cta_size_px: Some(36),
            },
            PlacementSpec {
                id: Uuid::new_v4(),
                name: "Facebook Feed".to_string(),
                channel: "social".to_string(),
                dsp: Some("Meta".to_string()),
                width: 1200,
                height: 628,
                safe_area_margin: 20,
                max_text_chars: Some(125),
                max_file_size_bytes: 5_000_000,
                allowed_formats: vec!["image/jpeg".to_string(), "image/png".to_string()],
                min_cta_size_px: None,
            },
            PlacementSpec {
                id: Uuid::new_v4(),
                name: "Instagram Story".to_string(),
                channel: "social".to_string(),
                dsp: Some("Meta".to_string()),
                width: 1080,
                height: 1920,
                safe_area_margin: 50,
                max_text_chars: Some(100),
                max_file_size_bytes: 10_000_000,
                allowed_formats: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "video/mp4".to_string(),
                ],
                min_cta_size_px: None,
            },
        ];

        for spec in specs {
            self.placements.insert(spec.id, spec);
        }
    }

    /// Validate a creative against a placement.
    pub fn validate_creative(
        &self,
        placement_id: &Uuid,
        creative_width: u32,
        creative_height: u32,
        file_size: u64,
        mime_type: &str,
        text_length: Option<u32>,
    ) -> Result<PlacementValidation, String> {
        let spec = self
            .placements
            .get(placement_id)
            .ok_or("Placement not found")?;

        let mut violations = Vec::new();

        if creative_width != spec.width || creative_height != spec.height {
            violations.push(format!(
                "Size {}x{} does not match required {}x{}",
                creative_width, creative_height, spec.width, spec.height
            ));
        }

        if file_size > spec.max_file_size_bytes {
            violations.push(format!(
                "File size {} exceeds max {}",
                file_size, spec.max_file_size_bytes
            ));
        }

        if !spec.allowed_formats.contains(&mime_type.to_string()) {
            violations.push(format!(
                "Format '{}' not allowed for this placement",
                mime_type
            ));
        }

        if let (Some(max_chars), Some(text_len)) = (spec.max_text_chars, text_length) {
            if text_len > max_chars {
                violations.push(format!(
                    "Text length {} exceeds max {} for placement",
                    text_len, max_chars
                ));
            }
        }

        Ok(PlacementValidation {
            placement_name: spec.name.clone(),
            passed: violations.is_empty(),
            violations,
        })
    }

    /// Auto-flag all placements a creative violates.
    pub fn auto_flag(
        &self,
        creative_width: u32,
        creative_height: u32,
        file_size: u64,
        mime_type: &str,
    ) -> Vec<PlacementValidation> {
        let specs: Vec<PlacementSpec> = self.placements.iter().map(|e| e.value().clone()).collect();

        specs
            .iter()
            .map(|spec| {
                let mut violations = Vec::new();
                if creative_width != spec.width || creative_height != spec.height {
                    violations.push(format!(
                        "Size {}x{} does not match {}x{}",
                        creative_width, creative_height, spec.width, spec.height
                    ));
                }
                if file_size > spec.max_file_size_bytes {
                    violations.push(format!(
                        "File too large ({} > {})",
                        file_size, spec.max_file_size_bytes
                    ));
                }
                if !spec.allowed_formats.contains(&mime_type.to_string()) {
                    violations.push(format!("Format '{}' not allowed", mime_type));
                }
                PlacementValidation {
                    placement_name: spec.name.clone(),
                    passed: violations.is_empty(),
                    violations,
                }
            })
            .collect()
    }

    pub fn list_placements(&self) -> Vec<PlacementSpec> {
        self.placements.iter().map(|e| e.value().clone()).collect()
    }
}

impl Default for PlacementRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Creative Performance (FR-DCO-003) ────────────────────────────────

/// Performance metrics for a creative variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantPerformance {
    pub variant_id: Uuid,
    pub variant_name: String,
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub ctr: f64,
    pub cvr: f64,
    pub spend: f64,
    pub revenue: f64,
    pub roas: f64,
}

/// Creative fatigue indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatigueIndicator {
    pub variant_id: Uuid,
    pub variant_name: String,
    pub days_active: u32,
    pub ctr_trend: f64,
    pub fatigue_score: f64,
    pub recommended_action: String,
}

/// Creative performance tracker.
pub struct CreativePerformanceTracker {
    metrics: DashMap<Uuid, VariantPerformance>,
}

impl CreativePerformanceTracker {
    pub fn new() -> Self {
        Self {
            metrics: DashMap::new(),
        }
    }

    /// Record performance data for a variant.
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &self,
        variant_id: Uuid,
        variant_name: String,
        impressions: u64,
        clicks: u64,
        conversions: u64,
        spend: f64,
        revenue: f64,
    ) {
        let ctr = if impressions > 0 {
            clicks as f64 / impressions as f64
        } else {
            0.0
        };
        let cvr = if clicks > 0 {
            conversions as f64 / clicks as f64
        } else {
            0.0
        };
        let roas = if spend > 0.0 { revenue / spend } else { 0.0 };

        let perf = VariantPerformance {
            variant_id,
            variant_name,
            impressions,
            clicks,
            conversions,
            ctr,
            cvr,
            spend,
            revenue,
            roas,
        };
        self.metrics.insert(variant_id, perf);
    }

    /// Get performance for a specific variant.
    pub fn get(&self, variant_id: &Uuid) -> Option<VariantPerformance> {
        self.metrics.get(variant_id).map(|e| e.clone())
    }

    /// Rank all variants by CTR descending.
    pub fn rank_by_ctr(&self) -> Vec<VariantPerformance> {
        let mut all: Vec<VariantPerformance> =
            self.metrics.iter().map(|e| e.value().clone()).collect();
        all.sort_by(|a, b| {
            b.ctr
                .partial_cmp(&a.ctr)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all
    }

    /// Detect creative fatigue (declining CTR trend).
    pub fn detect_fatigue(
        &self,
        variant_id: Uuid,
        variant_name: &str,
        days_active: u32,
        ctr_history: &[f64],
    ) -> FatigueIndicator {
        let ctr_trend = if ctr_history.len() >= 2 {
            let last = *ctr_history.last().unwrap_or(&0.0);
            let first = *ctr_history.first().unwrap_or(&0.0);
            if first > 0.0 {
                (last - first) / first
            } else {
                0.0
            }
        } else {
            0.0
        };

        let fatigue_score = if ctr_trend < -0.2 && days_active > 7 {
            0.8 + (-ctr_trend - 0.2).min(0.2)
        } else if ctr_trend < -0.1 && days_active > 14 {
            0.5
        } else {
            0.1
        };

        let recommended_action = if fatigue_score > 0.7 {
            "Refresh creative immediately — significant CTR decline".to_string()
        } else if fatigue_score > 0.4 {
            "Consider refreshing creative — moderate decline detected".to_string()
        } else {
            "No action needed — performance stable".to_string()
        };

        FatigueIndicator {
            variant_id,
            variant_name: variant_name.to_string(),
            days_active,
            ctr_trend,
            fatigue_score,
            recommended_action,
        }
    }
}

impl Default for CreativePerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Creative Approvals (FR-DCO-004) ──────────────────────────────────

/// Creative-level approval status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreativeApprovalStatus {
    Draft,
    PendingReview,
    ChangesRequested,
    Approved,
    Rejected,
}

/// A creative-level approval request (separate from campaign approvals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeApproval {
    pub id: Uuid,
    pub creative_id: Uuid,
    pub creative_name: String,
    pub status: CreativeApprovalStatus,
    pub submitted_by: Uuid,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_role: Option<String>,
    pub decision_reason: Option<String>,
    pub change_requests: Vec<ChangeRequest>,
    pub submitted_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

/// A specific change request on a creative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub id: Uuid,
    pub field: String,
    pub description: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
}

/// Creative approval workflow engine.
pub struct CreativeApprovalEngine {
    approvals: DashMap<Uuid, CreativeApproval>,
}

impl CreativeApprovalEngine {
    pub fn new() -> Self {
        Self {
            approvals: DashMap::new(),
        }
    }

    /// Submit a creative for review.
    pub fn submit_for_review(
        &self,
        creative_id: Uuid,
        creative_name: String,
        submitted_by: Uuid,
        reviewer_id: Uuid,
        reviewer_role: String,
    ) -> CreativeApproval {
        let approval = CreativeApproval {
            id: Uuid::new_v4(),
            creative_id,
            creative_name,
            status: CreativeApprovalStatus::PendingReview,
            submitted_by,
            reviewer_id: Some(reviewer_id),
            reviewer_role: Some(reviewer_role),
            decision_reason: None,
            change_requests: Vec::new(),
            submitted_at: Utc::now(),
            decided_at: None,
        };
        self.approvals.insert(approval.id, approval.clone());
        approval
    }

    /// Approve a creative.
    pub fn approve(
        &self,
        approval_id: &Uuid,
        reason: Option<String>,
    ) -> Result<CreativeApproval, String> {
        let mut entry = self
            .approvals
            .get_mut(approval_id)
            .ok_or("Approval not found")?;
        if entry.status != CreativeApprovalStatus::PendingReview {
            return Err("Creative is not pending review".to_string());
        }
        entry.status = CreativeApprovalStatus::Approved;
        entry.decision_reason = reason;
        entry.decided_at = Some(Utc::now());
        Ok(entry.clone())
    }

    /// Reject a creative.
    pub fn reject(&self, approval_id: &Uuid, reason: String) -> Result<CreativeApproval, String> {
        let mut entry = self
            .approvals
            .get_mut(approval_id)
            .ok_or("Approval not found")?;
        if entry.status != CreativeApprovalStatus::PendingReview {
            return Err("Creative is not pending review".to_string());
        }
        entry.status = CreativeApprovalStatus::Rejected;
        entry.decision_reason = Some(reason);
        entry.decided_at = Some(Utc::now());
        Ok(entry.clone())
    }

    /// Request changes on a creative.
    pub fn request_changes(
        &self,
        approval_id: &Uuid,
        changes: Vec<(String, String)>,
    ) -> Result<CreativeApproval, String> {
        let mut entry = self
            .approvals
            .get_mut(approval_id)
            .ok_or("Approval not found")?;
        entry.status = CreativeApprovalStatus::ChangesRequested;
        for (field, description) in changes {
            entry.change_requests.push(ChangeRequest {
                id: Uuid::new_v4(),
                field,
                description,
                resolved: false,
                created_at: Utc::now(),
            });
        }
        entry.decided_at = Some(Utc::now());
        Ok(entry.clone())
    }

    /// Resolve a change request.
    pub fn resolve_change(
        &self,
        approval_id: &Uuid,
        change_id: &Uuid,
    ) -> Result<CreativeApproval, String> {
        let mut entry = self
            .approvals
            .get_mut(approval_id)
            .ok_or("Approval not found")?;
        if let Some(cr) = entry
            .change_requests
            .iter_mut()
            .find(|c| c.id == *change_id)
        {
            cr.resolved = true;
        }
        // If all changes resolved, move back to PendingReview
        if entry.change_requests.iter().all(|c| c.resolved) {
            entry.status = CreativeApprovalStatus::PendingReview;
        }
        Ok(entry.clone())
    }

    /// Get pending approvals for a reviewer.
    pub fn pending_for_reviewer(&self, reviewer_id: &Uuid) -> Vec<CreativeApproval> {
        self.approvals
            .iter()
            .filter(|e| {
                let a = e.value();
                a.reviewer_id.as_ref() == Some(reviewer_id)
                    && a.status == CreativeApprovalStatus::PendingReview
            })
            .map(|e| e.value().clone())
            .collect()
    }
}

impl Default for CreativeApprovalEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dco_studio_template() {
        let studio = DcoStudio::new();
        let user = Uuid::new_v4();
        let tmpl = studio.create_template(
            "Summer Sale".to_string(),
            "DCO template for summer campaign".to_string(),
            100,
            user,
        );

        let headline_slot = ComponentSlot {
            id: Uuid::new_v4(),
            name: "headline".to_string(),
            slot_type: SlotType::Headline,
            required: true,
            max_variants: 5,
            constraints: SlotConstraints {
                max_text_length: Some(50),
                min_image_width: None,
                min_image_height: None,
                max_file_size_bytes: None,
                allowed_mime_types: vec![],
            },
        };
        let slot_id = headline_slot.id;
        studio.add_slot(&tmpl.id, headline_slot).unwrap();

        // Add variants
        studio
            .add_variant(
                &tmpl.id,
                SlotVariant {
                    id: Uuid::new_v4(),
                    slot_id,
                    asset_id: None,
                    text_content: Some("Save Big This Summer!".to_string()),
                    color_value: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        studio
            .add_variant(
                &tmpl.id,
                SlotVariant {
                    id: Uuid::new_v4(),
                    slot_id,
                    asset_id: None,
                    text_content: Some("Hot Summer Deals".to_string()),
                    color_value: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        // Validate
        let validation = studio.validate(&tmpl.id).unwrap();
        assert!(validation.valid);
        assert_eq!(validation.total_combinations, 2);

        // Preview
        let previews = studio.preview_top_k(&tmpl.id, 3).unwrap();
        assert!(!previews.is_empty());
    }

    #[test]
    fn test_slot_max_variants() {
        let studio = DcoStudio::new();
        let tmpl = studio.create_template("Test".to_string(), "".to_string(), 10, Uuid::new_v4());
        let slot = ComponentSlot {
            id: Uuid::new_v4(),
            name: "cta".to_string(),
            slot_type: SlotType::Cta,
            required: true,
            max_variants: 1,
            constraints: SlotConstraints {
                max_text_length: Some(20),
                min_image_width: None,
                min_image_height: None,
                max_file_size_bytes: None,
                allowed_mime_types: vec![],
            },
        };
        let slot_id = slot.id;
        studio.add_slot(&tmpl.id, slot).unwrap();

        studio
            .add_variant(
                &tmpl.id,
                SlotVariant {
                    id: Uuid::new_v4(),
                    slot_id,
                    asset_id: None,
                    text_content: Some("Buy Now".to_string()),
                    color_value: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        let err = studio.add_variant(
            &tmpl.id,
            SlotVariant {
                id: Uuid::new_v4(),
                slot_id,
                asset_id: None,
                text_content: Some("Shop Now".to_string()),
                color_value: None,
                metadata: HashMap::new(),
            },
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_placement_validation() {
        let registry = PlacementRegistry::new();
        let placements = registry.list_placements();
        assert!(placements.len() >= 5);

        let leaderboard = placements.iter().find(|p| p.name == "Leaderboard").unwrap();

        // Valid creative
        let result = registry
            .validate_creative(&leaderboard.id, 728, 90, 100_000, "image/png", Some(50))
            .unwrap();
        assert!(result.passed);

        // Invalid size
        let result = registry
            .validate_creative(&leaderboard.id, 600, 90, 100_000, "image/png", None)
            .unwrap();
        assert!(!result.passed);
        assert!(result.violations[0].contains("Size"));
    }

    #[test]
    fn test_creative_fatigue_detection() {
        let tracker = CreativePerformanceTracker::new();
        let variant_id = Uuid::new_v4();

        // Declining CTR history
        let history = vec![0.05, 0.045, 0.038, 0.030, 0.025, 0.020, 0.015];
        let fatigue = tracker.detect_fatigue(variant_id, "Summer Banner v1", 21, &history);

        assert!(fatigue.fatigue_score > 0.5);
        assert!(fatigue.recommended_action.contains("Refresh"));
    }

    #[test]
    fn test_creative_approval_workflow() {
        let engine = CreativeApprovalEngine::new();
        let submitter = Uuid::new_v4();
        let reviewer = Uuid::new_v4();

        let approval = engine.submit_for_review(
            Uuid::new_v4(),
            "Hero Banner v3".to_string(),
            submitter,
            reviewer,
            "brand_manager".to_string(),
        );
        assert_eq!(approval.status, CreativeApprovalStatus::PendingReview);

        // Request changes
        let updated = engine
            .request_changes(
                &approval.id,
                vec![
                    (
                        "headline".to_string(),
                        "Too long — shorten to 30 chars".to_string(),
                    ),
                    (
                        "cta".to_string(),
                        "Use 'Shop Now' instead of 'Click Here'".to_string(),
                    ),
                ],
            )
            .unwrap();
        assert_eq!(updated.status, CreativeApprovalStatus::ChangesRequested);
        assert_eq!(updated.change_requests.len(), 2);

        // Resolve changes
        let cr_ids: Vec<Uuid> = updated.change_requests.iter().map(|c| c.id).collect();
        engine.resolve_change(&approval.id, &cr_ids[0]).unwrap();
        engine.resolve_change(&approval.id, &cr_ids[1]).unwrap();

        let updated = engine.approvals.get(&approval.id).unwrap().clone();
        assert_eq!(updated.status, CreativeApprovalStatus::PendingReview);

        // Approve
        let final_state = engine
            .approve(&approval.id, Some("Looks good!".to_string()))
            .unwrap();
        assert_eq!(final_state.status, CreativeApprovalStatus::Approved);
    }
}
