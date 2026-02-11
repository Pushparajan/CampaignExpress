use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::info;
use uuid::Uuid;

use crate::assembler::CreativeAssembler;
use crate::scorer::VariantScorer;
use crate::types::{
    ComponentType, ComponentVariant, DcoScoreRequest, DcoScoreResponse, DcoTemplate,
    TemplateComponent, TemplateStatus, VariantPerformance,
};

/// Core DCO engine — manages templates, assembles creatives, and scores them.
#[derive(Debug, Clone)]
pub struct DcoEngine {
    templates: Arc<DashMap<Uuid, DcoTemplate>>,
    assembler: CreativeAssembler,
    scorer: VariantScorer,
}

impl DcoEngine {
    /// Create a new engine with an empty template store and default assembler/scorer.
    pub fn new() -> Self {
        Self {
            templates: Arc::new(DashMap::new()),
            assembler: CreativeAssembler::new(),
            scorer: VariantScorer::new(),
        }
    }

    /// Register (or overwrite) a template in the store. Returns the template id.
    pub fn register_template(&self, template: DcoTemplate) -> Result<Uuid> {
        let id = template.id;
        info!(%id, name = %template.name, "registering DCO template");
        self.templates.insert(id, template);
        Ok(id)
    }

    /// Retrieve a template by id (cloned).
    pub fn get_template(&self, id: &Uuid) -> Option<DcoTemplate> {
        self.templates.get(id).map(|entry| entry.clone())
    }

    /// List all registered templates.
    pub fn list_templates(&self) -> Vec<DcoTemplate> {
        self.templates
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Delete a template by id.
    pub fn delete_template(&self, id: &Uuid) -> Result<()> {
        self.templates
            .remove(id)
            .ok_or_else(|| anyhow!("template {id} not found"))?;
        info!(%id, "deleted DCO template");
        Ok(())
    }

    /// Score and assemble creatives for the given request.
    ///
    /// 1. Looks up the template.
    /// 2. Generates all valid combinations (capped by `max_variants`).
    /// 3. Scores every combination.
    /// 4. Returns the top-k assembled creatives ordered by descending score.
    pub fn score_and_assemble(&self, request: &DcoScoreRequest) -> Result<DcoScoreResponse> {
        let start = Instant::now();

        let template = self
            .get_template(&request.template_id)
            .ok_or_else(|| anyhow!("template {} not found", request.template_id))?;

        let combinations =
            self.assembler
                .generate_combinations(&template, request.max_variants);
        let total_combinations = combinations.len() as u64;

        let scores =
            self.scorer
                .score_combinations(&template, &combinations, &request.user_segments);

        // Pair combinations with scores and sort descending
        let mut scored: Vec<(usize, f32)> = scores.iter().copied().enumerate().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take up to max_variants assembled creatives
        let top_k = scored
            .iter()
            .take(request.max_variants)
            .map(|(idx, score)| {
                let mut creative = self.assembler.assemble(&template, &combinations[*idx]);
                creative.score = *score;
                creative.predicted_ctr = *score; // score approximates predicted CTR
                creative
            })
            .collect::<Vec<_>>();

        let latency = start.elapsed().as_micros() as u64;

        info!(
            template_id = %request.template_id,
            total_combinations,
            scored = top_k.len(),
            latency_us = latency,
            "score_and_assemble complete"
        );

        Ok(DcoScoreResponse {
            assembled_creatives: top_k,
            total_combinations,
            scored_combinations: total_combinations,
            latency_us: latency,
        })
    }

    /// Record an outcome (impression, click, or conversion) against a specific
    /// template + component + variant. Updates the in-memory performance counters.
    pub fn record_outcome(
        &self,
        _creative_id: &Uuid,
        template_id: &Uuid,
        component_id: &Uuid,
        variant_id: &Uuid,
        outcome: &str,
    ) {
        if let Some(mut template) = self.templates.get_mut(template_id) {
            for comp in template.components.iter_mut() {
                if &comp.id == component_id {
                    for variant in comp.variants.iter_mut() {
                        if &variant.id == variant_id {
                            match outcome {
                                "impression" => {
                                    variant.performance.impressions += 1;
                                }
                                "click" => {
                                    variant.performance.clicks += 1;
                                    // Recompute CTR
                                    if variant.performance.impressions > 0 {
                                        variant.performance.ctr = variant.performance.clicks
                                            as f64
                                            / variant.performance.impressions as f64;
                                    }
                                }
                                "conversion" => {
                                    variant.performance.conversions += 1;
                                    // Recompute CVR
                                    if variant.performance.clicks > 0 {
                                        variant.performance.cvr = variant.performance.conversions
                                            as f64
                                            / variant.performance.clicks as f64;
                                    }
                                }
                                _ => {
                                    info!(outcome, "unknown outcome type, ignored");
                                }
                            }
                            template.updated_at = Utc::now();
                            info!(
                                %template_id,
                                %component_id,
                                %variant_id,
                                outcome,
                                "recorded outcome"
                            );
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Seed two demo templates with 3 components each (headline, hero image, CTA),
    /// each component having 3 variants.
    pub fn seed_demo_templates(&self) {
        let now = Utc::now();

        for (name, desc) in [
            ("Summer Sale Banner", "Seasonal promotion creative"),
            ("Product Launch Hero", "New product launch creative"),
        ] {
            let template_id = Uuid::new_v4();

            let headline = TemplateComponent {
                id: Uuid::new_v4(),
                component_type: ComponentType::Headline,
                variants: vec![
                    make_variant("Bold Headline", "Save Big This Summer!", None),
                    make_variant("Question Headline", "Ready for Summer Savings?", None),
                    make_variant("Urgency Headline", "Limited Time Offer - Act Now!", None),
                ],
                required: true,
            };

            let hero_image = TemplateComponent {
                id: Uuid::new_v4(),
                component_type: ComponentType::HeroImage,
                variants: vec![
                    make_variant(
                        "Lifestyle Image",
                        "lifestyle",
                        Some("https://cdn.example.com/hero-lifestyle.jpg"),
                    ),
                    make_variant(
                        "Product Image",
                        "product",
                        Some("https://cdn.example.com/hero-product.jpg"),
                    ),
                    make_variant(
                        "Abstract Image",
                        "abstract",
                        Some("https://cdn.example.com/hero-abstract.jpg"),
                    ),
                ],
                required: true,
            };

            let cta = TemplateComponent {
                id: Uuid::new_v4(),
                component_type: ComponentType::Cta,
                variants: vec![
                    make_variant("Shop Now CTA", "Shop Now", None),
                    make_variant("Learn More CTA", "Learn More", None),
                    make_variant("Get Started CTA", "Get Started", None),
                ],
                required: true,
            };

            let template = DcoTemplate {
                id: template_id,
                name: name.to_string(),
                description: desc.to_string(),
                components: vec![headline, hero_image, cta],
                rules: Vec::new(),
                status: TemplateStatus::Active,
                created_at: now,
                updated_at: now,
            };

            self.templates.insert(template_id, template);
            info!(%template_id, name, "seeded demo template");
        }
    }
}

impl Default for DcoEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper: create a `ComponentVariant` with default performance.
fn make_variant(name: &str, content: &str, asset_url: Option<&str>) -> ComponentVariant {
    ComponentVariant {
        id: Uuid::new_v4(),
        name: name.to_string(),
        content: content.to_string(),
        asset_url: asset_url.map(String::from),
        metadata: serde_json::json!({}),
        performance: VariantPerformance::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_template() {
        let engine = DcoEngine::new();
        let now = Utc::now();

        let template = DcoTemplate {
            id: Uuid::new_v4(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            components: vec![],
            rules: vec![],
            status: TemplateStatus::Draft,
            created_at: now,
            updated_at: now,
        };

        let id = template.id;
        let result = engine.register_template(template);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);

        let retrieved = engine.get_template(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Template");

        // Verify list
        let all = engine.list_templates();
        assert_eq!(all.len(), 1);

        // Delete
        assert!(engine.delete_template(&id).is_ok());
        assert!(engine.get_template(&id).is_none());
    }

    #[test]
    fn test_score_and_assemble() {
        let engine = DcoEngine::new();
        engine.seed_demo_templates();

        let templates = engine.list_templates();
        assert_eq!(templates.len(), 2);

        let template = &templates[0];
        let request = DcoScoreRequest {
            template_id: template.id,
            user_segments: vec![1, 2, 3],
            context: serde_json::json!({"device": "mobile"}),
            max_variants: 5,
        };

        let response = engine.score_and_assemble(&request).unwrap();
        assert!(!response.assembled_creatives.is_empty());
        assert!(response.total_combinations > 0);
        assert!(response.scored_combinations > 0);

        // Each assembled creative should have 3 selected components
        for creative in &response.assembled_creatives {
            assert_eq!(creative.selected_components.len(), 3);
            assert!(creative.score >= 0.0);
        }
    }
}
