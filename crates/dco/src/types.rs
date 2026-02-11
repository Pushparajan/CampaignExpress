use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A DCO template defining the structure and rules for dynamic creative assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcoTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub components: Vec<TemplateComponent>,
    pub rules: Vec<AssemblyRule>,
    pub status: TemplateStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single component slot within a template (e.g. headline, hero image).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateComponent {
    pub id: Uuid,
    pub component_type: ComponentType,
    pub variants: Vec<ComponentVariant>,
    pub required: bool,
}

/// The kind of creative component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Headline,
    SubHeadline,
    BodyText,
    HeroImage,
    Logo,
    Cta,
    BackgroundColor,
    BorderStyle,
    ProductImage,
    PriceTag,
    DiscountBadge,
}

/// One possible variant for a component slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVariant {
    pub id: Uuid,
    pub name: String,
    pub content: String,
    pub asset_url: Option<String>,
    pub metadata: serde_json::Value,
    pub performance: VariantPerformance,
}

/// Aggregate performance metrics for a variant.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VariantPerformance {
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub ctr: f64,
    pub cvr: f64,
    pub revenue: f64,
}

/// Lifecycle status of a template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

/// A rule that influences which variants are preferred during assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyRule {
    pub id: Uuid,
    pub name: String,
    pub condition: String,
    pub component_id: Uuid,
    pub preferred_variants: Vec<Uuid>,
    pub priority: u32,
}

/// A fully assembled creative with scored component selections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledCreative {
    pub id: Uuid,
    pub template_id: Uuid,
    pub selected_components: HashMap<String, SelectedComponent>,
    pub score: f32,
    pub predicted_ctr: f32,
    pub assembly_latency_us: u64,
    pub timestamp: DateTime<Utc>,
}

/// A single component selection within an assembled creative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedComponent {
    pub component_id: Uuid,
    pub variant_id: Uuid,
    pub content: String,
    pub asset_url: Option<String>,
}

/// Request to score and assemble creatives from a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcoScoreRequest {
    pub template_id: Uuid,
    pub user_segments: Vec<u32>,
    pub context: serde_json::Value,
    pub max_variants: usize,
}

/// Response containing assembled and scored creatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcoScoreResponse {
    pub assembled_creatives: Vec<AssembledCreative>,
    pub total_combinations: u64,
    pub scored_combinations: u64,
    pub latency_us: u64,
}
