//! Page object models — typed representations of UI pages with their key
//! elements, expected content, and interaction targets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Page identifier matching the UI's route structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageId {
    Login,
    Dashboard,
    Campaigns,
    Creatives,
    Journeys,
    Dco,
    Cdp,
    Experiments,
    Platform,
    Billing,
    Ops,
    Settings,
    Monitoring,
}

impl PageId {
    /// URL path for this page.
    pub fn path(&self) -> &str {
        match self {
            PageId::Login => "/login",
            PageId::Dashboard => "/",
            PageId::Campaigns => "/campaigns",
            PageId::Creatives => "/creatives",
            PageId::Journeys => "/journeys",
            PageId::Dco => "/dco",
            PageId::Cdp => "/cdp",
            PageId::Experiments => "/experiments",
            PageId::Platform => "/platform",
            PageId::Billing => "/billing",
            PageId::Ops => "/ops",
            PageId::Settings => "/settings",
            PageId::Monitoring => "/monitoring",
        }
    }

    /// Human-readable page title.
    pub fn title(&self) -> &str {
        match self {
            PageId::Login => "Login",
            PageId::Dashboard => "Dashboard",
            PageId::Campaigns => "Campaigns",
            PageId::Creatives => "Creatives",
            PageId::Journeys => "Journey Orchestration",
            PageId::Dco => "Dynamic Creative Optimization",
            PageId::Cdp => "Customer Data Platform",
            PageId::Experiments => "Experiments",
            PageId::Platform => "Platform Administration",
            PageId::Billing => "Billing & Plans",
            PageId::Ops => "Operations",
            PageId::Settings => "Settings",
            PageId::Monitoring => "Monitoring",
        }
    }

    /// All navigable pages (excluding login).
    pub fn all_pages() -> Vec<PageId> {
        vec![
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
        ]
    }
}

/// An element on a page that can be interacted with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageElement {
    pub name: String,
    pub selector: String,
    pub element_type: ElementType,
    pub required: bool,
}

/// Type of interactive element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    Button,
    Link,
    Input,
    Select,
    Table,
    Card,
    Tab,
    Modal,
    Form,
    Badge,
    Chart,
    Text,
}

/// Full page object model describing a UI page's structure and interactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageObject {
    pub page: PageId,
    pub elements: Vec<PageElement>,
    pub api_endpoints: Vec<String>,
    pub expected_stats_cards: Vec<String>,
    pub expected_tables: Vec<String>,
}

/// Build the page object catalog for all Campaign Express UI pages.
pub fn build_page_catalog() -> HashMap<PageId, PageObject> {
    let mut catalog = HashMap::new();

    catalog.insert(
        PageId::Login,
        PageObject {
            page: PageId::Login,
            elements: vec![
                PageElement {
                    name: "email_input".into(),
                    selector: "input[type='email']".into(),
                    element_type: ElementType::Input,
                    required: true,
                },
                PageElement {
                    name: "password_input".into(),
                    selector: "input[type='password']".into(),
                    element_type: ElementType::Input,
                    required: true,
                },
                PageElement {
                    name: "login_button".into(),
                    selector: "button[type='submit']".into(),
                    element_type: ElementType::Button,
                    required: true,
                },
            ],
            api_endpoints: vec!["POST /api/v1/management/auth/login".into()],
            expected_stats_cards: vec![],
            expected_tables: vec![],
        },
    );

    catalog.insert(
        PageId::Dashboard,
        PageObject {
            page: PageId::Dashboard,
            elements: vec![
                PageElement {
                    name: "stats_grid".into(),
                    selector: ".grid".into(),
                    element_type: ElementType::Card,
                    required: true,
                },
                PageElement {
                    name: "performance_chart".into(),
                    selector: "[data-testid='performance-chart']".into(),
                    element_type: ElementType::Chart,
                    required: false,
                },
            ],
            api_endpoints: vec!["GET /api/v1/management/monitoring/overview".into()],
            expected_stats_cards: vec![
                "Total Campaigns".into(),
                "Total Offers".into(),
                "Avg Latency".into(),
                "Active Users".into(),
            ],
            expected_tables: vec![],
        },
    );

    catalog.insert(
        PageId::Campaigns,
        PageObject {
            page: PageId::Campaigns,
            elements: vec![
                PageElement {
                    name: "create_button".into(),
                    selector: "button:has-text('Create Campaign')".into(),
                    element_type: ElementType::Button,
                    required: true,
                },
                PageElement {
                    name: "campaigns_table".into(),
                    selector: "table".into(),
                    element_type: ElementType::Table,
                    required: true,
                },
                PageElement {
                    name: "search_input".into(),
                    selector: "input[placeholder*='Search']".into(),
                    element_type: ElementType::Input,
                    required: false,
                },
            ],
            api_endpoints: vec![
                "GET /api/v1/management/campaigns".into(),
                "POST /api/v1/management/campaigns".into(),
            ],
            expected_stats_cards: vec![],
            expected_tables: vec!["campaigns".into()],
        },
    );

    catalog.insert(
        PageId::Billing,
        PageObject {
            page: PageId::Billing,
            elements: vec![
                PageElement {
                    name: "plans_tab".into(),
                    selector: "[data-tab='plans']".into(),
                    element_type: ElementType::Tab,
                    required: true,
                },
                PageElement {
                    name: "invoices_tab".into(),
                    selector: "[data-tab='invoices']".into(),
                    element_type: ElementType::Tab,
                    required: true,
                },
                PageElement {
                    name: "usage_tab".into(),
                    selector: "[data-tab='usage']".into(),
                    element_type: ElementType::Tab,
                    required: true,
                },
            ],
            api_endpoints: vec![
                "GET /api/v1/management/billing/plans".into(),
                "GET /api/v1/management/billing/subscription".into(),
                "GET /api/v1/management/billing/invoices".into(),
                "GET /api/v1/management/billing/usage".into(),
            ],
            expected_stats_cards: vec![],
            expected_tables: vec!["plans".into(), "invoices".into()],
        },
    );

    catalog.insert(
        PageId::Platform,
        PageObject {
            page: PageId::Platform,
            elements: vec![
                PageElement {
                    name: "tenants_tab".into(),
                    selector: "[data-tab='tenants']".into(),
                    element_type: ElementType::Tab,
                    required: true,
                },
                PageElement {
                    name: "roles_tab".into(),
                    selector: "[data-tab='roles']".into(),
                    element_type: ElementType::Tab,
                    required: true,
                },
            ],
            api_endpoints: vec![
                "GET /api/v1/management/platform/tenants".into(),
                "GET /api/v1/management/platform/roles".into(),
                "GET /api/v1/management/platform/compliance".into(),
            ],
            expected_stats_cards: vec![],
            expected_tables: vec!["tenants".into(), "roles".into()],
        },
    );

    // Add remaining pages with minimal definitions
    for page in &[
        PageId::Creatives,
        PageId::Journeys,
        PageId::Dco,
        PageId::Cdp,
        PageId::Experiments,
        PageId::Ops,
    ] {
        catalog.insert(
            *page,
            PageObject {
                page: *page,
                elements: vec![PageElement {
                    name: "main_content".into(),
                    selector: "main".into(),
                    element_type: ElementType::Card,
                    required: true,
                }],
                api_endpoints: vec![format!(
                    "GET /api/v1/management/{}",
                    page.path().trim_start_matches('/')
                )],
                expected_stats_cards: vec![],
                expected_tables: vec![],
            },
        );
    }

    catalog
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_paths() {
        assert_eq!(PageId::Dashboard.path(), "/");
        assert_eq!(PageId::Campaigns.path(), "/campaigns");
        assert_eq!(PageId::Login.path(), "/login");
    }

    #[test]
    fn test_all_pages() {
        let pages = PageId::all_pages();
        assert_eq!(pages.len(), 10);
        assert!(!pages.contains(&PageId::Login));
    }

    #[test]
    fn test_page_catalog() {
        let catalog = build_page_catalog();
        assert!(catalog.len() >= 10);

        let login = catalog.get(&PageId::Login).unwrap();
        assert_eq!(login.elements.len(), 3);
        assert!(login
            .elements
            .iter()
            .any(|e| e.element_type == ElementType::Input));

        let dashboard = catalog.get(&PageId::Dashboard).unwrap();
        assert!(!dashboard.expected_stats_cards.is_empty());
    }
}
