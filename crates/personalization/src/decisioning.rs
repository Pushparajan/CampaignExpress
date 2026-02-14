//! Real-time 1:1 decision API with multi-objective optimization,
//! explainability, and simulation mode.
//!
//! Addresses FR-1TO1-001 through FR-1TO1-005.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ─── Decision Request / Response (FR-1TO1-001) ───────────────────────

/// A real-time decision request for 1:1 personalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub request_id: Uuid,
    pub user_id: String,
    pub context: DecisionContext,
    pub channel: String,
    pub placement_id: Option<String>,
    pub num_offers: u32,
    pub objectives: Vec<OptimizationObjective>,
    /// If true, return explanation factors for each decision.
    pub explain: bool,
    /// If true, run in simulation mode (no side effects).
    pub simulate: bool,
    pub timeout_ms: u64,
    pub requested_at: DateTime<Utc>,
}

/// Contextual information for decision-making.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub device_type: Option<String>,
    pub geo_region: Option<String>,
    pub session_id: Option<String>,
    pub page_url: Option<String>,
    pub referrer: Option<String>,
    pub user_segments: Vec<u32>,
    pub user_features: std::collections::HashMap<String, f64>,
    pub time_of_day: Option<String>,
    pub day_of_week: Option<String>,
}

/// A selected offer in the decision response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOffer {
    pub offer_id: String,
    pub score: f64,
    pub rank: u32,
    pub creative_id: Option<String>,
    pub explanation: Option<DecisionExplanation>,
}

/// Full decision response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub decision_id: Uuid,
    pub request_id: Uuid,
    pub user_id: String,
    pub offers: Vec<DecisionOffer>,
    pub latency_ms: u64,
    pub model_version: String,
    pub is_simulation: bool,
    pub decided_at: DateTime<Utc>,
}

// ─── Multi-Objective Optimization (FR-1TO1-002) ──────────────────────

/// Optimization objective with weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationObjective {
    pub metric: ObjectiveMetric,
    pub weight: f64,
}

/// Metrics that can be optimized.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveMetric {
    ClickThroughRate,
    ConversionRate,
    Revenue,
    LifetimeValue,
    Engagement,
    Retention,
}

/// Internal scored candidate during multi-objective optimization.
#[derive(Debug, Clone)]
struct ScoredCandidate {
    offer_id: String,
    creative_id: Option<String>,
    objective_scores: std::collections::HashMap<ObjectiveMetric, f64>,
    blended_score: f64,
}

// ─── Explainability (FR-1TO1-003) ────────────────────────────────────

/// Explanation of why an offer was selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionExplanation {
    pub factors: Vec<ExplanationFactor>,
    pub model_confidence: f64,
    pub exploration_bonus: f64,
}

/// A single factor contributing to the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationFactor {
    pub name: String,
    pub category: FactorCategory,
    pub contribution: f64,
    pub description: String,
}

/// Category of explanation factor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactorCategory {
    SegmentMembership,
    BehavioralSignal,
    ContextualRelevance,
    ModelPrediction,
    BusinessRule,
    ExplorationBonus,
}

// ─── Simulation Mode (FR-1TO1-004) ──────────────────────────────────

/// Result of a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub simulation_id: Uuid,
    pub scenario: SimulationScenario,
    pub decisions: Vec<DecisionResponse>,
    pub aggregate_metrics: SimulationMetrics,
    pub ran_at: DateTime<Utc>,
}

/// What-if scenario parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationScenario {
    pub name: String,
    pub description: String,
    pub overrides: std::collections::HashMap<String, serde_json::Value>,
    pub sample_size: u32,
}

/// Aggregated metrics from a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub avg_score: f64,
    pub predicted_ctr: f64,
    pub predicted_conversion_rate: f64,
    pub predicted_revenue: f64,
    pub offer_diversity: f64,
    pub coverage_percent: f64,
}

// ─── Decision Engine (FR-1TO1-005) ──────────────────────────────────

/// Offer catalog entry used for scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferCandidate {
    pub offer_id: String,
    pub creative_id: Option<String>,
    pub eligible_segments: Vec<u32>,
    pub base_scores: std::collections::HashMap<String, f64>,
    pub channel: String,
    pub active: bool,
}

/// Real-time decision engine.
pub struct DecisionEngine {
    offers: DashMap<String, OfferCandidate>,
    decision_log: DashMap<Uuid, DecisionResponse>,
    model_version: String,
}

impl DecisionEngine {
    pub fn new() -> Self {
        info!("Decision engine initialized");
        Self {
            offers: DashMap::new(),
            decision_log: DashMap::new(),
            model_version: "v1.0.0".to_string(),
        }
    }

    /// Register an offer candidate for decisioning.
    pub fn register_offer(&self, offer: OfferCandidate) {
        self.offers.insert(offer.offer_id.clone(), offer);
    }

    /// Execute a real-time decision.
    pub fn decide(&self, request: &DecisionRequest) -> DecisionResponse {
        let start = std::time::Instant::now();

        // Filter eligible offers
        let candidates: Vec<OfferCandidate> = self
            .offers
            .iter()
            .filter(|e| {
                let o = e.value();
                o.active
                    && o.channel == request.channel
                    && (o.eligible_segments.is_empty()
                        || o.eligible_segments
                            .iter()
                            .any(|s| request.context.user_segments.contains(s)))
            })
            .map(|e| e.value().clone())
            .collect();

        // Score each candidate using multi-objective blending
        let mut scored: Vec<ScoredCandidate> = candidates
            .iter()
            .map(|c| {
                let mut obj_scores = std::collections::HashMap::new();
                let mut blended = 0.0;

                for obj in &request.objectives {
                    let score = self.score_objective(c, &obj.metric, &request.context);
                    blended += score * obj.weight;
                    obj_scores.insert(obj.metric.clone(), score);
                }

                // Add exploration bonus
                let exploration = 0.01 * rand::random::<f64>();
                blended += exploration;

                ScoredCandidate {
                    offer_id: c.offer_id.clone(),
                    creative_id: c.creative_id.clone(),
                    objective_scores: obj_scores,
                    blended_score: blended,
                }
            })
            .collect();

        // Sort descending by blended score
        scored.sort_by(|a, b| {
            b.blended_score
                .partial_cmp(&a.blended_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top N
        let top_n = scored
            .into_iter()
            .take(request.num_offers as usize)
            .enumerate()
            .map(|(i, s)| {
                let explanation = if request.explain {
                    Some(self.build_explanation(&s, &request.context))
                } else {
                    None
                };

                DecisionOffer {
                    offer_id: s.offer_id,
                    score: s.blended_score,
                    rank: i as u32 + 1,
                    creative_id: s.creative_id,
                    explanation,
                }
            })
            .collect();

        let latency = start.elapsed().as_millis() as u64;
        let decision_id = Uuid::new_v4();

        let response = DecisionResponse {
            decision_id,
            request_id: request.request_id,
            user_id: request.user_id.clone(),
            offers: top_n,
            latency_ms: latency,
            model_version: self.model_version.clone(),
            is_simulation: request.simulate,
            decided_at: Utc::now(),
        };

        // Log decision (skip for simulations)
        if !request.simulate {
            self.decision_log.insert(decision_id, response.clone());
        }

        response
    }

    /// Score a single offer for a specific objective.
    fn score_objective(
        &self,
        candidate: &OfferCandidate,
        metric: &ObjectiveMetric,
        context: &DecisionContext,
    ) -> f64 {
        let key = format!("{:?}", metric);
        let base = candidate.base_scores.get(&key).copied().unwrap_or(0.5);

        // Contextual adjustments
        let mut adjustment = 0.0;

        // Segment affinity boost
        let segment_overlap = candidate
            .eligible_segments
            .iter()
            .filter(|s| context.user_segments.contains(s))
            .count();
        if !candidate.eligible_segments.is_empty() {
            adjustment += 0.1 * (segment_overlap as f64 / candidate.eligible_segments.len() as f64);
        }

        // Feature-based boost
        if let Some(recency) = context.user_features.get("recency_score") {
            adjustment += 0.05 * recency;
        }

        (base + adjustment).clamp(0.0, 1.0)
    }

    /// Build explanation factors for a scored candidate.
    fn build_explanation(
        &self,
        scored: &ScoredCandidate,
        context: &DecisionContext,
    ) -> DecisionExplanation {
        let mut factors = Vec::new();

        // Add objective-based factors
        for (metric, score) in &scored.objective_scores {
            factors.push(ExplanationFactor {
                name: format!("{:?}", metric),
                category: FactorCategory::ModelPrediction,
                contribution: *score,
                description: format!("{:?} predicted score: {:.3}", metric, score),
            });
        }

        // Segment factor
        if !context.user_segments.is_empty() {
            factors.push(ExplanationFactor {
                name: "segment_match".to_string(),
                category: FactorCategory::SegmentMembership,
                contribution: 0.1,
                description: format!("User belongs to {} segments", context.user_segments.len()),
            });
        }

        // Recency factor
        if let Some(recency) = context.user_features.get("recency_score") {
            factors.push(ExplanationFactor {
                name: "recency_score".to_string(),
                category: FactorCategory::BehavioralSignal,
                contribution: 0.05 * recency,
                description: format!("Recency score: {:.2}", recency),
            });
        }

        DecisionExplanation {
            factors,
            model_confidence: 0.85,
            exploration_bonus: 0.01,
        }
    }

    /// Run a simulation with overridden parameters.
    pub fn simulate(
        &self,
        scenario: SimulationScenario,
        base_request: &DecisionRequest,
    ) -> SimulationResult {
        let mut decisions = Vec::new();
        let mut total_score = 0.0;
        let mut offer_set = std::collections::HashSet::new();

        let count = scenario.sample_size.min(100);
        for _ in 0..count {
            let mut req = base_request.clone();
            req.simulate = true;
            req.request_id = Uuid::new_v4();

            let resp = self.decide(&req);
            for offer in &resp.offers {
                total_score += offer.score;
                offer_set.insert(offer.offer_id.clone());
            }
            decisions.push(resp);
        }

        let total_offers = self.offers.len().max(1);
        let avg_score = if decisions.is_empty() {
            0.0
        } else {
            total_score / (decisions.len() as f64 * base_request.num_offers as f64).max(1.0)
        };

        SimulationResult {
            simulation_id: Uuid::new_v4(),
            scenario,
            decisions,
            aggregate_metrics: SimulationMetrics {
                avg_score,
                predicted_ctr: avg_score * 0.05,
                predicted_conversion_rate: avg_score * 0.02,
                predicted_revenue: avg_score * 10.0,
                offer_diversity: offer_set.len() as f64 / total_offers as f64,
                coverage_percent: offer_set.len() as f64 / total_offers as f64 * 100.0,
            },
            ran_at: Utc::now(),
        }
    }

    /// Look up a past decision by ID.
    pub fn get_decision(&self, decision_id: &Uuid) -> Option<DecisionResponse> {
        self.decision_log.get(decision_id).map(|d| d.clone())
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine_with_offers() -> DecisionEngine {
        let engine = DecisionEngine::new();
        for i in 0..5 {
            let mut scores = std::collections::HashMap::new();
            scores.insert("ClickThroughRate".to_string(), 0.3 + i as f64 * 0.1);
            scores.insert("ConversionRate".to_string(), 0.1 + i as f64 * 0.05);
            scores.insert("Revenue".to_string(), 0.5 + i as f64 * 0.08);

            engine.register_offer(OfferCandidate {
                offer_id: format!("offer_{}", i),
                creative_id: Some(format!("creative_{}", i)),
                eligible_segments: vec![1, 2, 3],
                base_scores: scores,
                channel: "web".to_string(),
                active: true,
            });
        }
        engine
    }

    fn make_request() -> DecisionRequest {
        DecisionRequest {
            request_id: Uuid::new_v4(),
            user_id: "user_123".to_string(),
            context: DecisionContext {
                device_type: Some("mobile".to_string()),
                geo_region: Some("US".to_string()),
                session_id: Some("sess_abc".to_string()),
                page_url: None,
                referrer: None,
                user_segments: vec![1, 3],
                user_features: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("recency_score".to_string(), 0.8);
                    m
                },
                time_of_day: Some("afternoon".to_string()),
                day_of_week: Some("tuesday".to_string()),
            },
            channel: "web".to_string(),
            placement_id: None,
            num_offers: 3,
            objectives: vec![
                OptimizationObjective {
                    metric: ObjectiveMetric::ClickThroughRate,
                    weight: 0.5,
                },
                OptimizationObjective {
                    metric: ObjectiveMetric::Revenue,
                    weight: 0.3,
                },
                OptimizationObjective {
                    metric: ObjectiveMetric::ConversionRate,
                    weight: 0.2,
                },
            ],
            explain: true,
            simulate: false,
            timeout_ms: 50,
            requested_at: Utc::now(),
        }
    }

    #[test]
    fn test_decision_returns_ranked_offers() {
        let engine = make_engine_with_offers();
        let req = make_request();
        let resp = engine.decide(&req);

        assert_eq!(resp.offers.len(), 3);
        assert_eq!(resp.offers[0].rank, 1);
        assert_eq!(resp.offers[1].rank, 2);
        assert_eq!(resp.offers[2].rank, 3);
        assert!(resp.offers[0].score >= resp.offers[1].score);
        assert!(!resp.is_simulation);
    }

    #[test]
    fn test_decision_with_explanation() {
        let engine = make_engine_with_offers();
        let req = make_request();
        let resp = engine.decide(&req);

        let first = &resp.offers[0];
        assert!(first.explanation.is_some());
        let expl = first.explanation.as_ref().unwrap();
        assert!(!expl.factors.is_empty());
        assert!(expl.model_confidence > 0.0);
    }

    #[test]
    fn test_decision_logged() {
        let engine = make_engine_with_offers();
        let req = make_request();
        let resp = engine.decide(&req);

        let logged = engine.get_decision(&resp.decision_id);
        assert!(logged.is_some());
        assert_eq!(logged.unwrap().decision_id, resp.decision_id);
    }

    #[test]
    fn test_simulation_mode_not_logged() {
        let engine = make_engine_with_offers();
        let mut req = make_request();
        req.simulate = true;
        let resp = engine.decide(&req);

        assert!(resp.is_simulation);
        // Simulations should not be logged
        assert!(engine.get_decision(&resp.decision_id).is_none());
    }

    #[test]
    fn test_multi_objective_blending() {
        let engine = make_engine_with_offers();
        let mut req = make_request();
        // Only optimize for revenue
        req.objectives = vec![OptimizationObjective {
            metric: ObjectiveMetric::Revenue,
            weight: 1.0,
        }];
        req.explain = false;
        let resp = engine.decide(&req);

        assert_eq!(resp.offers.len(), 3);
        // Higher-indexed offers have higher revenue scores
        assert!(resp.offers[0].score > 0.0);
    }

    #[test]
    fn test_simulation_scenario() {
        let engine = make_engine_with_offers();
        let req = make_request();

        let scenario = SimulationScenario {
            name: "What-if higher frequency".to_string(),
            description: "Test impact of removing frequency cap".to_string(),
            overrides: std::collections::HashMap::new(),
            sample_size: 10,
        };

        let result = engine.simulate(scenario, &req);
        assert_eq!(result.decisions.len(), 10);
        assert!(result.aggregate_metrics.avg_score > 0.0);
        assert!(result.aggregate_metrics.offer_diversity > 0.0);
    }

    #[test]
    fn test_channel_filtering() {
        let engine = make_engine_with_offers();
        let mut req = make_request();
        req.channel = "sms".to_string(); // No offers for SMS
        let resp = engine.decide(&req);

        assert!(resp.offers.is_empty());
    }
}
