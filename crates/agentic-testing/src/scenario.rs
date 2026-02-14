//! Test scenario engine — declarative test plans that agents execute step by
//! step, with built-in scenarios for common UI flows.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::actions::{HttpMethod, TestAction};
use crate::assertions::Assertion;
use crate::page_objects::PageId;

/// A test scenario — an ordered sequence of steps to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub steps: Vec<ScenarioStep>,
    pub setup: Vec<TestAction>,
    pub teardown: Vec<TestAction>,
}

/// A single step within a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStep {
    pub name: String,
    pub action: TestAction,
    pub assertions: Vec<Assertion>,
    pub timeout_ms: u64,
}

/// A suite of scenarios grouped by feature area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub scenarios: Vec<TestScenario>,
}

/// Build the default smoke test suite covering all pages.
pub fn smoke_test_suite() -> TestSuite {
    TestSuite {
        name: "Smoke Tests".into(),
        description: "Basic navigation and page load verification".into(),
        scenarios: vec![
            login_scenario(),
            dashboard_scenario(),
            campaigns_crud_scenario(),
            billing_scenario(),
            platform_admin_scenario(),
        ],
    }
}

/// Login flow scenario.
pub fn login_scenario() -> TestScenario {
    TestScenario {
        id: Uuid::new_v4(),
        name: "Login Flow".into(),
        description: "Authenticate and verify session".into(),
        tags: vec!["auth".into(), "smoke".into()],
        steps: vec![
            ScenarioStep {
                name: "Navigate to login page".into(),
                action: TestAction::Navigate {
                    page: PageId::Login,
                    url: "/login".into(),
                },
                assertions: vec![],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "Enter email".into(),
                action: TestAction::TypeText {
                    selector: "input[type='email']".into(),
                    text: "admin@example.com".into(),
                    clear_first: true,
                },
                assertions: vec![],
                timeout_ms: 2000,
            },
            ScenarioStep {
                name: "Enter password".into(),
                action: TestAction::TypeText {
                    selector: "input[type='password']".into(),
                    text: "campaign2024".into(),
                    clear_first: true,
                },
                assertions: vec![],
                timeout_ms: 2000,
            },
            ScenarioStep {
                name: "Click login button".into(),
                action: TestAction::SubmitForm {
                    selector: "form".into(),
                },
                assertions: vec![Assertion::StatusCode(200)],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "Verify redirect to dashboard".into(),
                action: TestAction::WaitForElement {
                    selector: ".grid".into(),
                    timeout_ms: 5000,
                },
                assertions: vec![],
                timeout_ms: 5000,
            },
        ],
        setup: vec![],
        teardown: vec![],
    }
}

/// Dashboard page scenario.
pub fn dashboard_scenario() -> TestScenario {
    TestScenario {
        id: Uuid::new_v4(),
        name: "Dashboard Overview".into(),
        description: "Verify dashboard loads with stats and charts".into(),
        tags: vec!["dashboard".into(), "smoke".into()],
        steps: vec![
            ScenarioStep {
                name: "Navigate to dashboard".into(),
                action: TestAction::Navigate {
                    page: PageId::Dashboard,
                    url: "/".into(),
                },
                assertions: vec![],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "Verify stats cards visible".into(),
                action: TestAction::AssertVisible {
                    selector: ".grid".into(),
                },
                assertions: vec![Assertion::ElementVisible(".grid".into())],
                timeout_ms: 3000,
            },
            ScenarioStep {
                name: "Check monitoring API".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Get,
                    path: "/api/v1/management/monitoring/overview".into(),
                    body: None,
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![
                    Assertion::StatusCode(200),
                    Assertion::ResponseContains("total_campaigns".into()),
                ],
                timeout_ms: 5000,
            },
        ],
        setup: vec![],
        teardown: vec![],
    }
}

/// Campaign CRUD scenario.
pub fn campaigns_crud_scenario() -> TestScenario {
    TestScenario {
        id: Uuid::new_v4(),
        name: "Campaign CRUD".into(),
        description: "Create, read, update, delete a campaign".into(),
        tags: vec!["campaigns".into(), "crud".into()],
        steps: vec![
            ScenarioStep {
                name: "Navigate to campaigns".into(),
                action: TestAction::Navigate {
                    page: PageId::Campaigns,
                    url: "/campaigns".into(),
                },
                assertions: vec![],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "List campaigns via API".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Get,
                    path: "/api/v1/management/campaigns".into(),
                    body: None,
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![Assertion::StatusCode(200), Assertion::ResponseIsArray],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "Create campaign via API".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Post,
                    path: "/api/v1/management/campaigns".into(),
                    body: Some(serde_json::json!({
                        "name": "Agent Test Campaign",
                        "budget": 5000.0,
                        "start_date": "2026-03-01",
                        "end_date": "2026-03-31"
                    })),
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![Assertion::StatusCode(201)],
                timeout_ms: 5000,
            },
        ],
        setup: vec![],
        teardown: vec![],
    }
}

/// Billing page scenario.
pub fn billing_scenario() -> TestScenario {
    TestScenario {
        id: Uuid::new_v4(),
        name: "Billing Dashboard".into(),
        description: "Verify billing plans, invoices, and usage".into(),
        tags: vec!["billing".into(), "smoke".into()],
        steps: vec![
            ScenarioStep {
                name: "Navigate to billing".into(),
                action: TestAction::Navigate {
                    page: PageId::Billing,
                    url: "/billing".into(),
                },
                assertions: vec![],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "Fetch plans".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Get,
                    path: "/api/v1/management/billing/plans".into(),
                    body: None,
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![Assertion::StatusCode(200), Assertion::ResponseIsArray],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "Fetch invoices".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Get,
                    path: "/api/v1/management/billing/invoices".into(),
                    body: None,
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![Assertion::StatusCode(200)],
                timeout_ms: 5000,
            },
        ],
        setup: vec![],
        teardown: vec![],
    }
}

/// Platform admin scenario.
pub fn platform_admin_scenario() -> TestScenario {
    TestScenario {
        id: Uuid::new_v4(),
        name: "Platform Administration".into(),
        description: "Verify tenant management, roles, and compliance".into(),
        tags: vec!["platform".into(), "admin".into()],
        steps: vec![
            ScenarioStep {
                name: "Navigate to platform".into(),
                action: TestAction::Navigate {
                    page: PageId::Platform,
                    url: "/platform".into(),
                },
                assertions: vec![],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "List tenants".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Get,
                    path: "/api/v1/management/platform/tenants".into(),
                    body: None,
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![Assertion::StatusCode(200), Assertion::ResponseIsArray],
                timeout_ms: 5000,
            },
            ScenarioStep {
                name: "List roles".into(),
                action: TestAction::ApiCall {
                    method: HttpMethod::Get,
                    path: "/api/v1/management/platform/roles".into(),
                    body: None,
                    headers: std::collections::HashMap::new(),
                },
                assertions: vec![Assertion::StatusCode(200), Assertion::ResponseIsArray],
                timeout_ms: 5000,
            },
        ],
        setup: vec![],
        teardown: vec![],
    }
}

/// Generate an exploratory scenario that visits all pages.
pub fn exploratory_all_pages() -> TestScenario {
    let steps: Vec<ScenarioStep> = PageId::all_pages()
        .iter()
        .map(|page| ScenarioStep {
            name: format!("Navigate to {}", page.title()),
            action: TestAction::Navigate {
                page: *page,
                url: page.path().to_string(),
            },
            assertions: vec![],
            timeout_ms: 5000,
        })
        .collect();

    TestScenario {
        id: Uuid::new_v4(),
        name: "Exploratory - All Pages".into(),
        description: "Visit every page to verify basic rendering".into(),
        tags: vec!["exploratory".into(), "smoke".into()],
        steps,
        setup: vec![],
        teardown: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke_suite() {
        let suite = smoke_test_suite();
        assert_eq!(suite.name, "Smoke Tests");
        assert_eq!(suite.scenarios.len(), 5);

        let login = &suite.scenarios[0];
        assert_eq!(login.name, "Login Flow");
        assert_eq!(login.steps.len(), 5);
        assert!(login.tags.contains(&"auth".into()));
    }

    #[test]
    fn test_exploratory_scenario() {
        let scenario = exploratory_all_pages();
        assert_eq!(scenario.steps.len(), 10);
        assert!(scenario.tags.contains(&"exploratory".into()));
    }

    #[test]
    fn test_scenario_serde() {
        let scenario = login_scenario();
        let json = serde_json::to_string(&scenario).unwrap();
        let parsed: TestScenario = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Login Flow");
        assert_eq!(parsed.steps.len(), 5);
    }
}
