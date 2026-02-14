//! Test execution reporter — generates structured reports from agent runs
//! with pass/fail/flaky analysis and trend tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::agent::{AgentRun, AgentState};

/// Overall test result classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestVerdict {
    Passed,
    Failed,
    Flaky,
    Skipped,
}

/// Summary of a single test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: uuid::Uuid,
    pub scenario_name: String,
    pub verdict: TestVerdict,
    pub total_steps: usize,
    pub passed_steps: usize,
    pub failed_steps: usize,
    pub total_assertions: u32,
    pub passed_assertions: u32,
    pub failed_assertions: u32,
    pub duration_ms: u64,
    pub pages_visited: usize,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Aggregate report across multiple runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub title: String,
    pub generated_at: DateTime<Utc>,
    pub summaries: Vec<RunSummary>,
    pub total_runs: usize,
    pub passed_runs: usize,
    pub failed_runs: usize,
    pub flaky_runs: usize,
    pub overall_pass_rate: f64,
    pub total_duration_ms: u64,
    pub slowest_scenario: Option<String>,
    pub fastest_scenario: Option<String>,
    pub failed_assertions_detail: Vec<FailedAssertion>,
}

/// Detail of a failed assertion for debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedAssertion {
    pub scenario: String,
    pub step_name: String,
    pub step_number: u32,
    pub assertion_message: String,
}

/// Test reporter that generates reports from agent runs.
pub struct TestReporter;

impl TestReporter {
    /// Generate a report from a collection of agent runs.
    pub fn generate(title: impl Into<String>, runs: &[AgentRun]) -> TestReport {
        let title = title.into();
        let summaries: Vec<RunSummary> = runs.iter().map(Self::summarize_run).collect();

        let total_runs = summaries.len();
        let passed_runs = summaries
            .iter()
            .filter(|s| s.verdict == TestVerdict::Passed)
            .count();
        let failed_runs = summaries
            .iter()
            .filter(|s| s.verdict == TestVerdict::Failed)
            .count();
        let flaky_runs = summaries
            .iter()
            .filter(|s| s.verdict == TestVerdict::Flaky)
            .count();

        let overall_pass_rate = if total_runs == 0 {
            100.0
        } else {
            passed_runs as f64 / total_runs as f64 * 100.0
        };

        let total_duration_ms: u64 = summaries.iter().map(|s| s.duration_ms).sum();

        let slowest_scenario = summaries
            .iter()
            .max_by_key(|s| s.duration_ms)
            .map(|s| s.scenario_name.clone());

        let fastest_scenario = summaries
            .iter()
            .min_by_key(|s| s.duration_ms)
            .map(|s| s.scenario_name.clone());

        let failed_assertions_detail = Self::collect_failures(runs);

        TestReport {
            title,
            generated_at: Utc::now(),
            summaries,
            total_runs,
            passed_runs,
            failed_runs,
            flaky_runs,
            overall_pass_rate,
            total_duration_ms,
            slowest_scenario,
            fastest_scenario,
            failed_assertions_detail,
        }
    }

    fn summarize_run(run: &AgentRun) -> RunSummary {
        let failed_steps = run.steps.iter().filter(|s| !s.result.success).count();
        let passed_steps = run.steps.len() - failed_steps;

        let verdict = match run.state {
            AgentState::Completed if run.failed_assertions == 0 => TestVerdict::Passed,
            AgentState::Completed => TestVerdict::Failed,
            AgentState::Failed => TestVerdict::Failed,
            _ => TestVerdict::Skipped,
        };

        RunSummary {
            run_id: run.id,
            scenario_name: run.scenario.clone().unwrap_or_else(|| "unnamed".into()),
            verdict,
            total_steps: run.steps.len(),
            passed_steps,
            failed_steps,
            total_assertions: run.total_assertions,
            passed_assertions: run.passed_assertions,
            failed_assertions: run.failed_assertions,
            duration_ms: run.duration_ms,
            pages_visited: run.pages_visited.len(),
            error: run.error.clone(),
            started_at: run.started_at,
            completed_at: run.completed_at,
        }
    }

    fn collect_failures(runs: &[AgentRun]) -> Vec<FailedAssertion> {
        let mut failures = Vec::new();
        for run in runs {
            let scenario_name = run.scenario.clone().unwrap_or_else(|| "unnamed".into());
            for step in &run.steps {
                for assertion in &step.assertions {
                    if !assertion.passed {
                        failures.push(FailedAssertion {
                            scenario: scenario_name.clone(),
                            step_name: format!("Step {}", step.step_number),
                            step_number: step.step_number,
                            assertion_message: assertion.message.clone(),
                        });
                    }
                }
            }
        }
        failures
    }

    /// Render report as a formatted text table.
    pub fn render_text(report: &TestReport) -> String {
        let mut out = String::new();
        out.push_str(&format!("=== {} ===\n", report.title));
        out.push_str(&format!(
            "Generated: {}\n\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        out.push_str(&format!(
            "Total: {} | Passed: {} | Failed: {} | Flaky: {} | Pass Rate: {:.1}%\n",
            report.total_runs,
            report.passed_runs,
            report.failed_runs,
            report.flaky_runs,
            report.overall_pass_rate,
        ));
        out.push_str(&format!(
            "Total Duration: {}ms\n\n",
            report.total_duration_ms
        ));

        out.push_str(&format!(
            "  {:<30} {:<10} {:<8} {:<8} {:<10}\n",
            "Scenario", "Verdict", "Steps", "Assert", "Duration"
        ));
        out.push_str(&format!("  {}\n", "-".repeat(70)));

        for s in &report.summaries {
            let verdict = match s.verdict {
                TestVerdict::Passed => "PASS",
                TestVerdict::Failed => "FAIL",
                TestVerdict::Flaky => "FLAKY",
                TestVerdict::Skipped => "SKIP",
            };
            out.push_str(&format!(
                "  {:<30} {:<10} {:<8} {}/{:<5} {}ms\n",
                s.scenario_name,
                verdict,
                s.total_steps,
                s.passed_assertions,
                s.total_assertions,
                s.duration_ms,
            ));
        }

        if !report.failed_assertions_detail.is_empty() {
            out.push_str("\nFailed Assertions:\n");
            for f in &report.failed_assertions_detail {
                out.push_str(&format!(
                    "  [{}] {} - {}\n",
                    f.scenario, f.step_name, f.assertion_message
                ));
            }
        }

        out
    }
}

/// Detect flaky tests by running the same scenario multiple times.
pub fn detect_flaky(runs: &[AgentRun]) -> Vec<String> {
    use std::collections::HashMap;
    let mut results_by_scenario: HashMap<String, Vec<bool>> = HashMap::new();

    for run in runs {
        let name = run.scenario.clone().unwrap_or_else(|| "unnamed".into());
        let passed = run.state == AgentState::Completed && run.failed_assertions == 0;
        results_by_scenario.entry(name).or_default().push(passed);
    }

    results_by_scenario
        .into_iter()
        .filter(|(_, results)| {
            results.len() > 1 && results.iter().any(|r| *r) && results.iter().any(|r| !*r)
        })
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentConfig, AgentState};
    use crate::page_objects::PageId;

    fn make_run(name: &str, state: AgentState, passed: u32, failed: u32) -> AgentRun {
        AgentRun {
            id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            config: AgentConfig::default(),
            scenario: Some(name.into()),
            state,
            steps: vec![],
            pages_visited: vec![PageId::Dashboard],
            total_assertions: passed + failed,
            passed_assertions: passed,
            failed_assertions: failed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            duration_ms: 100,
            error: None,
        }
    }

    #[test]
    fn test_report_generation() {
        let runs = vec![
            make_run("Login Flow", AgentState::Completed, 3, 0),
            make_run("Dashboard", AgentState::Completed, 5, 0),
            make_run("Billing", AgentState::Failed, 2, 1),
        ];

        let report = TestReporter::generate("Test Suite", &runs);
        assert_eq!(report.total_runs, 3);
        assert_eq!(report.passed_runs, 2);
        assert_eq!(report.failed_runs, 1);
        assert!((report.overall_pass_rate - 66.666).abs() < 1.0);
    }

    #[test]
    fn test_empty_report() {
        let report = TestReporter::generate("Empty", &[]);
        assert_eq!(report.total_runs, 0);
        assert_eq!(report.overall_pass_rate, 100.0);
    }

    #[test]
    fn test_text_render() {
        let runs = vec![
            make_run("Login", AgentState::Completed, 2, 0),
            make_run("API", AgentState::Failed, 1, 1),
        ];
        let report = TestReporter::generate("Smoke Tests", &runs);
        let text = TestReporter::render_text(&report);
        assert!(text.contains("Smoke Tests"));
        assert!(text.contains("PASS"));
        assert!(text.contains("FAIL"));
    }

    #[test]
    fn test_flaky_detection() {
        let runs = vec![
            make_run("Flaky Test", AgentState::Completed, 3, 0),
            make_run("Flaky Test", AgentState::Failed, 1, 2),
            make_run("Stable Test", AgentState::Completed, 5, 0),
            make_run("Stable Test", AgentState::Completed, 5, 0),
        ];

        let flaky = detect_flaky(&runs);
        assert_eq!(flaky.len(), 1);
        assert!(flaky.contains(&"Flaky Test".to_string()));
    }
}
