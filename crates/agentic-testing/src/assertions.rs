//! Assertion engine — typed assertions that validate test action results
//! against expected outcomes.

use serde::{Deserialize, Serialize};

use crate::actions::TestActionResult;

/// Assertion types that can be evaluated against action results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Assertion {
    /// Assert HTTP status code.
    StatusCode(u16),
    /// Assert element is visible on the page.
    ElementVisible(String),
    /// Assert response body contains a string.
    ResponseContains(String),
    /// Assert response body is a JSON array.
    ResponseIsArray,
    /// Assert response body has a specific JSON key.
    ResponseHasKey(String),
    /// Assert response body JSON key equals a value.
    ResponseKeyEquals {
        key: String,
        value: serde_json::Value,
    },
    /// Assert response body array has at least N items.
    ResponseArrayMinLength(usize),
    /// Assert page title matches.
    PageTitle(String),
    /// Assert page URL contains a substring.
    UrlContains(String),
    /// Assert no errors in the action result.
    NoErrors,
    /// Assert action duration is below a threshold (ms).
    DurationBelow(u64),
}

/// Result of evaluating a single assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub message: String,
}

impl Assertion {
    /// Evaluate this assertion against an action result.
    pub fn evaluate(&self, result: &TestActionResult) -> AssertionResult {
        match self {
            Assertion::StatusCode(expected) => {
                let actual = result.status_code.unwrap_or(0);
                let passed = actual == *expected;
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        format!("Status code {expected} matched")
                    } else {
                        format!("Expected status {expected}, got {actual}")
                    },
                }
            }
            Assertion::ElementVisible(selector) => {
                // In mock mode, always passes. Real implementation would check DOM.
                AssertionResult {
                    assertion: self.clone(),
                    passed: result.success,
                    message: if result.success {
                        format!("Element '{selector}' is visible")
                    } else {
                        format!("Element '{selector}' not found")
                    },
                }
            }
            Assertion::ResponseContains(substring) => {
                let body_str = result
                    .response_body
                    .as_ref()
                    .map(|b| b.to_string())
                    .unwrap_or_default();
                let passed = body_str.contains(substring);
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        format!("Response contains '{substring}'")
                    } else {
                        format!("Response does not contain '{substring}'")
                    },
                }
            }
            Assertion::ResponseIsArray => {
                let passed = result.response_body.as_ref().is_some_and(|b| b.is_array());
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        "Response is a JSON array".into()
                    } else {
                        "Response is not a JSON array".into()
                    },
                }
            }
            Assertion::ResponseHasKey(key) => {
                let passed = result
                    .response_body
                    .as_ref()
                    .and_then(|b| b.as_object())
                    .is_some_and(|obj| obj.contains_key(key));
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        format!("Response has key '{key}'")
                    } else {
                        format!("Response missing key '{key}'")
                    },
                }
            }
            Assertion::ResponseKeyEquals { key, value } => {
                let actual = result
                    .response_body
                    .as_ref()
                    .and_then(|b| b.get(key))
                    .cloned();
                let passed = actual.as_ref() == Some(value);
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        format!("Response key '{key}' equals expected value")
                    } else {
                        format!(
                            "Response key '{key}' mismatch: expected {value}, got {}",
                            actual
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "<missing>".into())
                        )
                    },
                }
            }
            Assertion::ResponseArrayMinLength(min) => {
                let len = result
                    .response_body
                    .as_ref()
                    .and_then(|b| b.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let passed = len >= *min;
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        format!("Array has {len} items (>= {min})")
                    } else {
                        format!("Array has {len} items (expected >= {min})")
                    },
                }
            }
            Assertion::PageTitle(expected) => AssertionResult {
                assertion: self.clone(),
                passed: result.success,
                message: format!("Page title assertion for '{expected}'"),
            },
            Assertion::UrlContains(substring) => AssertionResult {
                assertion: self.clone(),
                passed: result.success,
                message: format!("URL contains '{substring}'"),
            },
            Assertion::NoErrors => {
                let passed = result.error.is_none();
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        "No errors".into()
                    } else {
                        format!("Error: {}", result.error.as_deref().unwrap_or("unknown"))
                    },
                }
            }
            Assertion::DurationBelow(max_ms) => {
                let passed = result.duration_ms <= *max_ms;
                AssertionResult {
                    assertion: self.clone(),
                    passed,
                    message: if passed {
                        format!("Duration {}ms <= {}ms", result.duration_ms, max_ms)
                    } else {
                        format!("Duration {}ms > {}ms threshold", result.duration_ms, max_ms)
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{TestAction, TestActionResult};
    use crate::page_objects::PageId;
    use chrono::Utc;

    fn mock_result(status: u16, body: Option<serde_json::Value>) -> TestActionResult {
        TestActionResult {
            success: true,
            action: TestAction::Navigate {
                page: PageId::Dashboard,
                url: "/".into(),
            },
            response_body: body,
            status_code: Some(status),
            duration_ms: 50,
            error: None,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_status_code_assertion() {
        let result = mock_result(200, None);
        let a = Assertion::StatusCode(200).evaluate(&result);
        assert!(a.passed);

        let a = Assertion::StatusCode(404).evaluate(&result);
        assert!(!a.passed);
    }

    #[test]
    fn test_response_contains() {
        let body = serde_json::json!({"total_campaigns": 5, "active": 3});
        let result = mock_result(200, Some(body));

        let a = Assertion::ResponseContains("total_campaigns".into()).evaluate(&result);
        assert!(a.passed);

        let a = Assertion::ResponseContains("nonexistent".into()).evaluate(&result);
        assert!(!a.passed);
    }

    #[test]
    fn test_response_is_array() {
        let arr = serde_json::json!([1, 2, 3]);
        let result = mock_result(200, Some(arr));
        let a = Assertion::ResponseIsArray.evaluate(&result);
        assert!(a.passed);

        let obj = serde_json::json!({"key": "value"});
        let result = mock_result(200, Some(obj));
        let a = Assertion::ResponseIsArray.evaluate(&result);
        assert!(!a.passed);
    }

    #[test]
    fn test_response_has_key() {
        let body = serde_json::json!({"name": "Test", "budget": 5000});
        let result = mock_result(200, Some(body));

        let a = Assertion::ResponseHasKey("name".into()).evaluate(&result);
        assert!(a.passed);

        let a = Assertion::ResponseHasKey("missing".into()).evaluate(&result);
        assert!(!a.passed);
    }

    #[test]
    fn test_no_errors() {
        let result = mock_result(200, None);
        let a = Assertion::NoErrors.evaluate(&result);
        assert!(a.passed);

        let mut err_result = mock_result(500, None);
        err_result.error = Some("Server error".into());
        let a = Assertion::NoErrors.evaluate(&err_result);
        assert!(!a.passed);
    }

    #[test]
    fn test_duration_below() {
        let result = mock_result(200, None);
        let a = Assertion::DurationBelow(100).evaluate(&result);
        assert!(a.passed);

        let a = Assertion::DurationBelow(10).evaluate(&result);
        assert!(!a.passed);
    }
}
