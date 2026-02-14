//! Test agent framework — autonomous AI-driven agents that explore, interact
//! with, and validate UI pages and API endpoints.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::actions::{TestAction, TestActionResult};
use crate::assertions::AssertionResult;
use crate::page_objects::PageId;
use crate::scenario::TestScenario;

/// Agent behavioural strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStrategy {
    /// Execute a scripted test scenario step-by-step.
    Scripted,
    /// Explore the UI autonomously, discovering pages and interactions.
    Exploratory,
    /// Fuzz inputs to find edge cases and crashes.
    Fuzzing,
    /// Regression — re-run previously captured interaction sequences.
    Regression,
    /// Accessibility — validate WCAG compliance across pages.
    Accessibility,
}

/// Agent lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

/// Configuration for a test agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub strategy: AgentStrategy,
    pub max_steps: u32,
    pub timeout_seconds: u64,
    pub base_url: String,
    pub auth_token: Option<String>,
    pub record_interactions: bool,
    pub screenshot_on_failure: bool,
    pub retry_flaky: bool,
    pub max_retries: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "default-agent".into(),
            strategy: AgentStrategy::Scripted,
            max_steps: 100,
            timeout_seconds: 300,
            base_url: "http://localhost:8080".into(),
            auth_token: None,
            record_interactions: true,
            screenshot_on_failure: true,
            retry_flaky: true,
            max_retries: 3,
        }
    }
}

/// A single step executed by the agent during a test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub step_number: u32,
    pub action: TestAction,
    pub result: TestActionResult,
    pub assertions: Vec<AssertionResult>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Record of a complete agent test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub config: AgentConfig,
    pub scenario: Option<String>,
    pub state: AgentState,
    pub steps: Vec<AgentStep>,
    pub pages_visited: Vec<PageId>,
    pub total_assertions: u32,
    pub passed_assertions: u32,
    pub failed_assertions: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// The test agent — drives UI interactions and validates behaviour.
pub struct TestAgent {
    pub id: Uuid,
    pub config: AgentConfig,
    pub state: AgentState,
    runs: Vec<AgentRun>,
    action_handler: Arc<dyn ActionHandler>,
}

/// Trait for executing actions — implementations can target headless browsers,
/// HTTP APIs, or mock simulators.
pub trait ActionHandler: Send + Sync {
    fn execute(&self, action: &TestAction) -> TestActionResult;
}

/// Default mock handler that simulates successful actions.
pub struct MockActionHandler;

impl ActionHandler for MockActionHandler {
    fn execute(&self, action: &TestAction) -> TestActionResult {
        TestActionResult {
            success: true,
            action: action.clone(),
            response_body: Some(serde_json::json!({"mock": true})),
            status_code: Some(200),
            duration_ms: 15,
            error: None,
            timestamp: Utc::now(),
        }
    }
}

impl TestAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            state: AgentState::Idle,
            runs: Vec::new(),
            action_handler: Arc::new(MockActionHandler),
        }
    }

    pub fn with_handler(mut self, handler: Arc<dyn ActionHandler>) -> Self {
        self.action_handler = handler;
        self
    }

    /// Execute a test scenario and return the run result.
    pub fn execute_scenario(&mut self, scenario: &TestScenario) -> AgentRun {
        let started_at = Utc::now();
        self.state = AgentState::Running;

        info!(
            agent_id = %self.id,
            scenario = %scenario.name,
            steps = scenario.steps.len(),
            "Agent starting scenario"
        );

        let mut steps = Vec::new();
        let mut pages_visited = Vec::new();
        let mut total_assertions = 0u32;
        let mut passed_assertions = 0u32;
        let mut failed_assertions = 0u32;
        let mut error = None;

        for (i, scenario_step) in scenario.steps.iter().enumerate() {
            let step_start = Utc::now();

            // Track page navigation
            if let TestAction::Navigate { page, .. } = &scenario_step.action {
                if !pages_visited.contains(page) {
                    pages_visited.push(*page);
                }
            }

            let result = self.action_handler.execute(&scenario_step.action);

            // Evaluate assertions
            let mut assertion_results = Vec::new();
            for assertion in &scenario_step.assertions {
                let ar = assertion.evaluate(&result);
                total_assertions += 1;
                if ar.passed {
                    passed_assertions += 1;
                } else {
                    failed_assertions += 1;
                }
                assertion_results.push(ar);
            }

            let step_duration = (Utc::now() - step_start).num_milliseconds().max(0) as u64;

            let step = AgentStep {
                step_number: (i + 1) as u32,
                action: scenario_step.action.clone(),
                result: result.clone(),
                assertions: assertion_results.clone(),
                duration_ms: step_duration,
                timestamp: step_start,
            };

            steps.push(step);

            if !result.success {
                error = result.error.clone();
                if !self.config.retry_flaky {
                    break;
                }
            }

            // Check step limit
            if i as u32 >= self.config.max_steps {
                error = Some("Max steps exceeded".into());
                break;
            }
        }

        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;

        let final_state = if failed_assertions > 0 || error.is_some() {
            AgentState::Failed
        } else {
            AgentState::Completed
        };

        self.state = final_state;

        let run = AgentRun {
            id: Uuid::new_v4(),
            agent_id: self.id,
            config: self.config.clone(),
            scenario: Some(scenario.name.clone()),
            state: final_state,
            steps,
            pages_visited,
            total_assertions,
            passed_assertions,
            failed_assertions,
            started_at,
            completed_at: Some(completed_at),
            duration_ms,
            error,
        };

        self.runs.push(run.clone());
        info!(
            agent_id = %self.id,
            passed = passed_assertions,
            failed = failed_assertions,
            duration_ms,
            "Agent scenario completed"
        );
        run
    }

    /// Run autonomous exploration starting from the dashboard.
    pub fn explore(&mut self, max_pages: usize) -> AgentRun {
        let started_at = Utc::now();
        self.state = AgentState::Running;

        let pages_to_visit = vec![
            PageId::Dashboard,
            PageId::Campaigns,
            PageId::Creatives,
            PageId::Journeys,
            PageId::Dco,
            PageId::Cdp,
            PageId::Experiments,
            PageId::Platform,
            PageId::Billing,
            PageId::Ops,
        ];

        let mut steps = Vec::new();
        let mut pages_visited = Vec::new();
        let mut total_assertions = 0u32;
        let mut passed_assertions = 0u32;

        for page in pages_to_visit.iter().take(max_pages) {
            let action = TestAction::Navigate {
                page: *page,
                url: page.path().to_string(),
            };

            let result = self.action_handler.execute(&action);

            total_assertions += 1;
            if result.success {
                passed_assertions += 1;
            }
            pages_visited.push(*page);

            steps.push(AgentStep {
                step_number: steps.len() as u32 + 1,
                action,
                result,
                assertions: Vec::new(),
                duration_ms: 20,
                timestamp: Utc::now(),
            });
        }

        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;
        self.state = AgentState::Completed;

        let run = AgentRun {
            id: Uuid::new_v4(),
            agent_id: self.id,
            config: self.config.clone(),
            scenario: Some("exploratory".into()),
            state: AgentState::Completed,
            steps,
            pages_visited,
            total_assertions,
            passed_assertions,
            failed_assertions: total_assertions - passed_assertions,
            started_at,
            completed_at: Some(completed_at),
            duration_ms,
            error: None,
        };

        self.runs.push(run.clone());
        run
    }

    /// Get all completed runs.
    pub fn runs(&self) -> &[AgentRun] {
        &self.runs
    }

    /// Get pass rate across all runs.
    pub fn pass_rate(&self) -> f64 {
        let total: u32 = self.runs.iter().map(|r| r.total_assertions).sum();
        let passed: u32 = self.runs.iter().map(|r| r.passed_assertions).sum();
        if total == 0 {
            100.0
        } else {
            passed as f64 / total as f64 * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{ScenarioStep, TestScenario};

    #[test]
    fn test_agent_creation() {
        let agent = TestAgent::new(AgentConfig::default());
        assert_eq!(agent.state, AgentState::Idle);
        assert!(agent.runs().is_empty());
    }

    #[test]
    fn test_agent_explore() {
        let mut agent = TestAgent::new(AgentConfig {
            strategy: AgentStrategy::Exploratory,
            ..Default::default()
        });

        let run = agent.explore(5);
        assert_eq!(run.state, AgentState::Completed);
        assert_eq!(run.pages_visited.len(), 5);
        assert_eq!(run.total_assertions, 5);
        assert_eq!(run.passed_assertions, 5);
    }

    #[test]
    fn test_agent_scenario() {
        let mut agent = TestAgent::new(AgentConfig::default());
        let scenario = TestScenario {
            id: Uuid::new_v4(),
            name: "basic_nav".into(),
            description: "Navigate to dashboard".into(),
            tags: vec!["smoke".into()],
            steps: vec![ScenarioStep {
                name: "Go to dashboard".into(),
                action: TestAction::Navigate {
                    page: PageId::Dashboard,
                    url: "/".into(),
                },
                assertions: vec![],
                timeout_ms: 5000,
            }],
            setup: vec![],
            teardown: vec![],
        };

        let run = agent.execute_scenario(&scenario);
        assert_eq!(run.state, AgentState::Completed);
        assert_eq!(run.steps.len(), 1);
        assert_eq!(agent.pass_rate(), 100.0);
    }
}
