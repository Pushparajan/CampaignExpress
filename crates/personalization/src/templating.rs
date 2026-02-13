//! Liquid-like template engine for message personalization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateContext {
    pub user: HashMap<String, serde_json::Value>,
    pub event: HashMap<String, serde_json::Value>,
    pub campaign: HashMap<String, serde_json::Value>,
    pub catalog: HashMap<String, serde_json::Value>,
    pub connected: HashMap<String, serde_json::Value>,
    pub custom: HashMap<String, serde_json::Value>,
}

type FilterFn = Box<dyn Fn(&str) -> String + Send + Sync>;

pub struct TemplateEngine {
    filters: HashMap<String, FilterFn>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut filters: HashMap<String, FilterFn> = HashMap::new();
        filters.insert("upcase".to_string(), Box::new(|s: &str| s.to_uppercase()));
        filters.insert("downcase".to_string(), Box::new(|s: &str| s.to_lowercase()));
        filters.insert(
            "capitalize".to_string(),
            Box::new(|s: &str| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                }
            }),
        );
        filters.insert(
            "strip".to_string(),
            Box::new(|s: &str| s.trim().to_string()),
        );
        filters.insert(
            "truncate".to_string(),
            Box::new(|s: &str| {
                if s.len() > 50 {
                    format!("{}...", &s[..50])
                } else {
                    s.to_string()
                }
            }),
        );
        filters.insert(
            "default".to_string(),
            Box::new(|s: &str| {
                if s.is_empty() {
                    "N/A".to_string()
                } else {
                    s.to_string()
                }
            }),
        );
        Self { filters }
    }

    pub fn render(&self, template: &str, context: &TemplateContext) -> String {
        let mut result = template.to_string();
        result = self.render_variables(&result, "user", &context.user);
        result = self.render_variables(&result, "event", &context.event);
        result = self.render_variables(&result, "campaign", &context.campaign);
        result = self.render_variables(&result, "catalog", &context.catalog);
        result = self.render_variables(&result, "connected", &context.connected);
        result = self.render_variables(&result, "custom", &context.custom);
        result = self.render_conditionals(&result, context);
        result
    }

    fn render_variables(
        &self,
        template: &str,
        prefix: &str,
        vars: &HashMap<String, serde_json::Value>,
    ) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            let placeholder = format!("{{{{{}.{}}}}}", prefix, key);
            let display_value = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            result = result.replace(&placeholder, &display_value);

            for (filter_name, filter_fn) in &self.filters {
                let filtered_placeholder = format!("{{{{{}.{} | {}}}}}", prefix, key, filter_name);
                if result.contains(&filtered_placeholder) {
                    let filtered = filter_fn(&display_value);
                    result = result.replace(&filtered_placeholder, &filtered);
                }
            }
        }
        result
    }

    fn render_conditionals(&self, template: &str, _context: &TemplateContext) -> String {
        // Simplified conditional rendering: strip {% if %} / {% endif %} blocks
        // In production, this would use a proper parser
        template.to_string()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
