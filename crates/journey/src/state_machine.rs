use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::types::InstanceStatus;

/// Describes a single valid state transition for a journey instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: InstanceStatus,
    pub to: InstanceStatus,
    pub trigger: String,
}

/// Guards journey-instance lifecycle by enforcing a finite set of valid
/// state transitions.
#[derive(Debug, Clone)]
pub struct JourneyStateMachine {
    pub state: InstanceStatus,
    pub transitions: Vec<StateTransition>,
}

impl JourneyStateMachine {
    /// Creates a new state machine starting in `Active` with all valid
    /// transitions pre-configured.
    pub fn new() -> Self {
        let transitions = vec![
            // Active ->
            StateTransition {
                from: InstanceStatus::Active,
                to: InstanceStatus::Waiting,
                trigger: "wait_step".to_string(),
            },
            StateTransition {
                from: InstanceStatus::Active,
                to: InstanceStatus::Completed,
                trigger: "journey_complete".to_string(),
            },
            StateTransition {
                from: InstanceStatus::Active,
                to: InstanceStatus::Exited,
                trigger: "exit_step".to_string(),
            },
            StateTransition {
                from: InstanceStatus::Active,
                to: InstanceStatus::Error,
                trigger: "processing_error".to_string(),
            },
            // Waiting ->
            StateTransition {
                from: InstanceStatus::Waiting,
                to: InstanceStatus::Active,
                trigger: "wait_complete".to_string(),
            },
            StateTransition {
                from: InstanceStatus::Waiting,
                to: InstanceStatus::Exited,
                trigger: "exit_while_waiting".to_string(),
            },
            StateTransition {
                from: InstanceStatus::Waiting,
                to: InstanceStatus::Error,
                trigger: "wait_error".to_string(),
            },
            // Error ->
            StateTransition {
                from: InstanceStatus::Error,
                to: InstanceStatus::Active,
                trigger: "retry".to_string(),
            },
            StateTransition {
                from: InstanceStatus::Error,
                to: InstanceStatus::Exited,
                trigger: "abandon".to_string(),
            },
        ];

        Self {
            state: InstanceStatus::Active,
            transitions,
        }
    }

    /// Returns `true` if the given transition is allowed.
    pub fn can_transition(&self, from: &InstanceStatus, to: &InstanceStatus) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == *from && t.to == *to)
    }

    /// Attempts to move the state machine to `to`. Returns an error if the
    /// transition is not permitted.
    pub fn transition(&mut self, to: InstanceStatus) -> Result<()> {
        if self.can_transition(&self.state, &to) {
            self.state = to;
            Ok(())
        } else {
            Err(anyhow!(
                "Invalid state transition from {:?} to {:?}",
                self.state,
                to
            ))
        }
    }
}

impl Default for JourneyStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
