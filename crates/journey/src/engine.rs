use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::info;
use uuid::Uuid;

use campaign_core::event_bus::{make_event, EventSink};
use campaign_core::types::EventType;

use crate::evaluator::{JourneyEvaluator, StepResult};
use crate::types::{
    ActionType, DecisionBranch, DecisionConfig, ExitConfig, InstanceStatus, Journey,
    JourneyInstance, JourneyStats, JourneyStatus, JourneyStep, JourneyTrigger, StepExecution,
    StepTransition, StepType, WaitConfig,
};

/// Core orchestration engine — manages journey definitions and user instances.
#[derive(Clone)]
pub struct JourneyEngine {
    journeys: Arc<DashMap<Uuid, Journey>>,
    instances: Arc<DashMap<Uuid, JourneyInstance>>,
    evaluator: Arc<JourneyEvaluator>,
    event_sink: Arc<dyn EventSink>,
}

impl std::fmt::Debug for JourneyEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JourneyEngine")
            .field("journeys", &self.journeys.len())
            .field("instances", &self.instances.len())
            .finish()
    }
}

impl JourneyEngine {
    /// Creates a new engine with empty maps and a default evaluator.
    pub fn new() -> Self {
        Self {
            journeys: Arc::new(DashMap::new()),
            instances: Arc::new(DashMap::new()),
            evaluator: Arc::new(JourneyEvaluator::new()),
            event_sink: campaign_core::event_bus::noop_sink(),
        }
    }

    /// Attach an event sink for emitting analytics events.
    pub fn with_event_sink(mut self, sink: Arc<dyn EventSink>) -> Self {
        self.event_sink = sink;
        self
    }

    /// Stores a journey and returns its id.
    pub fn create_journey(&self, journey: Journey) -> Result<Uuid> {
        let id = journey.id;
        info!(journey_id = %id, name = %journey.name, "Creating journey");
        self.journeys.insert(id, journey);
        Ok(id)
    }

    /// Returns a clone of the journey with the given id, if it exists.
    pub fn get_journey(&self, id: &Uuid) -> Option<Journey> {
        self.journeys.get(id).map(|r| r.clone())
    }

    /// Returns all journeys.
    pub fn list_journeys(&self) -> Vec<Journey> {
        self.journeys.iter().map(|r| r.value().clone()).collect()
    }

    /// Updates the status and `updated_at` timestamp of a journey.
    pub fn update_journey_status(&self, id: &Uuid, status: JourneyStatus) -> Result<()> {
        let mut entry = self
            .journeys
            .get_mut(id)
            .ok_or_else(|| anyhow!("Journey {} not found", id))?;
        info!(journey_id = %id, ?status, "Updating journey status");
        entry.status = status;
        entry.updated_at = Utc::now();
        Ok(())
    }

    /// Removes a journey from the engine.
    pub fn delete_journey(&self, id: &Uuid) -> Result<()> {
        self.journeys
            .remove(id)
            .ok_or_else(|| anyhow!("Journey {} not found", id))?;
        info!(journey_id = %id, "Deleted journey");
        Ok(())
    }

    /// Creates a new `JourneyInstance` for the given user, positioned at the
    /// first step of the journey.
    pub fn enter_journey(&self, journey_id: &Uuid, user_id: &str) -> Result<Uuid> {
        let journey = self
            .journeys
            .get(journey_id)
            .ok_or_else(|| anyhow!("Journey {} not found", journey_id))?;

        if journey.status != JourneyStatus::Active {
            return Err(anyhow!("Journey {} is not active", journey_id));
        }

        let first_step = journey
            .steps
            .first()
            .ok_or_else(|| anyhow!("Journey {} has no steps", journey_id))?;

        let instance_id = Uuid::new_v4();
        let now = Utc::now();
        let instance = JourneyInstance {
            id: instance_id,
            journey_id: *journey_id,
            user_id: user_id.to_string(),
            current_step_id: first_step.id,
            status: InstanceStatus::Active,
            context: serde_json::json!({}),
            entered_at: now,
            updated_at: now,
            step_history: Vec::new(),
        };

        info!(
            instance_id = %instance_id,
            journey_id = %journey_id,
            user_id = %user_id,
            "User entered journey"
        );

        // Emit JourneyEntered event
        self.event_sink.emit(make_event(
            EventType::JourneyEntered,
            instance_id.to_string(),
            Some(user_id.to_string()),
            None,
        ));

        self.instances.insert(instance_id, instance);
        Ok(instance_id)
    }

    /// Evaluates the current step for the given instance and advances it.
    pub fn process_step(&self, instance_id: &Uuid) -> Result<StepResult> {
        let mut instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow!("Instance {} not found", instance_id))?;

        let journey = self
            .journeys
            .get(&instance.journey_id)
            .ok_or_else(|| anyhow!("Journey {} not found", instance.journey_id))?;

        let step = journey
            .steps
            .iter()
            .find(|s| s.id == instance.current_step_id)
            .ok_or_else(|| {
                anyhow!(
                    "Step {} not found in journey {}",
                    instance.current_step_id,
                    journey.id
                )
            })?;

        let result = self.evaluator.evaluate_step(step, &instance)?;

        // Record execution in history.
        let now = Utc::now();
        let step_type_label = match &step.step_type {
            StepType::Action(_) => "action",
            StepType::Wait(_) => "wait",
            StepType::Decision(_) => "decision",
            StepType::Split(_) => "split",
            StepType::Exit(_) => "exit",
        };
        instance.step_history.push(StepExecution {
            step_id: step.id,
            step_type: step_type_label.to_string(),
            started_at: now,
            completed_at: Some(now),
            result: serde_json::to_value(&result).unwrap_or_default(),
        });

        // Advance instance based on result.
        let user_id = instance.user_id.clone();
        let inst_id = instance.id;
        match &result {
            StepResult::ExecuteAction { next_step, .. } => {
                if let Some(next) = next_step {
                    instance.current_step_id = *next;
                    instance.status = InstanceStatus::Active;
                } else {
                    instance.status = InstanceStatus::Completed;
                }
            }
            StepResult::Wait { next_step, .. } => {
                instance.status = InstanceStatus::Waiting;
                if let Some(next) = next_step {
                    instance.current_step_id = *next;
                }
            }
            StepResult::Transition { next_step } => {
                instance.current_step_id = *next_step;
                instance.status = InstanceStatus::Active;
            }
            StepResult::Complete => {
                instance.status = InstanceStatus::Completed;
            }
            StepResult::Error(msg) => {
                info!(error = %msg, "Step evaluation error");
                instance.status = InstanceStatus::Error;
            }
        }

        instance.updated_at = now;

        // Emit journey step event
        let event_type = match instance.status {
            InstanceStatus::Completed => EventType::JourneyCompleted,
            InstanceStatus::Error => EventType::JourneyExited,
            _ => EventType::JourneyStepCompleted,
        };
        self.event_sink.emit(make_event(
            event_type,
            inst_id.to_string(),
            Some(user_id),
            None,
        ));

        Ok(result)
    }

    /// Returns campaign IDs that should be suppressed because the user is in
    /// an active journey that contains a `SuppressBid` action for those
    /// campaigns.
    pub fn check_suppressions(&self, user_id: &str, campaign_ids: &[String]) -> Vec<String> {
        let mut suppressed: Vec<String> = Vec::new();

        // Collect active instances for this user.
        let user_instances: Vec<JourneyInstance> = self
            .instances
            .iter()
            .filter(|r| r.value().user_id == user_id && r.value().status == InstanceStatus::Active)
            .map(|r| r.value().clone())
            .collect();

        for inst in &user_instances {
            if let Some(journey) = self.journeys.get(&inst.journey_id) {
                for step in &journey.steps {
                    if let StepType::Action(ActionType::SuppressBid) = &step.step_type {
                        // If this journey has a SuppressBid action, suppress the
                        // campaigns from the journey trigger.
                        if let JourneyTrigger::BidContext {
                            campaign_ids: trigger_ids,
                        } = &journey.trigger
                        {
                            for cid in campaign_ids {
                                if trigger_ids.contains(cid) && !suppressed.contains(cid) {
                                    suppressed.push(cid.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        suppressed
    }

    /// Computes aggregate statistics for the given journey from its instances.
    pub fn get_stats(&self, journey_id: &Uuid) -> JourneyStats {
        let mut total_entered: u64 = 0;
        let mut active: u64 = 0;
        let mut completed: u64 = 0;
        let mut exited: u64 = 0;
        let mut error: u64 = 0;
        let mut total_completion_secs: f64 = 0.0;
        let mut completion_count: u64 = 0;
        let mut step_enter_counts: HashMap<String, u64> = HashMap::new();
        let mut step_complete_counts: HashMap<String, u64> = HashMap::new();

        for entry in self.instances.iter() {
            let inst = entry.value();
            if inst.journey_id != *journey_id {
                continue;
            }
            total_entered += 1;
            match inst.status {
                InstanceStatus::Active => active += 1,
                InstanceStatus::Waiting => {} // Waiting instances are NOT active
                InstanceStatus::Completed => {
                    completed += 1;
                    let duration = inst
                        .updated_at
                        .signed_duration_since(inst.entered_at)
                        .num_seconds() as f64;
                    total_completion_secs += duration;
                    completion_count += 1;
                }
                InstanceStatus::Exited => exited += 1,
                InstanceStatus::Error => error += 1,
            }

            // Track step-level conversion.
            for exec in inst.step_history.iter() {
                let key = exec.step_id.to_string();
                *step_enter_counts.entry(key.clone()).or_insert(0) += 1;
                if exec.completed_at.is_some() {
                    *step_complete_counts.entry(key).or_insert(0) += 1;
                }
            }
        }

        let avg_completion_time_secs = if completion_count > 0 {
            total_completion_secs / completion_count as f64
        } else {
            0.0
        };

        let mut step_conversion_rates: HashMap<String, f64> = HashMap::new();
        for (step_key, entered) in &step_enter_counts {
            let completed_count = step_complete_counts.get(step_key).copied().unwrap_or(0);
            let rate = if *entered > 0 {
                completed_count as f64 / *entered as f64
            } else {
                0.0
            };
            step_conversion_rates.insert(step_key.clone(), rate);
        }

        JourneyStats {
            journey_id: *journey_id,
            total_entered,
            active,
            completed,
            exited,
            error,
            avg_completion_time_secs,
            step_conversion_rates,
        }
    }

    /// Seeds three demo journeys for development and testing.
    pub fn seed_demo_journeys(&self) {
        info!("Seeding demo journeys");

        let now = Utc::now();

        // ---- 1. Welcome Series (email sequence) ----
        let welcome_step1_id = Uuid::new_v4();
        let welcome_step2_id = Uuid::new_v4();
        let welcome_step3_id = Uuid::new_v4();
        let welcome_step4_id = Uuid::new_v4();

        let welcome = Journey {
            id: Uuid::new_v4(),
            name: "Welcome Series".to_string(),
            description: "Onboarding email sequence for new users".to_string(),
            status: JourneyStatus::Active,
            trigger: JourneyTrigger::EventBased {
                event_type: "user_signup".to_string(),
                filters: serde_json::json!({}),
            },
            steps: vec![
                JourneyStep {
                    id: welcome_step1_id,
                    step_type: StepType::Action(ActionType::SendEmail),
                    config: serde_json::json!({"template": "welcome_email"}),
                    position: 0,
                    next_steps: vec![StepTransition {
                        target_step: welcome_step2_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: welcome_step2_id,
                    step_type: StepType::Wait(WaitConfig {
                        duration_secs: 86400,
                        until_event: None,
                    }),
                    config: serde_json::json!({}),
                    position: 1,
                    next_steps: vec![StepTransition {
                        target_step: welcome_step3_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: welcome_step3_id,
                    step_type: StepType::Action(ActionType::SendEmail),
                    config: serde_json::json!({"template": "tips_email"}),
                    position: 2,
                    next_steps: vec![StepTransition {
                        target_step: welcome_step4_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: welcome_step4_id,
                    step_type: StepType::Exit(ExitConfig {
                        reason: "Welcome series complete".to_string(),
                    }),
                    config: serde_json::json!({}),
                    position: 3,
                    next_steps: vec![],
                },
            ],
            created_at: now,
            updated_at: now,
            version: 1,
        };

        // ---- 2. Cart Abandonment (push + email) ----
        let cart_step1_id = Uuid::new_v4();
        let cart_step2_id = Uuid::new_v4();
        let cart_step3_id = Uuid::new_v4();
        let cart_step4_id = Uuid::new_v4();
        let cart_step5_id = Uuid::new_v4();

        let cart_abandonment = Journey {
            id: Uuid::new_v4(),
            name: "Cart Abandonment".to_string(),
            description: "Re-engage users who abandoned their shopping cart".to_string(),
            status: JourneyStatus::Active,
            trigger: JourneyTrigger::EventBased {
                event_type: "cart_abandoned".to_string(),
                filters: serde_json::json!({"min_cart_value": 25}),
            },
            steps: vec![
                JourneyStep {
                    id: cart_step1_id,
                    step_type: StepType::Wait(WaitConfig {
                        duration_secs: 3600,
                        until_event: Some("cart_purchased".to_string()),
                    }),
                    config: serde_json::json!({}),
                    position: 0,
                    next_steps: vec![StepTransition {
                        target_step: cart_step2_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: cart_step2_id,
                    step_type: StepType::Action(ActionType::SendPush),
                    config: serde_json::json!({"template": "cart_reminder_push"}),
                    position: 1,
                    next_steps: vec![StepTransition {
                        target_step: cart_step3_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: cart_step3_id,
                    step_type: StepType::Wait(WaitConfig {
                        duration_secs: 43200,
                        until_event: Some("cart_purchased".to_string()),
                    }),
                    config: serde_json::json!({}),
                    position: 2,
                    next_steps: vec![StepTransition {
                        target_step: cart_step4_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: cart_step4_id,
                    step_type: StepType::Action(ActionType::SendEmail),
                    config: serde_json::json!({"template": "cart_reminder_email"}),
                    position: 3,
                    next_steps: vec![StepTransition {
                        target_step: cart_step5_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: cart_step5_id,
                    step_type: StepType::Exit(ExitConfig {
                        reason: "Cart abandonment flow complete".to_string(),
                    }),
                    config: serde_json::json!({}),
                    position: 4,
                    next_steps: vec![],
                },
            ],
            created_at: now,
            updated_at: now,
            version: 1,
        };

        // ---- 3. Loyalty Re-engagement (multi-channel) ----
        let loyalty_step1_id = Uuid::new_v4();
        let loyalty_step2_id = Uuid::new_v4();
        let loyalty_step3_id = Uuid::new_v4();
        let loyalty_step4_id = Uuid::new_v4();
        let loyalty_step5_id = Uuid::new_v4();
        let loyalty_step6_id = Uuid::new_v4();

        let loyalty_reengagement = Journey {
            id: Uuid::new_v4(),
            name: "Loyalty Re-engagement".to_string(),
            description: "Multi-channel re-engagement for inactive loyalty members".to_string(),
            status: JourneyStatus::Active,
            trigger: JourneyTrigger::SegmentEntry { segment_id: 42 },
            steps: vec![
                JourneyStep {
                    id: loyalty_step1_id,
                    step_type: StepType::Action(ActionType::SendEmail),
                    config: serde_json::json!({"template": "loyalty_winback_email"}),
                    position: 0,
                    next_steps: vec![StepTransition {
                        target_step: loyalty_step2_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: loyalty_step2_id,
                    step_type: StepType::Wait(WaitConfig {
                        duration_secs: 172800,
                        until_event: None,
                    }),
                    config: serde_json::json!({}),
                    position: 1,
                    next_steps: vec![StepTransition {
                        target_step: loyalty_step3_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: loyalty_step3_id,
                    step_type: StepType::Decision(DecisionConfig {
                        branches: vec![
                            DecisionBranch {
                                condition: "email_opened".to_string(),
                                next_step: loyalty_step4_id,
                            },
                            DecisionBranch {
                                condition: "always".to_string(),
                                next_step: loyalty_step5_id,
                            },
                        ],
                    }),
                    config: serde_json::json!({}),
                    position: 2,
                    next_steps: vec![],
                },
                JourneyStep {
                    id: loyalty_step4_id,
                    step_type: StepType::Action(ActionType::SendInApp),
                    config: serde_json::json!({"template": "loyalty_offer_inapp"}),
                    position: 3,
                    next_steps: vec![StepTransition {
                        target_step: loyalty_step6_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: loyalty_step5_id,
                    step_type: StepType::Action(ActionType::SendSms),
                    config: serde_json::json!({"template": "loyalty_offer_sms"}),
                    position: 4,
                    next_steps: vec![StepTransition {
                        target_step: loyalty_step6_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: loyalty_step6_id,
                    step_type: StepType::Exit(ExitConfig {
                        reason: "Loyalty re-engagement complete".to_string(),
                    }),
                    config: serde_json::json!({}),
                    position: 5,
                    next_steps: vec![],
                },
            ],
            created_at: now,
            updated_at: now,
            version: 1,
        };

        let _ = self.create_journey(welcome);
        let _ = self.create_journey(cart_abandonment);
        let _ = self.create_journey(loyalty_reengagement);

        info!("Seeded 3 demo journeys");
    }
}

impl Default for JourneyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_journey() -> Journey {
        let step1_id = Uuid::new_v4();
        let step2_id = Uuid::new_v4();
        let now = Utc::now();

        Journey {
            id: Uuid::new_v4(),
            name: "Test Journey".to_string(),
            description: "A journey for testing".to_string(),
            status: JourneyStatus::Active,
            trigger: JourneyTrigger::ApiBased {
                api_key: "test-key".to_string(),
            },
            steps: vec![
                JourneyStep {
                    id: step1_id,
                    step_type: StepType::Action(ActionType::SendEmail),
                    config: serde_json::json!({"template": "test"}),
                    position: 0,
                    next_steps: vec![StepTransition {
                        target_step: step2_id,
                        condition: None,
                    }],
                },
                JourneyStep {
                    id: step2_id,
                    step_type: StepType::Exit(ExitConfig {
                        reason: "done".to_string(),
                    }),
                    config: serde_json::json!({}),
                    position: 1,
                    next_steps: vec![],
                },
            ],
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    #[test]
    fn test_create_journey() {
        let engine = JourneyEngine::new();
        let journey = make_simple_journey();
        let id = journey.id;

        let result = engine.create_journey(journey);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);

        let fetched = engine.get_journey(&id);
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Test Journey");
    }

    #[test]
    fn test_enter_and_process() {
        let engine = JourneyEngine::new();
        let journey = make_simple_journey();
        let journey_id = journey.id;

        engine.create_journey(journey).unwrap();

        let instance_id = engine.enter_journey(&journey_id, "user-123").unwrap();

        // Process the first step (Action -> SendEmail).
        let result = engine.process_step(&instance_id).unwrap();
        match &result {
            StepResult::ExecuteAction { action_type, .. } => {
                assert!(matches!(action_type, ActionType::SendEmail));
            }
            other => panic!("Expected ExecuteAction, got {:?}", other),
        }

        // Process the second step (Exit).
        let result = engine.process_step(&instance_id).unwrap();
        assert!(matches!(result, StepResult::Complete));

        // Instance should now be completed.
        let inst = engine.instances.get(&instance_id).unwrap();
        assert_eq!(inst.status, InstanceStatus::Completed);
    }

    #[test]
    fn test_suppression_check() {
        let engine = JourneyEngine::new();
        let step_id = Uuid::new_v4();
        let now = Utc::now();

        let journey = Journey {
            id: Uuid::new_v4(),
            name: "Suppression Journey".to_string(),
            description: "Tests bid suppression".to_string(),
            status: JourneyStatus::Active,
            trigger: JourneyTrigger::BidContext {
                campaign_ids: vec!["camp-A".to_string(), "camp-B".to_string()],
            },
            steps: vec![JourneyStep {
                id: step_id,
                step_type: StepType::Action(ActionType::SuppressBid),
                config: serde_json::json!({}),
                position: 0,
                next_steps: vec![],
            }],
            created_at: now,
            updated_at: now,
            version: 1,
        };

        let journey_id = journey.id;
        engine.create_journey(journey).unwrap();
        engine.enter_journey(&journey_id, "user-456").unwrap();

        let suppressed =
            engine.check_suppressions("user-456", &["camp-A".to_string(), "camp-C".to_string()]);

        assert!(suppressed.contains(&"camp-A".to_string()));
        assert!(!suppressed.contains(&"camp-C".to_string()));
    }
}
