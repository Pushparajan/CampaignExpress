use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A journey definition describing a multi-step user experience flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journey {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: JourneyStatus,
    pub trigger: JourneyTrigger,
    pub steps: Vec<JourneyStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

/// Lifecycle status of a journey definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JourneyStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
}

/// What triggers a user's entry into a journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum JourneyTrigger {
    EventBased {
        event_type: String,
        filters: serde_json::Value,
    },
    SegmentEntry {
        segment_id: u32,
    },
    ScheduleBased {
        cron_expression: String,
    },
    ApiBased {
        api_key: String,
    },
    BidContext {
        campaign_ids: Vec<String>,
    },
}

/// A single step within a journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    pub id: Uuid,
    pub step_type: StepType,
    pub config: serde_json::Value,
    pub position: u32,
    pub next_steps: Vec<StepTransition>,
}

/// The kind of work a step performs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StepType {
    Action(ActionType),
    Wait(WaitConfig),
    Decision(DecisionConfig),
    Split(SplitConfig),
    Exit(ExitConfig),
}

/// Concrete action types that can be executed in an Action step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    SendPush,
    SendEmail,
    SendSms,
    SendInApp,
    SendWebhook,
    SuppressBid,
    UpdateProfile,
    AddToSegment,
    RemoveFromSegment,
    TriggerCampaign,
}

/// Configuration for a wait step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitConfig {
    pub duration_secs: u64,
    pub until_event: Option<String>,
}

/// Configuration for a decision (branching) step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConfig {
    pub branches: Vec<DecisionBranch>,
}

/// A single branch inside a decision step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionBranch {
    pub condition: String,
    pub next_step: Uuid,
}

/// Configuration for an A/B or multi-variant split step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitConfig {
    pub variants: Vec<SplitVariant>,
    pub split_type: SplitType,
}

/// How traffic is distributed across split variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitType {
    Random,
    Deterministic,
}

/// A single variant in a split step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitVariant {
    pub name: String,
    pub weight: f32,
    pub next_step: Uuid,
}

/// A directed edge between two journey steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTransition {
    pub target_step: Uuid,
    pub condition: Option<String>,
}

/// Configuration for an exit step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitConfig {
    pub reason: String,
}

/// A concrete instance of a user progressing through a journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyInstance {
    pub id: Uuid,
    pub journey_id: Uuid,
    pub user_id: String,
    pub current_step_id: Uuid,
    pub status: InstanceStatus,
    pub context: serde_json::Value,
    pub entered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub step_history: Vec<StepExecution>,
}

/// Runtime status of a journey instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    Active,
    Waiting,
    Completed,
    Exited,
    Error,
}

/// Record of a step that has been executed for an instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    pub step_id: Uuid,
    pub step_type: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: serde_json::Value,
}

/// Aggregate statistics for a journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStats {
    pub journey_id: Uuid,
    pub total_entered: u64,
    pub active: u64,
    pub completed: u64,
    pub exited: u64,
    pub error: u64,
    pub avg_completion_time_secs: f64,
    pub step_conversion_rates: HashMap<String, f64>,
}
