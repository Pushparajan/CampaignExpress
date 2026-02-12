//! Message template rendering engine.

use crate::types::{MessageTemplate, TemplateStatus, TemplateVariable};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Simple template renderer using {{variable}} syntax
pub struct TemplateRenderer {
    templates: HashMap<Uuid, MessageTemplate>,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register_template(&mut self, template: MessageTemplate) -> Uuid {
        let id = template.id;
        self.templates.insert(id, template);
        id
    }

    pub fn get_template(&self, id: &Uuid) -> Option<&MessageTemplate> {
        self.templates.get(id)
    }

    pub fn list_templates(&self) -> Vec<&MessageTemplate> {
        self.templates.values().collect()
    }

    /// Render a template with the given variables
    pub fn render(
        &self,
        template_id: &Uuid,
        variables: &HashMap<String, String>,
    ) -> Option<RenderedMessage> {
        let template = self.templates.get(template_id)?;
        if template.status != TemplateStatus::Active {
            return None;
        }

        let body = self.substitute(&template.body_template, variables, &template.variables);
        let subject = template
            .subject
            .as_ref()
            .map(|s| self.substitute(s, variables, &template.variables));

        Some(RenderedMessage {
            template_id: *template_id,
            channel: template.channel.clone(),
            subject,
            body,
            rendered_at: Utc::now(),
        })
    }

    fn substitute(
        &self,
        template_str: &str,
        variables: &HashMap<String, String>,
        var_defs: &[TemplateVariable],
    ) -> String {
        let mut result = template_str.to_string();
        for var_def in var_defs {
            let placeholder = format!("{{{{{}}}}}", var_def.name);
            let value = variables
                .get(&var_def.name)
                .cloned()
                .or_else(|| var_def.default_value.clone())
                .unwrap_or_default();
            result = result.replace(&placeholder, &value);
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct RenderedMessage {
    pub template_id: Uuid,
    pub channel: String,
    pub subject: Option<String>,
    pub body: String,
    pub rendered_at: chrono::DateTime<chrono::Utc>,
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}
