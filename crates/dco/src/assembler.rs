use std::collections::HashMap;
use std::time::Instant;

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::types::{AssembledCreative, ComponentType, DcoTemplate, SelectedComponent};

/// Generates and assembles creative combinations from DCO templates.
#[derive(Debug, Clone)]
pub struct CreativeAssembler;

impl CreativeAssembler {
    /// Create a new assembler instance.
    pub fn new() -> Self {
        Self
    }

    /// Generate all component-variant combinations for a template, capped at `max`.
    ///
    /// Each combination is a `Vec<(component_id, variant_id)>` representing one
    /// possible assembled creative.
    pub fn generate_combinations(
        &self,
        template: &DcoTemplate,
        max: usize,
    ) -> Vec<Vec<(Uuid, Uuid)>> {
        let component_options: Vec<Vec<(Uuid, Uuid)>> = template
            .components
            .iter()
            .map(|comp| {
                comp.variants
                    .iter()
                    .map(|v| (comp.id, v.id))
                    .collect::<Vec<_>>()
            })
            .collect();

        if component_options.is_empty() {
            return Vec::new();
        }

        // Cartesian product via iterative expansion.
        // We expand through every component so that all combinations are complete
        // (i.e. contain one selection per component). The intermediate set is
        // capped at `max` after each component to bound memory usage.
        let mut combinations: Vec<Vec<(Uuid, Uuid)>> = vec![vec![]];

        for options in &component_options {
            let mut next = Vec::new();
            for existing in &combinations {
                for pair in options {
                    let mut combo = existing.clone();
                    combo.push(*pair);
                    next.push(combo);
                }
            }
            // Truncate intermediate results to keep memory bounded while
            // ensuring every remaining combination is expanded with all
            // remaining components.
            if next.len() > max {
                info!(
                    count = next.len(),
                    max, "combination limit reached, truncating"
                );
                next.truncate(max);
            }
            combinations = next;
        }

        info!(
            count = combinations.len(),
            "generated creative combinations"
        );
        combinations
    }

    /// Assemble a single creative from a template and a chosen combination.
    pub fn assemble(
        &self,
        template: &DcoTemplate,
        combination: &[(Uuid, Uuid)],
    ) -> AssembledCreative {
        let start = Instant::now();
        let mut selected_components = HashMap::new();

        for (component_id, variant_id) in combination {
            if let Some(comp) = template.components.iter().find(|c| &c.id == component_id) {
                if let Some(variant) = comp.variants.iter().find(|v| &v.id == variant_id) {
                    let key = component_type_key(&comp.component_type);
                    selected_components.insert(
                        key,
                        SelectedComponent {
                            component_id: *component_id,
                            variant_id: *variant_id,
                            content: variant.content.clone(),
                            asset_url: variant.asset_url.clone(),
                        },
                    );
                }
            }
        }

        let latency = start.elapsed().as_micros() as u64;

        AssembledCreative {
            id: Uuid::new_v4(),
            template_id: template.id,
            selected_components,
            score: 0.0,
            predicted_ctr: 0.0,
            assembly_latency_us: latency,
            timestamp: Utc::now(),
        }
    }
}

impl Default for CreativeAssembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a `ComponentType` to a stable string key for the selected-components map.
fn component_type_key(ct: &ComponentType) -> String {
    match ct {
        ComponentType::Headline => "headline".to_string(),
        ComponentType::SubHeadline => "sub_headline".to_string(),
        ComponentType::BodyText => "body_text".to_string(),
        ComponentType::HeroImage => "hero_image".to_string(),
        ComponentType::Logo => "logo".to_string(),
        ComponentType::Cta => "cta".to_string(),
        ComponentType::BackgroundColor => "background_color".to_string(),
        ComponentType::BorderStyle => "border_style".to_string(),
        ComponentType::ProductImage => "product_image".to_string(),
        ComponentType::PriceTag => "price_tag".to_string(),
        ComponentType::DiscountBadge => "discount_badge".to_string(),
    }
}
