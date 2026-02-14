//! Test actions — the interaction primitives that agents execute against UI
//! pages and API endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::page_objects::PageId;

/// A test action that an agent can perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestAction {
    /// Navigate to a page.
    Navigate { page: PageId, url: String },

    /// Click an element identified by selector.
    Click {
        selector: String,
        description: String,
    },

    /// Type text into an input field.
    TypeText {
        selector: String,
        text: String,
        clear_first: bool,
    },

    /// Select an option from a dropdown.
    Select { selector: String, value: String },

    /// Submit a form.
    SubmitForm { selector: String },

    /// Scroll to an element or position.
    Scroll {
        selector: Option<String>,
        x: i32,
        y: i32,
    },

    /// Wait for an element to appear.
    WaitForElement { selector: String, timeout_ms: u64 },

    /// Wait for a specific duration.
    WaitMs { duration_ms: u64 },

    /// Make an API call.
    ApiCall {
        method: HttpMethod,
        path: String,
        body: Option<serde_json::Value>,
        headers: HashMap<String, String>,
    },

    /// Take a screenshot (for visual regression).
    Screenshot { name: String },

    /// Assert page title.
    AssertTitle { expected: String },

    /// Assert element text content.
    AssertText { selector: String, expected: String },

    /// Assert element exists/visible.
    AssertVisible { selector: String },

    /// Assert element count.
    AssertCount { selector: String, expected: usize },

    /// Assert API response status code.
    AssertStatus { expected: u16 },

    /// Custom action with arbitrary payload.
    Custom {
        name: String,
        params: serde_json::Value,
    },
}

/// HTTP method for API calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Result of executing a test action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestActionResult {
    pub success: bool,
    pub action: TestAction,
    pub response_body: Option<serde_json::Value>,
    pub status_code: Option<u16>,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl TestActionResult {
    /// Create a successful result.
    pub fn ok(action: TestAction, duration_ms: u64) -> Self {
        Self {
            success: true,
            action,
            response_body: None,
            status_code: Some(200),
            duration_ms,
            error: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a failed result.
    pub fn fail(action: TestAction, error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            success: false,
            action,
            response_body: None,
            status_code: None,
            duration_ms,
            error: Some(error.into()),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_serde() {
        let action = TestAction::Click {
            selector: "#submit-btn".into(),
            description: "Click submit".into(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: TestAction = serde_json::from_str(&json).unwrap();
        match parsed {
            TestAction::Click {
                selector,
                description,
            } => {
                assert_eq!(selector, "#submit-btn");
                assert_eq!(description, "Click submit");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_action_result() {
        let action = TestAction::Navigate {
            page: PageId::Dashboard,
            url: "/".into(),
        };
        let ok = TestActionResult::ok(action.clone(), 50);
        assert!(ok.success);
        assert_eq!(ok.duration_ms, 50);

        let fail = TestActionResult::fail(action, "Element not found", 100);
        assert!(!fail.success);
        assert_eq!(fail.error, Some("Element not found".into()));
    }
}
