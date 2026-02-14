//! Template & block ecosystem: template library across lifecycle/campaign/DCO,
//! template lifecycle management, and reusable blocks/snippets with impact analysis.
//!
//! Addresses FR-TPL-001 through FR-TPL-003.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Template Library (FR-TPL-001) ────────────────────────────────────

/// Category of template.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Lifecycle message templates (email, push, SMS, in-app, WhatsApp).
    LifecycleMessage,
    /// Campaign templates (objective-driven presets).
    CampaignPreset,
    /// DCO templates (creative assembly presets).
    DcoCreative,
}

/// Channel for lifecycle message templates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateChannel {
    Email,
    Push,
    Sms,
    InApp,
    WhatsApp,
    ContentCard,
}

/// Template lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateStatus {
    Draft,
    InReview,
    Approved,
    Active,
    Deprecated,
    Archived,
}

/// A template entry in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub channel: Option<TemplateChannel>,
    pub tags: Vec<String>,
    pub version: u32,
    pub status: TemplateStatus,
    pub content: HashMap<String, String>,
    pub block_ids: Vec<Uuid>,
    pub created_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deprecated_at: Option<DateTime<Utc>>,
}

/// The template library.
pub struct TemplateLibrary {
    templates: DashMap<Uuid, LibraryTemplate>,
    /// track version history per template
    versions: DashMap<Uuid, Vec<(u32, DateTime<Utc>, String)>>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        Self {
            templates: DashMap::new(),
            versions: DashMap::new(),
        }
    }

    /// Create a new template.
    pub fn create(
        &self,
        name: String,
        description: String,
        category: TemplateCategory,
        channel: Option<TemplateChannel>,
        content: HashMap<String, String>,
        user_id: Uuid,
    ) -> LibraryTemplate {
        let now = Utc::now();
        let template = LibraryTemplate {
            id: Uuid::new_v4(),
            name,
            description,
            category,
            channel,
            tags: Vec::new(),
            version: 1,
            status: TemplateStatus::Draft,
            content,
            block_ids: Vec::new(),
            created_by: user_id,
            approved_by: None,
            created_at: now,
            updated_at: now,
            deprecated_at: None,
        };
        self.templates.insert(template.id, template.clone());
        self.versions
            .insert(template.id, vec![(1, now, "Initial version".to_string())]);
        template
    }

    /// Submit template for review.
    pub fn submit_for_review(&self, id: &Uuid) -> Result<LibraryTemplate, String> {
        let mut entry = self.templates.get_mut(id).ok_or("Template not found")?;
        if entry.status != TemplateStatus::Draft {
            return Err("Only draft templates can be submitted for review".to_string());
        }
        entry.status = TemplateStatus::InReview;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Approve a template.
    pub fn approve(&self, id: &Uuid, approver_id: Uuid) -> Result<LibraryTemplate, String> {
        let mut entry = self.templates.get_mut(id).ok_or("Template not found")?;
        if entry.status != TemplateStatus::InReview {
            return Err("Only templates in review can be approved".to_string());
        }
        entry.status = TemplateStatus::Approved;
        entry.approved_by = Some(approver_id);
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Activate an approved template.
    pub fn activate(&self, id: &Uuid) -> Result<LibraryTemplate, String> {
        let mut entry = self.templates.get_mut(id).ok_or("Template not found")?;
        if entry.status != TemplateStatus::Approved {
            return Err("Only approved templates can be activated".to_string());
        }
        entry.status = TemplateStatus::Active;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Deprecate a template.
    pub fn deprecate(&self, id: &Uuid) -> Result<LibraryTemplate, String> {
        let mut entry = self.templates.get_mut(id).ok_or("Template not found")?;
        entry.status = TemplateStatus::Deprecated;
        entry.deprecated_at = Some(Utc::now());
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Archive a template.
    pub fn archive(&self, id: &Uuid) -> Result<LibraryTemplate, String> {
        let mut entry = self.templates.get_mut(id).ok_or("Template not found")?;
        entry.status = TemplateStatus::Archived;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Update template content and bump version.
    pub fn update_version(
        &self,
        id: &Uuid,
        content: HashMap<String, String>,
        changelog: String,
    ) -> Result<LibraryTemplate, String> {
        let mut entry = self.templates.get_mut(id).ok_or("Template not found")?;
        entry.version += 1;
        entry.content = content;
        entry.status = TemplateStatus::Draft;
        entry.updated_at = Utc::now();

        let new_version = entry.version;
        drop(entry);

        self.versions
            .entry(*id)
            .or_default()
            .push((new_version, Utc::now(), changelog));

        self.templates
            .get(id)
            .map(|e| e.clone())
            .ok_or("Template not found".to_string())
    }

    /// Search templates by category, channel, tags, or status.
    pub fn search(
        &self,
        category: Option<&TemplateCategory>,
        channel: Option<&TemplateChannel>,
        status: Option<&TemplateStatus>,
        query: Option<&str>,
    ) -> Vec<LibraryTemplate> {
        self.templates
            .iter()
            .filter(|e| {
                let t = e.value();
                let cat_ok = category.is_none_or(|c| t.category == *c);
                let ch_ok = channel.is_none_or(|c| t.channel.as_ref() == Some(c));
                let status_ok = status.is_none_or(|s| t.status == *s);
                let query_ok = query.is_none_or(|q| {
                    let ql = q.to_lowercase();
                    t.name.to_lowercase().contains(&ql)
                        || t.description.to_lowercase().contains(&ql)
                        || t.tags.iter().any(|tag| tag.to_lowercase().contains(&ql))
                });
                cat_ok && ch_ok && status_ok && query_ok
            })
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn get(&self, id: &Uuid) -> Option<LibraryTemplate> {
        self.templates.get(id).map(|e| e.clone())
    }

    /// Get version history.
    pub fn version_history(&self, id: &Uuid) -> Vec<(u32, DateTime<Utc>, String)> {
        self.versions.get(id).map(|v| v.clone()).unwrap_or_default()
    }
}

impl Default for TemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Reusable Blocks/Snippets (FR-TPL-003) ───────────────────────────

/// Type of reusable block.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReusableBlockType {
    Header,
    Footer,
    LegalDisclaimer,
    OfferBlock,
    ProductGrid,
    SocialLinks,
    BrandSignoff,
    UnsubscribeBlock,
    CustomHtml,
}

/// A reusable block/snippet that can be shared across templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReusableBlock {
    pub id: Uuid,
    pub name: String,
    pub block_type: ReusableBlockType,
    pub content: String,
    pub variables: Vec<String>,
    pub version: u32,
    pub status: TemplateStatus,
    pub brand_approved: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Usage tracking for a reusable block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockUsage {
    pub block_id: Uuid,
    pub used_in_template: Uuid,
    pub template_name: String,
    pub pinned_version: Option<u32>,
}

/// Block library with usage tracking for impact analysis.
pub struct BlockLibrary {
    blocks: DashMap<Uuid, ReusableBlock>,
    /// block_id -> list of templates using this block
    usages: DashMap<Uuid, Vec<BlockUsage>>,
}

impl BlockLibrary {
    pub fn new() -> Self {
        Self {
            blocks: DashMap::new(),
            usages: DashMap::new(),
        }
    }

    /// Create a reusable block.
    pub fn create_block(
        &self,
        name: String,
        block_type: ReusableBlockType,
        content: String,
        variables: Vec<String>,
        user_id: Uuid,
    ) -> ReusableBlock {
        let now = Utc::now();
        let block = ReusableBlock {
            id: Uuid::new_v4(),
            name,
            block_type,
            content,
            variables,
            version: 1,
            status: TemplateStatus::Draft,
            brand_approved: false,
            created_by: user_id,
            created_at: now,
            updated_at: now,
        };
        self.blocks.insert(block.id, block.clone());
        block
    }

    /// Approve a block.
    pub fn approve_block(&self, id: &Uuid) -> Result<ReusableBlock, String> {
        let mut entry = self.blocks.get_mut(id).ok_or("Block not found")?;
        entry.status = TemplateStatus::Approved;
        entry.brand_approved = true;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Update a block's content and bump version.
    pub fn update_block(&self, id: &Uuid, content: String) -> Result<ReusableBlock, String> {
        let mut entry = self.blocks.get_mut(id).ok_or("Block not found")?;
        entry.version += 1;
        entry.content = content;
        entry.status = TemplateStatus::Draft;
        entry.brand_approved = false;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }

    /// Register that a template uses this block.
    pub fn register_usage(
        &self,
        block_id: Uuid,
        template_id: Uuid,
        template_name: String,
        pinned_version: Option<u32>,
    ) {
        self.usages.entry(block_id).or_default().push(BlockUsage {
            block_id,
            used_in_template: template_id,
            template_name,
            pinned_version,
        });
    }

    /// Impact analysis: which templates/campaigns use this block?
    pub fn impact_analysis(&self, block_id: &Uuid) -> Vec<BlockUsage> {
        self.usages
            .get(block_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Search blocks by type and name.
    pub fn search(
        &self,
        block_type: Option<&ReusableBlockType>,
        query: Option<&str>,
        approved_only: bool,
    ) -> Vec<ReusableBlock> {
        self.blocks
            .iter()
            .filter(|e| {
                let b = e.value();
                let type_ok = block_type.is_none_or(|bt| b.block_type == *bt);
                let query_ok =
                    query.is_none_or(|q| b.name.to_lowercase().contains(&q.to_lowercase()));
                let approval_ok = !approved_only || b.brand_approved;
                type_ok && query_ok && approval_ok
            })
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn get(&self, id: &Uuid) -> Option<ReusableBlock> {
        self.blocks.get(id).map(|e| e.clone())
    }

    /// List all blocks.
    pub fn list_all(&self) -> Vec<ReusableBlock> {
        self.blocks.iter().map(|e| e.value().clone()).collect()
    }
}

impl Default for BlockLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_lifecycle() {
        let library = TemplateLibrary::new();
        let user = Uuid::new_v4();
        let approver = Uuid::new_v4();

        let tmpl = library.create(
            "Welcome Email".to_string(),
            "Onboarding welcome email".to_string(),
            TemplateCategory::LifecycleMessage,
            Some(TemplateChannel::Email),
            vec![("subject".to_string(), "Welcome!".to_string())]
                .into_iter()
                .collect(),
            user,
        );

        assert_eq!(tmpl.status, TemplateStatus::Draft);
        assert_eq!(tmpl.version, 1);

        // Draft -> InReview -> Approved -> Active
        library.submit_for_review(&tmpl.id).unwrap();
        library.approve(&tmpl.id, approver).unwrap();
        library.activate(&tmpl.id).unwrap();

        let tmpl = library.get(&tmpl.id).unwrap();
        assert_eq!(tmpl.status, TemplateStatus::Active);

        // Deprecate -> Archive
        library.deprecate(&tmpl.id).unwrap();
        let tmpl = library.get(&tmpl.id).unwrap();
        assert_eq!(tmpl.status, TemplateStatus::Deprecated);
        assert!(tmpl.deprecated_at.is_some());

        library.archive(&tmpl.id).unwrap();
        let tmpl = library.get(&tmpl.id).unwrap();
        assert_eq!(tmpl.status, TemplateStatus::Archived);
    }

    #[test]
    fn test_template_versioning() {
        let library = TemplateLibrary::new();
        let user = Uuid::new_v4();

        let tmpl = library.create(
            "Promo Template".to_string(),
            "".to_string(),
            TemplateCategory::CampaignPreset,
            None,
            HashMap::new(),
            user,
        );

        library
            .update_version(
                &tmpl.id,
                vec![("body".to_string(), "Updated body v2".to_string())]
                    .into_iter()
                    .collect(),
                "Updated promotional copy".to_string(),
            )
            .unwrap();

        let tmpl = library.get(&tmpl.id).unwrap();
        assert_eq!(tmpl.version, 2);
        assert_eq!(tmpl.status, TemplateStatus::Draft); // back to draft

        let history = library.version_history(&tmpl.id);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_template_search() {
        let library = TemplateLibrary::new();
        let user = Uuid::new_v4();

        library.create(
            "Welcome Email".to_string(),
            "Onboarding welcome".to_string(),
            TemplateCategory::LifecycleMessage,
            Some(TemplateChannel::Email),
            HashMap::new(),
            user,
        );

        library.create(
            "Push Reminder".to_string(),
            "Cart abandon push".to_string(),
            TemplateCategory::LifecycleMessage,
            Some(TemplateChannel::Push),
            HashMap::new(),
            user,
        );

        library.create(
            "Summer Campaign".to_string(),
            "Preset for seasonal promos".to_string(),
            TemplateCategory::CampaignPreset,
            None,
            HashMap::new(),
            user,
        );

        // Search by category
        let lifecycle = library.search(Some(&TemplateCategory::LifecycleMessage), None, None, None);
        assert_eq!(lifecycle.len(), 2);

        // Search by channel
        let push = library.search(None, Some(&TemplateChannel::Push), None, None);
        assert_eq!(push.len(), 1);
        assert_eq!(push[0].name, "Push Reminder");

        // Search by query
        let welcome = library.search(None, None, None, Some("welcome"));
        assert_eq!(welcome.len(), 1);
    }

    #[test]
    fn test_reusable_blocks_and_impact() {
        let library = BlockLibrary::new();
        let user = Uuid::new_v4();

        let header = library.create_block(
            "Brand Header".to_string(),
            ReusableBlockType::Header,
            "<div class=\"header\">{{brand_name}}</div>".to_string(),
            vec!["brand_name".to_string()],
            user,
        );

        let footer = library.create_block(
            "Legal Footer".to_string(),
            ReusableBlockType::LegalDisclaimer,
            "<footer>Copyright {{year}} {{company}}</footer>".to_string(),
            vec!["year".to_string(), "company".to_string()],
            user,
        );

        // Approve header
        library.approve_block(&header.id).unwrap();
        let header = library.get(&header.id).unwrap();
        assert!(header.brand_approved);

        // Register usage
        let tmpl1_id = Uuid::new_v4();
        let tmpl2_id = Uuid::new_v4();
        library.register_usage(header.id, tmpl1_id, "Welcome Email".to_string(), None);
        library.register_usage(header.id, tmpl2_id, "Promo Email".to_string(), Some(1));
        library.register_usage(footer.id, tmpl1_id, "Welcome Email".to_string(), None);

        // Impact analysis: header is used in 2 templates
        let impact = library.impact_analysis(&header.id);
        assert_eq!(impact.len(), 2);

        // Footer used in 1
        let impact = library.impact_analysis(&footer.id);
        assert_eq!(impact.len(), 1);

        // Update block (bumps version, clears approval)
        let updated = library
            .update_block(
                &header.id,
                "<div class=\"header-v2\">{{brand_name}}</div>".to_string(),
            )
            .unwrap();
        assert_eq!(updated.version, 2);
        assert!(!updated.brand_approved);
    }

    #[test]
    fn test_block_search() {
        let library = BlockLibrary::new();
        let user = Uuid::new_v4();

        library.create_block(
            "Main Header".to_string(),
            ReusableBlockType::Header,
            "<header></header>".to_string(),
            vec![],
            user,
        );
        let footer = library.create_block(
            "Main Footer".to_string(),
            ReusableBlockType::Footer,
            "<footer></footer>".to_string(),
            vec![],
            user,
        );
        library.approve_block(&footer.id).unwrap();

        // Search by type
        let headers = library.search(Some(&ReusableBlockType::Header), None, false);
        assert_eq!(headers.len(), 1);

        // Search approved only
        let approved = library.search(None, None, true);
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].name, "Main Footer");
    }
}
