//! Content Studio for lifecycle messaging: email template builder,
//! HTML editor + hybrid mode, message variables, compliance automation,
//! and localization/variants.
//!
//! Addresses FR-CNT-001 through FR-CNT-005.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Email Template Builder (FR-CNT-001) ──────────────────────────────

/// Block type for drag-and-drop email builder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Hero,
    Text,
    Button,
    ProductGrid,
    Divider,
    Spacer,
    Social,
    LegalFooter,
    Image,
    Columns,
    Countdown,
    Video,
    Html,
}

/// A single block in an email template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBlock {
    pub id: Uuid,
    pub block_type: BlockType,
    pub content: HashMap<String, String>,
    pub styles: HashMap<String, String>,
    pub mobile_styles: Option<HashMap<String, String>>,
    pub sort_order: u32,
    /// If this block came from the snippets library.
    pub snippet_id: Option<Uuid>,
}

/// Responsive layout mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutMode {
    Desktop,
    Mobile,
    Hybrid,
}

/// A complete email template built with the drag-and-drop builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub preheader: Option<String>,
    pub blocks: Vec<EmailBlock>,
    pub layout_mode: LayoutMode,
    pub global_styles: HashMap<String, String>,
    pub version: u32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The email template builder engine.
pub struct EmailTemplateBuilder {
    templates: DashMap<Uuid, EmailTemplate>,
}

impl EmailTemplateBuilder {
    pub fn new() -> Self {
        Self {
            templates: DashMap::new(),
        }
    }

    pub fn create_template(&self, name: String, subject: String, user_id: Uuid) -> EmailTemplate {
        let now = Utc::now();
        let template = EmailTemplate {
            id: Uuid::new_v4(),
            name,
            subject,
            preheader: None,
            blocks: Vec::new(),
            layout_mode: LayoutMode::Hybrid,
            global_styles: HashMap::new(),
            version: 1,
            created_by: user_id,
            created_at: now,
            updated_at: now,
        };
        self.templates.insert(template.id, template.clone());
        template
    }

    pub fn add_block(&self, template_id: &Uuid, block: EmailBlock) -> Result<(), String> {
        let mut entry = self
            .templates
            .get_mut(template_id)
            .ok_or("Template not found")?;
        entry.blocks.push(block);
        entry.blocks.sort_by_key(|b| b.sort_order);
        entry.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_block(&self, template_id: &Uuid, block_id: &Uuid) -> Result<(), String> {
        let mut entry = self
            .templates
            .get_mut(template_id)
            .ok_or("Template not found")?;
        entry.blocks.retain(|b| b.id != *block_id);
        entry.updated_at = Utc::now();
        Ok(())
    }

    pub fn reorder_blocks(&self, template_id: &Uuid, block_ids: &[Uuid]) -> Result<(), String> {
        let mut entry = self
            .templates
            .get_mut(template_id)
            .ok_or("Template not found")?;
        for (i, id) in block_ids.iter().enumerate() {
            if let Some(block) = entry.blocks.iter_mut().find(|b| b.id == *id) {
                block.sort_order = i as u32;
            }
        }
        entry.blocks.sort_by_key(|b| b.sort_order);
        entry.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_template(&self, id: &Uuid) -> Option<EmailTemplate> {
        self.templates.get(id).map(|t| t.clone())
    }

    /// Render the template to HTML (simplified).
    pub fn render_html(&self, template_id: &Uuid) -> Result<String, String> {
        let entry = self
            .templates
            .get(template_id)
            .ok_or("Template not found")?;

        let mut html = String::from("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"></head><body>\n");

        for block in &entry.blocks {
            match block.block_type {
                BlockType::Hero => {
                    let src = block
                        .content
                        .get("image_url")
                        .map(|s| s.as_str())
                        .unwrap_or("#");
                    let alt = block
                        .content
                        .get("alt_text")
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    html.push_str(&format!("<div class=\"hero\"><img src=\"{}\" alt=\"{}\" style=\"width:100%;\"/></div>\n", src, alt));
                }
                BlockType::Text => {
                    let text = block.content.get("text").map(|s| s.as_str()).unwrap_or("");
                    html.push_str(&format!(
                        "<div class=\"text-block\"><p>{}</p></div>\n",
                        text
                    ));
                }
                BlockType::Button => {
                    let label = block
                        .content
                        .get("label")
                        .map(|s| s.as_str())
                        .unwrap_or("Click");
                    let url = block.content.get("url").map(|s| s.as_str()).unwrap_or("#");
                    html.push_str(&format!(
                        "<div class=\"button-block\"><a href=\"{}\" class=\"btn\">{}</a></div>\n",
                        url, label
                    ));
                }
                BlockType::Divider => {
                    html.push_str("<hr class=\"divider\"/>\n");
                }
                BlockType::Spacer => {
                    let height = block
                        .content
                        .get("height")
                        .map(|s| s.as_str())
                        .unwrap_or("20");
                    html.push_str(&format!("<div style=\"height:{}px;\"></div>\n", height));
                }
                BlockType::LegalFooter => {
                    let text = block.content.get("text").map(|s| s.as_str()).unwrap_or("");
                    html.push_str(&format!("<footer class=\"legal\">{}</footer>\n", text));
                }
                _ => {
                    let content = block.content.get("html").map(|s| s.as_str()).unwrap_or("");
                    html.push_str(&format!("<div class=\"block\">{}</div>\n", content));
                }
            }
        }

        html.push_str("</body></html>");
        Ok(html)
    }
}

impl Default for EmailTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ─── HTML Editor + Hybrid Mode (FR-CNT-002) ───────────────────────────

/// HTML lint issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlLintIssue {
    pub severity: HtmlLintSeverity,
    pub message: String,
    pub line: Option<u32>,
}

/// Severity levels for HTML lint issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HtmlLintSeverity {
    Error,
    Warning,
    Info,
}

/// HTML editor with sanitization and linting.
pub struct HtmlEditor;

impl HtmlEditor {
    /// Sanitize HTML by removing dangerous tags and attributes.
    pub fn sanitize(html: &str) -> String {
        let dangerous_tags = [
            "<script",
            "</script>",
            "<iframe",
            "</iframe>",
            "<object",
            "</object>",
            "<embed",
            "</embed>",
            "<applet",
            "</applet>",
            "javascript:",
            "onerror=",
            "onclick=",
            "onload=",
            "onmouseover=",
        ];

        let mut sanitized = html.to_string();
        for tag in &dangerous_tags {
            // Case-insensitive removal
            let lower = sanitized.to_lowercase();
            if let Some(pos) = lower.find(&tag.to_lowercase()) {
                let end = if tag.starts_with("on") || tag.starts_with("javascript") {
                    // Remove up to next quote or >
                    sanitized[pos..]
                        .find(['"', '\'', '>', ' '])
                        .map(|e| pos + e)
                        .unwrap_or(sanitized.len())
                } else if tag.starts_with("</") {
                    pos + tag.len()
                } else {
                    // Remove to closing >
                    sanitized[pos..]
                        .find('>')
                        .map(|e| pos + e + 1)
                        .unwrap_or(sanitized.len())
                };
                sanitized = format!("{}{}", &sanitized[..pos], &sanitized[end..]);
            }
        }

        sanitized
    }

    /// Lint HTML for common issues.
    pub fn lint(html: &str) -> Vec<HtmlLintIssue> {
        let mut issues = Vec::new();

        // Check for images without alt text
        let lower = html.to_lowercase();
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find("<img") {
            let abs_pos = search_from + pos;
            let tag_end = lower[abs_pos..].find('>').unwrap_or(lower.len() - abs_pos);
            let tag = &lower[abs_pos..abs_pos + tag_end];
            if !tag.contains("alt=") {
                issues.push(HtmlLintIssue {
                    severity: HtmlLintSeverity::Warning,
                    message: "Image tag missing alt text".to_string(),
                    line: None,
                });
            }
            search_from = abs_pos + tag_end + 1;
            if search_from >= lower.len() {
                break;
            }
        }

        // Check for broken link patterns
        if lower.contains("href=\"\"") || lower.contains("href=\"#\"") {
            issues.push(HtmlLintIssue {
                severity: HtmlLintSeverity::Warning,
                message: "Link with empty or placeholder href detected".to_string(),
                line: None,
            });
        }

        // Check for deprecated tags
        let deprecated = ["<font", "<center", "<marquee", "<blink"];
        for tag in &deprecated {
            if lower.contains(tag) {
                issues.push(HtmlLintIssue {
                    severity: HtmlLintSeverity::Warning,
                    message: format!("Deprecated HTML tag {} found", tag),
                    line: None,
                });
            }
        }

        // Check for missing doctype
        if !lower.starts_with("<!doctype") {
            issues.push(HtmlLintIssue {
                severity: HtmlLintSeverity::Info,
                message: "Missing DOCTYPE declaration".to_string(),
                line: None,
            });
        }

        issues
    }

    /// Render HTML template with test data variables.
    pub fn render_with_data(html: &str, data: &HashMap<String, String>) -> String {
        let mut rendered = html.to_string();
        for (key, value) in data {
            rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
            rendered = rendered.replace(&format!("{{{{ {} }}}}", key), value);
        }
        rendered
    }
}

// ─── Message Variables (FR-CNT-003) ───────────────────────────────────

/// Variable data type for validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    Text,
    Number,
    Date,
    Boolean,
    Url,
    Currency,
    List,
}

/// A message variable definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageVariable {
    pub name: String,
    pub display_name: String,
    pub var_type: VariableType,
    pub default_value: Option<String>,
    pub required: bool,
    pub description: String,
    pub category: String,
    pub example_value: String,
}

/// Variable browser/registry for autocomplete.
pub struct VariableBrowser {
    variables: DashMap<String, MessageVariable>,
}

impl VariableBrowser {
    pub fn new() -> Self {
        let browser = Self {
            variables: DashMap::new(),
        };
        browser.seed_standard_variables();
        browser
    }

    fn seed_standard_variables(&self) {
        let vars = vec![
            MessageVariable {
                name: "first_name".to_string(),
                display_name: "First Name".to_string(),
                var_type: VariableType::Text,
                default_value: Some("Valued Customer".to_string()),
                required: false,
                description: "Recipient's first name".to_string(),
                category: "profile".to_string(),
                example_value: "Jane".to_string(),
            },
            MessageVariable {
                name: "last_name".to_string(),
                display_name: "Last Name".to_string(),
                var_type: VariableType::Text,
                default_value: None,
                required: false,
                description: "Recipient's last name".to_string(),
                category: "profile".to_string(),
                example_value: "Smith".to_string(),
            },
            MessageVariable {
                name: "email".to_string(),
                display_name: "Email Address".to_string(),
                var_type: VariableType::Text,
                default_value: None,
                required: true,
                description: "Recipient's email address".to_string(),
                category: "profile".to_string(),
                example_value: "jane@example.com".to_string(),
            },
            MessageVariable {
                name: "loyalty_tier".to_string(),
                display_name: "Loyalty Tier".to_string(),
                var_type: VariableType::Text,
                default_value: Some("Bronze".to_string()),
                required: false,
                description: "Customer loyalty tier".to_string(),
                category: "loyalty".to_string(),
                example_value: "Gold".to_string(),
            },
            MessageVariable {
                name: "points_balance".to_string(),
                display_name: "Points Balance".to_string(),
                var_type: VariableType::Number,
                default_value: Some("0".to_string()),
                required: false,
                description: "Current loyalty points balance".to_string(),
                category: "loyalty".to_string(),
                example_value: "1250".to_string(),
            },
            MessageVariable {
                name: "offer_url".to_string(),
                display_name: "Offer URL".to_string(),
                var_type: VariableType::Url,
                default_value: None,
                required: false,
                description: "Personalized offer landing page URL".to_string(),
                category: "campaign".to_string(),
                example_value: "https://shop.example.com/offers/123".to_string(),
            },
            MessageVariable {
                name: "offer_amount".to_string(),
                display_name: "Offer Amount".to_string(),
                var_type: VariableType::Currency,
                default_value: None,
                required: false,
                description: "Offer discount amount".to_string(),
                category: "campaign".to_string(),
                example_value: "$25.00".to_string(),
            },
            MessageVariable {
                name: "expiry_date".to_string(),
                display_name: "Offer Expiry Date".to_string(),
                var_type: VariableType::Date,
                default_value: None,
                required: false,
                description: "Offer expiration date".to_string(),
                category: "campaign".to_string(),
                example_value: "2026-03-15".to_string(),
            },
            MessageVariable {
                name: "unsubscribe_url".to_string(),
                display_name: "Unsubscribe URL".to_string(),
                var_type: VariableType::Url,
                default_value: Some("https://example.com/unsubscribe".to_string()),
                required: true,
                description: "One-click unsubscribe URL (required for CAN-SPAM)".to_string(),
                category: "compliance".to_string(),
                example_value: "https://example.com/unsubscribe?id=abc".to_string(),
            },
        ];

        for var in vars {
            self.variables.insert(var.name.clone(), var);
        }
    }

    /// Autocomplete: search variables by prefix.
    pub fn autocomplete(&self, prefix: &str) -> Vec<MessageVariable> {
        let prefix_lower = prefix.to_lowercase();
        self.variables
            .iter()
            .filter(|e| {
                e.key().to_lowercase().starts_with(&prefix_lower)
                    || e.value()
                        .display_name
                        .to_lowercase()
                        .contains(&prefix_lower)
            })
            .map(|e| e.value().clone())
            .collect()
    }

    /// Get all variables grouped by category.
    pub fn by_category(&self) -> HashMap<String, Vec<MessageVariable>> {
        let mut categories: HashMap<String, Vec<MessageVariable>> = HashMap::new();
        for entry in self.variables.iter() {
            categories
                .entry(entry.value().category.clone())
                .or_default()
                .push(entry.value().clone());
        }
        categories
    }

    /// Validate that all required variables have values in the provided data.
    pub fn validate(&self, data: &HashMap<String, String>) -> Vec<String> {
        let mut errors = Vec::new();
        for entry in self.variables.iter() {
            let var = entry.value();
            if var.required && !data.contains_key(&var.name) && var.default_value.is_none() {
                errors.push(format!("Required variable '{}' is missing", var.name));
            }
        }
        errors
    }

    /// Build example data map for test rendering.
    pub fn example_data(&self) -> HashMap<String, String> {
        self.variables
            .iter()
            .map(|e| (e.key().clone(), e.value().example_value.clone()))
            .collect()
    }

    pub fn register_variable(&self, var: MessageVariable) {
        self.variables.insert(var.name.clone(), var);
    }
}

impl Default for VariableBrowser {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Compliance Automation (FR-CNT-004) ───────────────────────────────

/// Channel compliance issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    pub channel: String,
    pub check_name: String,
    pub severity: ComplianceSeverity,
    pub message: String,
    pub auto_fixable: bool,
}

/// Compliance severity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Blocking,
    Warning,
    Info,
}

/// Channel-specific compliance checker.
pub struct ComplianceChecker;

impl ComplianceChecker {
    /// Check email compliance (CAN-SPAM).
    pub fn check_email(
        html: &str,
        has_unsubscribe: bool,
        has_physical_address: bool,
    ) -> Vec<ComplianceIssue> {
        let mut issues = Vec::new();

        if !has_unsubscribe {
            issues.push(ComplianceIssue {
                channel: "email".to_string(),
                check_name: "CAN-SPAM Unsubscribe".to_string(),
                severity: ComplianceSeverity::Blocking,
                message: "Missing unsubscribe link (required by CAN-SPAM)".to_string(),
                auto_fixable: true,
            });
        }

        if !has_physical_address {
            issues.push(ComplianceIssue {
                channel: "email".to_string(),
                check_name: "CAN-SPAM Physical Address".to_string(),
                severity: ComplianceSeverity::Blocking,
                message: "Missing physical mailing address (required by CAN-SPAM)".to_string(),
                auto_fixable: true,
            });
        }

        let lower = html.to_lowercase();
        if lower.contains("unsubscribe") && !lower.contains("href") {
            issues.push(ComplianceIssue {
                channel: "email".to_string(),
                check_name: "Unsubscribe Link".to_string(),
                severity: ComplianceSeverity::Warning,
                message: "Unsubscribe text found but no clickable link detected".to_string(),
                auto_fixable: false,
            });
        }

        issues
    }

    /// Check SMS/WhatsApp compliance (STOP/HELP, quiet hours).
    pub fn check_sms(
        text: &str,
        is_whatsapp: bool,
        send_hour: u32,
        quiet_start: u32,
        quiet_end: u32,
    ) -> Vec<ComplianceIssue> {
        let mut issues = Vec::new();
        let lower = text.to_lowercase();
        let channel = if is_whatsapp { "whatsapp" } else { "sms" };

        // STOP/HELP language
        if !lower.contains("stop") && !lower.contains("opt out") && !lower.contains("unsubscribe") {
            issues.push(ComplianceIssue {
                channel: channel.to_string(),
                check_name: "STOP Language".to_string(),
                severity: ComplianceSeverity::Blocking,
                message: "Missing STOP/opt-out instruction (TCPA requirement)".to_string(),
                auto_fixable: true,
            });
        }

        if !lower.contains("help") && !lower.contains("support") {
            issues.push(ComplianceIssue {
                channel: channel.to_string(),
                check_name: "HELP Language".to_string(),
                severity: ComplianceSeverity::Warning,
                message: "Missing HELP instruction for customer support".to_string(),
                auto_fixable: true,
            });
        }

        // Quiet hours
        if (quiet_start < quiet_end && (send_hour >= quiet_start && send_hour < quiet_end))
            || (quiet_start > quiet_end && (send_hour >= quiet_start || send_hour < quiet_end))
        {
            issues.push(ComplianceIssue {
                channel: channel.to_string(),
                check_name: "Quiet Hours".to_string(),
                severity: ComplianceSeverity::Blocking,
                message: format!(
                    "Send time {}:00 falls within quiet hours ({:02}:00-{:02}:00)",
                    send_hour, quiet_start, quiet_end
                ),
                auto_fixable: false,
            });
        }

        issues
    }

    /// Check frequency cap violations.
    pub fn check_frequency_cap(
        messages_sent_in_window: u32,
        frequency_cap: u32,
        channel: &str,
    ) -> Vec<ComplianceIssue> {
        let mut issues = Vec::new();

        if messages_sent_in_window >= frequency_cap {
            issues.push(ComplianceIssue {
                channel: channel.to_string(),
                check_name: "Frequency Cap".to_string(),
                severity: ComplianceSeverity::Warning,
                message: format!(
                    "User has received {}/{} messages in the current window",
                    messages_sent_in_window, frequency_cap
                ),
                auto_fixable: false,
            });
        }

        issues
    }
}

// ─── Localization & Variants (FR-CNT-005) ─────────────────────────────

/// A locale-specific variant of a message/template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleVariant {
    pub id: Uuid,
    pub template_id: Uuid,
    pub locale: String,
    pub subject: String,
    pub body: String,
    pub variables_override: HashMap<String, String>,
    pub status: LocaleStatus,
    pub translator: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status of a locale variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocaleStatus {
    Draft,
    InTranslation,
    Translated,
    Reviewed,
    Approved,
}

/// Localization engine managing locale variants per template.
pub struct LocalizationEngine {
    variants: DashMap<Uuid, LocaleVariant>,
    fallback_locale: String,
}

impl LocalizationEngine {
    pub fn new(fallback_locale: &str) -> Self {
        Self {
            variants: DashMap::new(),
            fallback_locale: fallback_locale.to_string(),
        }
    }

    pub fn add_variant(&self, variant: LocaleVariant) {
        self.variants.insert(variant.id, variant);
    }

    /// Get the best variant for a template + locale, falling back to default locale.
    pub fn get_variant(&self, template_id: &Uuid, locale: &str) -> Option<LocaleVariant> {
        // Try exact match
        let exact = self.variants.iter().find(|e| {
            let v = e.value();
            v.template_id == *template_id && v.locale == locale
        });

        if let Some(found) = exact {
            return Some(found.value().clone());
        }

        // Try language-only match (e.g. "fr" from "fr-CA")
        let lang = locale.split('-').next().unwrap_or(locale);
        let language_match = self.variants.iter().find(|e| {
            let v = e.value();
            v.template_id == *template_id && v.locale.starts_with(lang)
        });

        if let Some(found) = language_match {
            return Some(found.value().clone());
        }

        // Fallback locale
        self.variants
            .iter()
            .find(|e| {
                let v = e.value();
                v.template_id == *template_id && v.locale == self.fallback_locale
            })
            .map(|e| e.value().clone())
    }

    /// List all variants for a template.
    pub fn list_variants(&self, template_id: &Uuid) -> Vec<LocaleVariant> {
        self.variants
            .iter()
            .filter(|e| e.value().template_id == *template_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Export variants for external translation (returns map of locale -> text).
    pub fn export_for_translation(&self, template_id: &Uuid) -> HashMap<String, String> {
        self.variants
            .iter()
            .filter(|e| e.value().template_id == *template_id)
            .map(|e| (e.value().locale.clone(), e.value().body.clone()))
            .collect()
    }

    /// Import translated content for a locale variant.
    pub fn import_translation(
        &self,
        variant_id: &Uuid,
        translated_body: String,
        translator: String,
    ) -> Result<LocaleVariant, String> {
        let mut entry = self
            .variants
            .get_mut(variant_id)
            .ok_or("Variant not found")?;
        entry.body = translated_body;
        entry.translator = Some(translator);
        entry.status = LocaleStatus::Translated;
        entry.updated_at = Utc::now();
        Ok(entry.clone())
    }
}

impl Default for LocalizationEngine {
    fn default() -> Self {
        Self::new("en-US")
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_template_builder() {
        let builder = EmailTemplateBuilder::new();
        let user = Uuid::new_v4();
        let tmpl =
            builder.create_template("Welcome".to_string(), "Welcome aboard!".to_string(), user);

        let hero = EmailBlock {
            id: Uuid::new_v4(),
            block_type: BlockType::Hero,
            content: vec![
                (
                    "image_url".to_string(),
                    "https://cdn.example.com/hero.jpg".to_string(),
                ),
                ("alt_text".to_string(), "Welcome banner".to_string()),
            ]
            .into_iter()
            .collect(),
            styles: HashMap::new(),
            mobile_styles: None,
            sort_order: 0,
            snippet_id: None,
        };
        builder.add_block(&tmpl.id, hero).unwrap();

        let text = EmailBlock {
            id: Uuid::new_v4(),
            block_type: BlockType::Text,
            content: vec![("text".to_string(), "Hello {{first_name}}!".to_string())]
                .into_iter()
                .collect(),
            styles: HashMap::new(),
            mobile_styles: None,
            sort_order: 1,
            snippet_id: None,
        };
        builder.add_block(&tmpl.id, text).unwrap();

        let tmpl = builder.get_template(&tmpl.id).unwrap();
        assert_eq!(tmpl.blocks.len(), 2);

        let html = builder.render_html(&tmpl.id).unwrap();
        assert!(html.contains("Welcome banner"));
        assert!(html.contains("{{first_name}}"));
    }

    #[test]
    fn test_html_sanitization() {
        let dirty = "<p>Hello</p><script>alert('xss')</script><img src=x onerror=alert(1)>";
        let clean = HtmlEditor::sanitize(dirty);
        assert!(!clean.to_lowercase().contains("<script"));
        assert!(!clean.to_lowercase().contains("onerror"));
    }

    #[test]
    fn test_html_linting() {
        let html = "<img src=\"test.jpg\"><a href=\"\">Click</a><font>old</font>";
        let issues = HtmlEditor::lint(html);
        assert!(issues.iter().any(|i| i.message.contains("alt text")));
        assert!(issues
            .iter()
            .any(|i| i.message.contains("empty or placeholder")));
        assert!(issues.iter().any(|i| i.message.contains("Deprecated")));
    }

    #[test]
    fn test_variable_browser() {
        let browser = VariableBrowser::new();

        let results = browser.autocomplete("first");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "first_name");

        let categories = browser.by_category();
        assert!(categories.contains_key("profile"));
        assert!(categories.contains_key("campaign"));

        // Validate with missing required var
        let data: HashMap<String, String> = HashMap::new();
        let errors = browser.validate(&data);
        assert!(errors.iter().any(|e| e.contains("email")));
    }

    #[test]
    fn test_render_with_variables() {
        let html = "<p>Hello {{first_name}}, your balance is {{points_balance}}!</p>";
        let data: HashMap<String, String> = vec![
            ("first_name".to_string(), "Jane".to_string()),
            ("points_balance".to_string(), "1250".to_string()),
        ]
        .into_iter()
        .collect();

        let rendered = HtmlEditor::render_with_data(html, &data);
        assert!(rendered.contains("Hello Jane"));
        assert!(rendered.contains("1250"));
    }

    #[test]
    fn test_email_compliance_canspam() {
        let issues = ComplianceChecker::check_email("<p>Buy now!</p>", false, false);
        assert_eq!(issues.len(), 2);
        assert!(issues
            .iter()
            .all(|i| i.severity == ComplianceSeverity::Blocking));
    }

    #[test]
    fn test_sms_compliance() {
        let issues = ComplianceChecker::check_sms(
            "Great deals await you!",
            false,
            22,
            21,
            8, // send at 22:00, quiet 21-08
        );
        assert!(issues.iter().any(|i| i.check_name == "STOP Language"));
        assert!(issues.iter().any(|i| i.check_name == "Quiet Hours"));
    }

    #[test]
    fn test_localization_fallback() {
        let engine = LocalizationEngine::new("en-US");
        let template_id = Uuid::new_v4();

        engine.add_variant(LocaleVariant {
            id: Uuid::new_v4(),
            template_id,
            locale: "en-US".to_string(),
            subject: "Welcome!".to_string(),
            body: "Hello!".to_string(),
            variables_override: HashMap::new(),
            status: LocaleStatus::Approved,
            translator: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        engine.add_variant(LocaleVariant {
            id: Uuid::new_v4(),
            template_id,
            locale: "fr-FR".to_string(),
            subject: "Bienvenue!".to_string(),
            body: "Bonjour!".to_string(),
            variables_override: HashMap::new(),
            status: LocaleStatus::Approved,
            translator: Some("translator@example.com".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        // Exact match
        let v = engine.get_variant(&template_id, "fr-FR").unwrap();
        assert_eq!(v.body, "Bonjour!");

        // Language fallback: fr-CA -> fr-FR
        let v = engine.get_variant(&template_id, "fr-CA").unwrap();
        assert_eq!(v.body, "Bonjour!");

        // Fallback locale: de-DE -> en-US
        let v = engine.get_variant(&template_id, "de-DE").unwrap();
        assert_eq!(v.body, "Hello!");

        // Export for translation
        let exported = engine.export_for_translation(&template_id);
        assert_eq!(exported.len(), 2);
    }
}
