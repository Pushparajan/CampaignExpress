//! Predicate types and evaluation logic for segment criteria.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateGroup {
    pub operator: LogicalOperator,
    pub predicates: Vec<Predicate>,
    pub groups: Vec<PredicateGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    Attribute {
        key: String,
        operator: ComparisonOperator,
        value: serde_json::Value,
    },
    Event {
        event_name: String,
        count_operator: ComparisonOperator,
        count: u64,
        within_days: u32,
    },
    ComputedProperty {
        key: String,
        operator: ComparisonOperator,
        value: serde_json::Value,
    },
    SegmentMembership {
        segment_id: Uuid,
        is_member: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    IsSet,
    IsNotSet,
    InList,
    NotInList,
    Between,
    Regex,
}

#[allow(clippy::unnecessary_map_or)]
pub fn compare_values(
    actual: &serde_json::Value,
    operator: &ComparisonOperator,
    expected: &serde_json::Value,
) -> bool {
    match operator {
        ComparisonOperator::Equals => actual == expected,
        ComparisonOperator::NotEquals => actual != expected,
        ComparisonOperator::GreaterThan => {
            numeric_cmp(actual, expected).map_or(false, |o| o == std::cmp::Ordering::Greater)
        }
        ComparisonOperator::GreaterThanOrEqual => {
            numeric_cmp(actual, expected).map_or(false, |o| o != std::cmp::Ordering::Less)
        }
        ComparisonOperator::LessThan => {
            numeric_cmp(actual, expected).map_or(false, |o| o == std::cmp::Ordering::Less)
        }
        ComparisonOperator::LessThanOrEqual => {
            numeric_cmp(actual, expected).map_or(false, |o| o != std::cmp::Ordering::Greater)
        }
        ComparisonOperator::Contains => actual
            .as_str()
            .zip(expected.as_str())
            .map_or(false, |(a, e)| a.contains(e)),
        ComparisonOperator::NotContains => actual
            .as_str()
            .zip(expected.as_str())
            .map_or(true, |(a, e)| !a.contains(e)),
        ComparisonOperator::StartsWith => actual
            .as_str()
            .zip(expected.as_str())
            .map_or(false, |(a, e)| a.starts_with(e)),
        ComparisonOperator::EndsWith => actual
            .as_str()
            .zip(expected.as_str())
            .map_or(false, |(a, e)| a.ends_with(e)),
        ComparisonOperator::IsSet => !actual.is_null(),
        ComparisonOperator::IsNotSet => actual.is_null(),
        ComparisonOperator::InList => expected
            .as_array()
            .map_or(false, |list| list.contains(actual)),
        ComparisonOperator::NotInList => expected
            .as_array()
            .map_or(true, |list| !list.contains(actual)),
        _ => false,
    }
}

pub fn compare_numbers(actual: u64, operator: &ComparisonOperator, expected: u64) -> bool {
    match operator {
        ComparisonOperator::Equals => actual == expected,
        ComparisonOperator::NotEquals => actual != expected,
        ComparisonOperator::GreaterThan => actual > expected,
        ComparisonOperator::GreaterThanOrEqual => actual >= expected,
        ComparisonOperator::LessThan => actual < expected,
        ComparisonOperator::LessThanOrEqual => actual <= expected,
        _ => false,
    }
}

fn numeric_cmp(a: &serde_json::Value, b: &serde_json::Value) -> Option<std::cmp::Ordering> {
    let a_num = a.as_f64()?;
    let b_num = b.as_f64()?;
    a_num.partial_cmp(&b_num)
}
