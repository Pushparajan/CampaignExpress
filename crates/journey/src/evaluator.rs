use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::types::{
    ActionType, DecisionConfig, JourneyInstance, JourneyStep, SplitConfig, SplitType, StepType,
};

/// Result of evaluating a single journey step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepResult {
    ExecuteAction {
        action_type: ActionType,
        next_step: Option<Uuid>,
    },
    Wait {
        duration_secs: u64,
        next_step: Option<Uuid>,
    },
    Transition {
        next_step: Uuid,
    },
    Complete,
    Error(String),
}

/// Evaluates journey steps and conditions for a given instance context.
#[derive(Debug, Clone)]
pub struct JourneyEvaluator;

impl JourneyEvaluator {
    /// Creates a new evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates a single step in the context of the given journey instance and
    /// returns a `StepResult` describing what should happen next.
    pub fn evaluate_step(
        &self,
        step: &JourneyStep,
        instance: &JourneyInstance,
    ) -> Result<StepResult> {
        info!(
            step_id = %step.id,
            instance_id = %instance.id,
            "Evaluating journey step"
        );

        match &step.step_type {
            StepType::Action(action_type) => {
                let next_step = self.resolve_next_step(step, &instance.context);
                Ok(StepResult::ExecuteAction {
                    action_type: action_type.clone(),
                    next_step,
                })
            }
            StepType::Wait(wait_config) => {
                let next_step = self.resolve_next_step(step, &instance.context);
                Ok(StepResult::Wait {
                    duration_secs: wait_config.duration_secs,
                    next_step,
                })
            }
            StepType::Decision(decision_config) => {
                self.evaluate_decision(decision_config, &instance.context)
            }
            StepType::Split(split_config) => self.evaluate_split(split_config),
            StepType::Exit(exit_config) => {
                info!(reason = %exit_config.reason, "Journey step is an exit");
                Ok(StepResult::Complete)
            }
        }
    }

    /// Evaluates a simple condition expression against a JSON context.
    ///
    /// Supports:
    /// - `"always"` -> true
    /// - `"never"` -> false
    /// - Any other string -> treated as a key-existence check on the context object
    pub fn evaluate_condition(&self, condition: &str, context: &serde_json::Value) -> bool {
        match condition {
            "always" => true,
            "never" => false,
            key => {
                if let Some(obj) = context.as_object() {
                    obj.contains_key(key)
                } else {
                    false
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Picks the first matching transition from `next_steps`, falling back to
    /// the first unconditional transition.
    fn resolve_next_step(
        &self,
        step: &JourneyStep,
        context: &serde_json::Value,
    ) -> Option<Uuid> {
        for transition in &step.next_steps {
            match &transition.condition {
                Some(cond) => {
                    if self.evaluate_condition(cond, context) {
                        return Some(transition.target_step);
                    }
                }
                None => return Some(transition.target_step),
            }
        }
        None
    }

    /// Evaluates a decision step by iterating branches and selecting the first
    /// whose condition evaluates to true.
    fn evaluate_decision(
        &self,
        config: &DecisionConfig,
        context: &serde_json::Value,
    ) -> Result<StepResult> {
        for branch in &config.branches {
            if self.evaluate_condition(&branch.condition, context) {
                return Ok(StepResult::Transition {
                    next_step: branch.next_step,
                });
            }
        }
        Err(anyhow!("No matching branch found in decision step"))
    }

    /// Evaluates a split step by choosing a variant according to the split type.
    fn evaluate_split(&self, config: &SplitConfig) -> Result<StepResult> {
        if config.variants.is_empty() {
            return Err(anyhow!("Split step has no variants"));
        }

        let chosen = match config.split_type {
            SplitType::Random => {
                use rand::Rng;
                let total_weight: f32 = config.variants.iter().map(|v| v.weight).sum();
                let mut rng = rand::thread_rng();
                let mut roll: f32 = rng.gen::<f32>() * total_weight;
                let mut selected = &config.variants[0];
                for variant in &config.variants {
                    roll -= variant.weight;
                    if roll <= 0.0 {
                        selected = variant;
                        break;
                    }
                }
                selected
            }
            SplitType::Deterministic => {
                // Deterministic: always pick the highest-weight variant.
                config
                    .variants
                    .iter()
                    .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap()
            }
        };

        info!(variant = %chosen.name, "Split step resolved");

        Ok(StepResult::Transition {
            next_step: chosen.next_step,
        })
    }
}

impl Default for JourneyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
