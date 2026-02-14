//! Marketer-first workspace UX: unified create flow, operator-grade calendar,
//! cross-channel preview/QA, bulk operations, and explainability surfaces.
//!
//! Addresses FR-UX-001 through FR-UX-005.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Campaign Types & Objectives ──────────────────────────────────────

/// The top-level campaign type chosen in the unified create flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CampaignType {
    /// Lifecycle messaging: email, SMS, push, in-app, WhatsApp, webhooks.
    Lifecycle,
    /// Paid media distribution: DSP placements, programmatic.
    PaidMedia,
    /// Combined lifecycle + paid media.
    Hybrid,
}

/// Business objective for a campaign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CampaignObjective {
    Awareness,
    Engagement,
    Conversion,
    Retention,
    Reactivation,
    Upsell,
    CrossSell,
}

/// Channel selection for lifecycle campaigns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleChannel {
    Email,
    Sms,
    Push,
    InApp,
    WhatsApp,
    Webhook,
    ContentCard,
}

/// Steps in the unified create-flow wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreateFlowStep {
    Objective,
    Audience,
    Content,
    Schedule,
    Governance,
    Review,
    Launch,
}

impl CreateFlowStep {
    /// Return the next step, or `None` if this is the last step.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Objective => Some(Self::Audience),
            Self::Audience => Some(Self::Content),
            Self::Content => Some(Self::Schedule),
            Self::Schedule => Some(Self::Governance),
            Self::Governance => Some(Self::Review),
            Self::Review => Some(Self::Launch),
            Self::Launch => None,
        }
    }

    /// Return the previous step, or `None` if this is the first step.
    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Objective => None,
            Self::Audience => Some(Self::Objective),
            Self::Content => Some(Self::Audience),
            Self::Schedule => Some(Self::Content),
            Self::Governance => Some(Self::Schedule),
            Self::Review => Some(Self::Governance),
            Self::Launch => Some(Self::Review),
        }
    }
}

// ─── Unified Create Flow (FR-UX-001) ──────────────────────────────────

/// Audience targeting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceConfig {
    pub segment_ids: Vec<Uuid>,
    pub exclusion_segment_ids: Vec<Uuid>,
    pub estimated_reach: u64,
    pub filters: HashMap<String, String>,
}

/// Schedule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub timezone: String,
    pub send_time_optimization: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub frequency_cap: Option<u32>,
    pub frequency_cap_window_hours: Option<u32>,
}

/// State of a campaign being built in the create-flow wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFlowState {
    pub id: Uuid,
    pub campaign_type: CampaignType,
    pub current_step: CreateFlowStep,
    pub objective: Option<CampaignObjective>,
    pub name: Option<String>,
    pub channels: Vec<LifecycleChannel>,
    pub audience: Option<AudienceConfig>,
    pub content_ids: Vec<Uuid>,
    pub creative_ids: Vec<Uuid>,
    pub schedule: Option<ScheduleConfig>,
    pub budget: Option<f64>,
    pub governance_approved: bool,
    pub completed_steps: Vec<CreateFlowStep>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Manages the unified create-flow wizard for lifecycle + paid media campaigns.
pub struct UnifiedCreateFlow {
    flows: DashMap<Uuid, CreateFlowState>,
}

impl UnifiedCreateFlow {
    pub fn new() -> Self {
        Self {
            flows: DashMap::new(),
        }
    }

    /// Start a new create-flow wizard session.
    pub fn start(&self, campaign_type: CampaignType, user_id: Uuid) -> CreateFlowState {
        let now = Utc::now();
        let state = CreateFlowState {
            id: Uuid::new_v4(),
            campaign_type,
            current_step: CreateFlowStep::Objective,
            objective: None,
            name: None,
            channels: Vec::new(),
            audience: None,
            content_ids: Vec::new(),
            creative_ids: Vec::new(),
            schedule: None,
            budget: None,
            governance_approved: false,
            completed_steps: Vec::new(),
            created_by: user_id,
            created_at: now,
            updated_at: now,
        };
        self.flows.insert(state.id, state.clone());
        state
    }

    /// Set the objective for a flow and advance to the next step.
    pub fn set_objective(
        &self,
        flow_id: &Uuid,
        objective: CampaignObjective,
        name: String,
    ) -> Result<CreateFlowState, String> {
        let mut entry = self
            .flows
            .get_mut(flow_id)
            .ok_or("Flow not found".to_string())?;
        if entry.current_step != CreateFlowStep::Objective {
            return Err("Not on the Objective step".to_string());
        }
        entry.objective = Some(objective);
        entry.name = Some(name);
        entry.completed_steps.push(CreateFlowStep::Objective);
        entry.current_step = CreateFlowStep::Audience;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Set audience targeting and advance.
    pub fn set_audience(
        &self,
        flow_id: &Uuid,
        audience: AudienceConfig,
    ) -> Result<CreateFlowState, String> {
        let mut entry = self
            .flows
            .get_mut(flow_id)
            .ok_or("Flow not found".to_string())?;
        if entry.current_step != CreateFlowStep::Audience {
            return Err("Not on the Audience step".to_string());
        }
        entry.audience = Some(audience);
        entry.completed_steps.push(CreateFlowStep::Audience);
        entry.current_step = CreateFlowStep::Content;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Set content/creative references and advance.
    pub fn set_content(
        &self,
        flow_id: &Uuid,
        channels: Vec<LifecycleChannel>,
        content_ids: Vec<Uuid>,
        creative_ids: Vec<Uuid>,
    ) -> Result<CreateFlowState, String> {
        let mut entry = self
            .flows
            .get_mut(flow_id)
            .ok_or("Flow not found".to_string())?;
        if entry.current_step != CreateFlowStep::Content {
            return Err("Not on the Content step".to_string());
        }
        entry.channels = channels;
        entry.content_ids = content_ids;
        entry.creative_ids = creative_ids;
        entry.completed_steps.push(CreateFlowStep::Content);
        entry.current_step = CreateFlowStep::Schedule;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Set schedule and advance.
    pub fn set_schedule(
        &self,
        flow_id: &Uuid,
        schedule: ScheduleConfig,
        budget: Option<f64>,
    ) -> Result<CreateFlowState, String> {
        let mut entry = self
            .flows
            .get_mut(flow_id)
            .ok_or("Flow not found".to_string())?;
        if entry.current_step != CreateFlowStep::Schedule {
            return Err("Not on the Schedule step".to_string());
        }
        entry.schedule = Some(schedule);
        entry.budget = budget;
        entry.completed_steps.push(CreateFlowStep::Schedule);
        entry.current_step = CreateFlowStep::Governance;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Get the current state of a flow.
    pub fn get(&self, flow_id: &Uuid) -> Option<CreateFlowState> {
        self.flows.get(flow_id).map(|e| e.clone())
    }

    /// Navigate back to a previous step (e.g. for edits).
    pub fn go_to_step(
        &self,
        flow_id: &Uuid,
        step: CreateFlowStep,
    ) -> Result<CreateFlowState, String> {
        let mut entry = self
            .flows
            .get_mut(flow_id)
            .ok_or("Flow not found".to_string())?;
        entry.current_step = step;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }
}

impl Default for UnifiedCreateFlow {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Operator-Grade Calendar (FR-UX-002) ──────────────────────────────

/// Filter criteria for the campaign calendar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalendarFilter {
    pub channels: Vec<LifecycleChannel>,
    pub objectives: Vec<CampaignObjective>,
    pub statuses: Vec<String>,
    pub owner_ids: Vec<Uuid>,
    pub min_budget: Option<f64>,
    pub max_budget: Option<f64>,
    pub tags: Vec<String>,
}

/// An enriched calendar entry for the operator calendar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorCalendarEntry {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub campaign_type: CampaignType,
    pub objective: CampaignObjective,
    pub channels: Vec<LifecycleChannel>,
    pub status: String,
    pub owner_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub budget: Option<f64>,
    pub color: String,
    pub tags: Vec<String>,
}

/// Operator-grade calendar with filtering by channel, objective, status, owner, budget.
pub struct OperatorCalendar {
    entries: DashMap<Uuid, OperatorCalendarEntry>,
}

impl OperatorCalendar {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }

    pub fn add_entry(&self, entry: OperatorCalendarEntry) {
        self.entries.insert(entry.id, entry);
    }

    /// Query entries within a date range, applying optional filters.
    pub fn query(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        filter: &CalendarFilter,
    ) -> Vec<OperatorCalendarEntry> {
        self.entries
            .iter()
            .filter(|e| {
                let entry = e.value();
                let in_range = entry.start_date >= from && entry.start_date <= to;
                if !in_range {
                    return false;
                }
                if !filter.channels.is_empty()
                    && !entry.channels.iter().any(|c| filter.channels.contains(c))
                {
                    return false;
                }
                if !filter.objectives.is_empty() && !filter.objectives.contains(&entry.objective) {
                    return false;
                }
                if !filter.statuses.is_empty() && !filter.statuses.contains(&entry.status) {
                    return false;
                }
                if !filter.owner_ids.is_empty() && !filter.owner_ids.contains(&entry.owner_id) {
                    return false;
                }
                if let Some(min) = filter.min_budget {
                    if entry.budget.unwrap_or(0.0) < min {
                        return false;
                    }
                }
                if let Some(max) = filter.max_budget {
                    if entry.budget.unwrap_or(0.0) > max {
                        return false;
                    }
                }
                true
            })
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn get_by_campaign(&self, campaign_id: &Uuid) -> Vec<OperatorCalendarEntry> {
        self.entries
            .iter()
            .filter(|e| e.value().campaign_id == *campaign_id)
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn remove(&self, id: &Uuid) -> bool {
        self.entries.remove(id).is_some()
    }
}

impl Default for OperatorCalendar {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Cross-Channel Preview (FR-UX-003) ────────────────────────────────

/// Email client preview target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmailClient {
    GmailDesktop,
    GmailMobile,
    OutlookDesktop,
    OutlookMobile,
    AppleMail,
    YahooMail,
    ThunderbirdDesktop,
}

/// Push notification OS preview target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PushOS {
    IOSLatest,
    IOSPrevious,
    AndroidLatest,
    AndroidPrevious,
}

/// SMS preview analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsPreview {
    pub original_text: String,
    pub character_count: usize,
    pub is_unicode: bool,
    pub segment_count: u32,
    pub shortened_links: Vec<(String, String)>,
    pub estimated_cost_segments: u32,
}

/// Ad placement preview result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdPreview {
    pub placement_name: String,
    pub width: u32,
    pub height: u32,
    pub safe_area: (u32, u32, u32, u32),
    pub cta_visible: bool,
    pub text_overflow: bool,
    pub file_size_ok: bool,
    pub warnings: Vec<String>,
}

/// Cross-channel preview engine for QA from a single place.
pub struct CrossChannelPreview;

impl CrossChannelPreview {
    /// Generate an SMS preview with segment analysis and link shortening.
    pub fn preview_sms(text: &str, link_shortener_domain: &str) -> SmsPreview {
        let is_unicode = !text.is_ascii();
        let chars_per_segment: usize = if is_unicode { 70 } else { 160 };
        let multi_segment_chars: usize = if is_unicode { 67 } else { 153 };

        // Detect and shorten links
        let mut shortened = Vec::new();
        let mut processed = text.to_string();
        for word in text.split_whitespace() {
            if word.starts_with("https://") || word.starts_with("http://") {
                let short = format!(
                    "{}/s/{}",
                    link_shortener_domain,
                    &Uuid::new_v4().to_string()[..8]
                );
                shortened.push((word.to_string(), short.clone()));
                processed = processed.replace(word, &short);
            }
        }

        let char_count = processed.chars().count();
        let segment_count = if char_count == 0 {
            0
        } else if char_count <= chars_per_segment {
            1
        } else {
            ((char_count as f64) / multi_segment_chars as f64).ceil() as u32
        };

        SmsPreview {
            original_text: text.to_string(),
            character_count: char_count,
            is_unicode,
            segment_count,
            shortened_links: shortened,
            estimated_cost_segments: segment_count,
        }
    }

    /// Validate an ad creative against placement constraints.
    #[allow(clippy::too_many_arguments)]
    pub fn preview_ad(
        placement_name: &str,
        width: u32,
        height: u32,
        required_width: u32,
        required_height: u32,
        safe_margin: u32,
        has_cta: bool,
        file_size_bytes: u64,
        max_file_size: u64,
    ) -> AdPreview {
        let mut warnings = Vec::new();

        if width != required_width || height != required_height {
            warnings.push(format!(
                "Size {}x{} does not match required {}x{}",
                width, height, required_width, required_height
            ));
        }

        let text_overflow = width < required_width;
        let file_size_ok = file_size_bytes <= max_file_size;
        if !file_size_ok {
            warnings.push(format!(
                "File size {} exceeds max {}",
                file_size_bytes, max_file_size
            ));
        }
        if !has_cta {
            warnings.push("No CTA detected in creative".to_string());
        }

        let safe_area = (
            safe_margin,
            safe_margin,
            width.saturating_sub(safe_margin),
            height.saturating_sub(safe_margin),
        );

        AdPreview {
            placement_name: placement_name.to_string(),
            width,
            height,
            safe_area,
            cta_visible: has_cta,
            text_overflow,
            file_size_ok,
            warnings,
        }
    }
}

// ─── Bulk Operations (FR-UX-004) ──────────────────────────────────────

/// Type of bulk operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkOperationType {
    Pause,
    Resume,
    Reschedule,
    CreativeSwap,
    UtmUpdate,
    Archive,
    TagUpdate,
}

/// Result of a single item in a bulk operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkItemResult {
    pub campaign_id: Uuid,
    pub success: bool,
    pub error: Option<String>,
}

/// A recorded bulk operation with audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperation {
    pub id: Uuid,
    pub operation_type: BulkOperationType,
    pub campaign_ids: Vec<Uuid>,
    pub parameters: HashMap<String, String>,
    pub results: Vec<BulkItemResult>,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub executed_by: Uuid,
    pub executed_at: DateTime<Utc>,
}

/// Engine for bulk campaign operations with guardrails and audit logging.
pub struct BulkOperationEngine {
    operations: DashMap<Uuid, BulkOperation>,
    max_batch_size: usize,
}

impl BulkOperationEngine {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            operations: DashMap::new(),
            max_batch_size,
        }
    }

    /// Execute a bulk pause operation. Returns per-campaign results.
    pub fn bulk_pause(
        &self,
        campaign_ids: Vec<Uuid>,
        user_id: Uuid,
        live_campaigns: &[Uuid],
    ) -> Result<BulkOperation, String> {
        if campaign_ids.len() > self.max_batch_size {
            return Err(format!(
                "Batch size {} exceeds maximum {}",
                campaign_ids.len(),
                self.max_batch_size
            ));
        }

        let results: Vec<BulkItemResult> = campaign_ids
            .iter()
            .map(|id| {
                if live_campaigns.contains(id) {
                    BulkItemResult {
                        campaign_id: *id,
                        success: true,
                        error: None,
                    }
                } else {
                    BulkItemResult {
                        campaign_id: *id,
                        success: false,
                        error: Some("Campaign is not live".to_string()),
                    }
                }
            })
            .collect();

        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        let op = BulkOperation {
            id: Uuid::new_v4(),
            operation_type: BulkOperationType::Pause,
            campaign_ids,
            parameters: HashMap::new(),
            results,
            total: succeeded + failed,
            succeeded,
            failed,
            executed_by: user_id,
            executed_at: Utc::now(),
        };

        self.operations.insert(op.id, op.clone());
        Ok(op)
    }

    /// Execute a bulk UTM update.
    pub fn bulk_utm_update(
        &self,
        campaign_ids: Vec<Uuid>,
        utm_params: HashMap<String, String>,
        user_id: Uuid,
    ) -> Result<BulkOperation, String> {
        if campaign_ids.len() > self.max_batch_size {
            return Err(format!(
                "Batch size {} exceeds maximum {}",
                campaign_ids.len(),
                self.max_batch_size
            ));
        }

        let results: Vec<BulkItemResult> = campaign_ids
            .iter()
            .map(|id| BulkItemResult {
                campaign_id: *id,
                success: true,
                error: None,
            })
            .collect();

        let total = campaign_ids.len();
        let op = BulkOperation {
            id: Uuid::new_v4(),
            operation_type: BulkOperationType::UtmUpdate,
            campaign_ids,
            parameters: utm_params,
            results,
            total,
            succeeded: total,
            failed: 0,
            executed_by: user_id,
            executed_at: Utc::now(),
        };

        self.operations.insert(op.id, op.clone());
        Ok(op)
    }

    /// Get audit trail for a bulk operation.
    pub fn get_operation(&self, id: &Uuid) -> Option<BulkOperation> {
        self.operations.get(id).map(|e| e.clone())
    }

    /// List all bulk operations by a user.
    pub fn list_by_user(&self, user_id: &Uuid) -> Vec<BulkOperation> {
        self.operations
            .iter()
            .filter(|e| e.value().executed_by == *user_id)
            .map(|e| e.value().clone())
            .collect()
    }
}

impl Default for BulkOperationEngine {
    fn default() -> Self {
        Self::new(500)
    }
}

// ─── Explainability Surfaces (FR-UX-005) ──────────────────────────────

/// Type of explainability query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplainQuery {
    /// Why did user X receive this message?
    MessageDelivery { user_id: Uuid, message_id: Uuid },
    /// Why did this DCO variant win?
    DcoVariantWin { campaign_id: Uuid, variant_id: Uuid },
    /// Why was an asset blocked by brand rules?
    AssetBlocked { asset_id: Uuid, guideline_id: Uuid },
}

/// A single factor contributing to an explainability answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainFactor {
    pub factor_type: String,
    pub description: String,
    pub weight: f64,
    pub details: HashMap<String, String>,
}

/// The full explainability answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub query: String,
    pub summary: String,
    pub factors: Vec<ExplainFactor>,
    pub generated_at: DateTime<Utc>,
}

/// Engine for "Why?" queries to explain personalization, variant selection, and blocking decisions.
pub struct ExplainabilityEngine {
    results: DashMap<Uuid, ExplainResult>,
}

impl ExplainabilityEngine {
    pub fn new() -> Self {
        Self {
            results: DashMap::new(),
        }
    }

    /// Explain why a user received a message.
    pub fn explain_message_delivery(
        &self,
        user_id: Uuid,
        message_id: Uuid,
        segment_names: &[String],
        channel: &str,
        personalization_score: f64,
    ) -> ExplainResult {
        let factors = vec![
            ExplainFactor {
                factor_type: "segment_membership".to_string(),
                description: format!("User matched {} segment(s)", segment_names.len()),
                weight: 0.4,
                details: vec![("segments".to_string(), segment_names.join(", "))]
                    .into_iter()
                    .collect(),
            },
            ExplainFactor {
                factor_type: "channel_preference".to_string(),
                description: format!("User opted in to {} channel", channel),
                weight: 0.3,
                details: vec![("channel".to_string(), channel.to_string())]
                    .into_iter()
                    .collect(),
            },
            ExplainFactor {
                factor_type: "personalization_score".to_string(),
                description: format!(
                    "Personalization model scored {:.2} (above threshold)",
                    personalization_score
                ),
                weight: 0.3,
                details: vec![("score".to_string(), format!("{:.4}", personalization_score))]
                    .into_iter()
                    .collect(),
            },
        ];

        let result = ExplainResult {
            query: format!(
                "Why did user {} receive message {}?",
                user_id, message_id
            ),
            summary: format!(
                "User was in {} qualifying segment(s) with {} channel opt-in and a personalization score of {:.2}",
                segment_names.len(),
                channel,
                personalization_score
            ),
            factors,
            generated_at: Utc::now(),
        };

        let id = Uuid::new_v4();
        self.results.insert(id, result.clone());
        result
    }

    /// Explain why a DCO variant was selected.
    pub fn explain_variant_win(
        &self,
        variant_id: Uuid,
        variant_name: &str,
        thompson_score: f64,
        click_rate: f64,
        impressions: u64,
    ) -> ExplainResult {
        let factors = vec![
            ExplainFactor {
                factor_type: "thompson_sampling".to_string(),
                description: format!(
                    "Thompson Sampling score: {:.4} (highest among variants)",
                    thompson_score
                ),
                weight: 0.5,
                details: vec![("score".to_string(), format!("{:.4}", thompson_score))]
                    .into_iter()
                    .collect(),
            },
            ExplainFactor {
                factor_type: "historical_ctr".to_string(),
                description: format!("Historical CTR: {:.2}%", click_rate * 100.0),
                weight: 0.3,
                details: vec![
                    ("ctr".to_string(), format!("{:.4}", click_rate)),
                    ("impressions".to_string(), impressions.to_string()),
                ]
                .into_iter()
                .collect(),
            },
            ExplainFactor {
                factor_type: "exploration_bonus".to_string(),
                description: "Exploration bonus for under-served variant".to_string(),
                weight: 0.2,
                details: HashMap::new(),
            },
        ];

        let result = ExplainResult {
            query: format!("Why did variant {} ({}) win?", variant_id, variant_name),
            summary: format!(
                "Variant '{}' won with Thompson score {:.4} and {:.2}% CTR over {} impressions",
                variant_name,
                thompson_score,
                click_rate * 100.0,
                impressions
            ),
            factors,
            generated_at: Utc::now(),
        };

        let id = Uuid::new_v4();
        self.results.insert(id, result.clone());
        result
    }

    /// Explain why an asset was blocked by brand rules.
    pub fn explain_asset_blocked(
        &self,
        asset_id: Uuid,
        violations: Vec<(String, String, String)>,
    ) -> ExplainResult {
        let factors: Vec<ExplainFactor> = violations
            .iter()
            .enumerate()
            .map(|(i, (rule_type, severity, message))| ExplainFactor {
                factor_type: rule_type.clone(),
                description: message.clone(),
                weight: 1.0 / (i as f64 + 1.0),
                details: vec![("severity".to_string(), severity.clone())]
                    .into_iter()
                    .collect(),
            })
            .collect();

        let result = ExplainResult {
            query: format!("Why was asset {} blocked?", asset_id),
            summary: format!(
                "Asset violated {} brand rule(s): {}",
                factors.len(),
                violations
                    .iter()
                    .map(|(t, _, _)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            factors,
            generated_at: Utc::now(),
        };

        let id = Uuid::new_v4();
        self.results.insert(id, result.clone());
        result
    }
}

impl Default for ExplainabilityEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_flow_wizard() {
        let flow = UnifiedCreateFlow::new();
        let user = Uuid::new_v4();

        let state = flow.start(CampaignType::Lifecycle, user);
        assert_eq!(state.current_step, CreateFlowStep::Objective);

        let state = flow
            .set_objective(
                &state.id,
                CampaignObjective::Conversion,
                "Summer Sale".to_string(),
            )
            .unwrap();
        assert_eq!(state.current_step, CreateFlowStep::Audience);
        assert_eq!(state.objective, Some(CampaignObjective::Conversion));

        let audience = AudienceConfig {
            segment_ids: vec![Uuid::new_v4()],
            exclusion_segment_ids: vec![],
            estimated_reach: 50000,
            filters: HashMap::new(),
        };
        let state = flow.set_audience(&state.id, audience).unwrap();
        assert_eq!(state.current_step, CreateFlowStep::Content);
        assert_eq!(state.completed_steps.len(), 2);
    }

    #[test]
    fn test_create_flow_step_navigation() {
        assert_eq!(
            CreateFlowStep::Objective.next(),
            Some(CreateFlowStep::Audience)
        );
        assert_eq!(CreateFlowStep::Launch.next(), None);
        assert_eq!(CreateFlowStep::Objective.prev(), None);
        assert_eq!(
            CreateFlowStep::Audience.prev(),
            Some(CreateFlowStep::Objective)
        );
    }

    #[test]
    fn test_operator_calendar_filters() {
        let calendar = OperatorCalendar::new();
        let now = Utc::now();
        let owner = Uuid::new_v4();

        calendar.add_entry(OperatorCalendarEntry {
            id: Uuid::new_v4(),
            campaign_id: Uuid::new_v4(),
            campaign_name: "Email Promo".to_string(),
            campaign_type: CampaignType::Lifecycle,
            objective: CampaignObjective::Conversion,
            channels: vec![LifecycleChannel::Email],
            status: "live".to_string(),
            owner_id: owner,
            start_date: now + chrono::Duration::hours(1),
            end_date: None,
            budget: Some(5000.0),
            color: "#FF5733".to_string(),
            tags: vec!["promo".to_string()],
        });

        calendar.add_entry(OperatorCalendarEntry {
            id: Uuid::new_v4(),
            campaign_id: Uuid::new_v4(),
            campaign_name: "SMS Flash".to_string(),
            campaign_type: CampaignType::Lifecycle,
            objective: CampaignObjective::Engagement,
            channels: vec![LifecycleChannel::Sms],
            status: "scheduled".to_string(),
            owner_id: Uuid::new_v4(),
            start_date: now + chrono::Duration::hours(2),
            end_date: None,
            budget: Some(1000.0),
            color: "#33FF57".to_string(),
            tags: vec![],
        });

        // No filter — should return both
        let all = calendar.query(
            now,
            now + chrono::Duration::days(1),
            &CalendarFilter::default(),
        );
        assert_eq!(all.len(), 2);

        // Filter by channel
        let email_only = calendar.query(
            now,
            now + chrono::Duration::days(1),
            &CalendarFilter {
                channels: vec![LifecycleChannel::Email],
                ..Default::default()
            },
        );
        assert_eq!(email_only.len(), 1);
        assert_eq!(email_only[0].campaign_name, "Email Promo");

        // Filter by owner
        let owner_filter = calendar.query(
            now,
            now + chrono::Duration::days(1),
            &CalendarFilter {
                owner_ids: vec![owner],
                ..Default::default()
            },
        );
        assert_eq!(owner_filter.len(), 1);

        // Filter by budget
        let high_budget = calendar.query(
            now,
            now + chrono::Duration::days(1),
            &CalendarFilter {
                min_budget: Some(3000.0),
                ..Default::default()
            },
        );
        assert_eq!(high_budget.len(), 1);
        assert_eq!(high_budget[0].campaign_name, "Email Promo");
    }

    #[test]
    fn test_sms_preview() {
        let preview = CrossChannelPreview::preview_sms(
            "Hello! Check out https://example.com/sale for deals.",
            "https://short.io",
        );
        assert!(!preview.is_unicode);
        assert_eq!(preview.shortened_links.len(), 1);
        assert!(preview.segment_count >= 1);
    }

    #[test]
    fn test_ad_preview_warnings() {
        let preview = CrossChannelPreview::preview_ad(
            "Leaderboard 728x90",
            600,
            90,
            728,
            90,
            10,
            false,
            200_000,
            150_000,
        );
        assert!(!preview.warnings.is_empty());
        assert!(!preview.cta_visible);
        assert!(!preview.file_size_ok);
    }

    #[test]
    fn test_bulk_pause_with_guardrails() {
        let engine = BulkOperationEngine::new(100);
        let user = Uuid::new_v4();
        let live = vec![Uuid::new_v4(), Uuid::new_v4()];
        let not_live = Uuid::new_v4();

        let result = engine
            .bulk_pause(vec![live[0], live[1], not_live], user, &live)
            .unwrap();

        assert_eq!(result.succeeded, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.total, 3);

        let audit = engine.get_operation(&result.id).unwrap();
        assert_eq!(audit.operation_type, BulkOperationType::Pause);
    }

    #[test]
    fn test_explainability_message() {
        let engine = ExplainabilityEngine::new();
        let result = engine.explain_message_delivery(
            Uuid::new_v4(),
            Uuid::new_v4(),
            &["High Value".to_string(), "Email Active".to_string()],
            "email",
            0.87,
        );

        assert!(result.summary.contains("2 qualifying segment"));
        assert_eq!(result.factors.len(), 3);
    }

    #[test]
    fn test_explainability_variant_win() {
        let engine = ExplainabilityEngine::new();
        let result =
            engine.explain_variant_win(Uuid::new_v4(), "Hero Blue CTA", 0.923, 0.042, 15000);

        assert!(result.summary.contains("Hero Blue CTA"));
        assert!(result.summary.contains("0.9230"));
    }

    #[test]
    fn test_explainability_asset_blocked() {
        let engine = ExplainabilityEngine::new();
        let result = engine.explain_asset_blocked(
            Uuid::new_v4(),
            vec![
                (
                    "ColorCompliance".to_string(),
                    "Block".to_string(),
                    "Uses #FF00FF".to_string(),
                ),
                (
                    "MinImageResolution".to_string(),
                    "Block".to_string(),
                    "400x300 too small".to_string(),
                ),
            ],
        );

        assert_eq!(result.factors.len(), 2);
        assert!(result.summary.contains("2 brand rule"));
    }
}
