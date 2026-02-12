//! Core segmentation engine — evaluates user membership in real-time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::predicates::{Predicate, PredicateGroup};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub segment_type: SegmentType,
    pub criteria: PredicateGroup,
    pub estimated_size: Option<u64>,
    pub actual_size: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_dynamic: bool,
    pub refresh_interval_seconds: Option<u64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Behavioral,
    Demographic,
    Predictive,
    Lifecycle,
    Custom,
    Lookalike {
        seed_segment_id: Uuid,
        similarity: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
    pub events: Vec<UserEvent>,
    pub computed_properties: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub event_name: String,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

pub struct SegmentationEngine {
    segments: dashmap::DashMap<Uuid, Segment>,
}

impl SegmentationEngine {
    pub fn new() -> Self {
        Self {
            segments: dashmap::DashMap::new(),
        }
    }

    pub fn register_segment(&self, segment: Segment) {
        self.segments.insert(segment.id, segment);
    }

    pub fn evaluate_user(&self, context: &UserContext) -> Vec<Uuid> {
        let mut memberships = Vec::new();
        for entry in self.segments.iter() {
            let segment = entry.value();
            if self.matches_criteria(context, &segment.criteria) {
                memberships.push(segment.id);
            }
        }
        memberships
    }

    fn matches_criteria(&self, context: &UserContext, group: &PredicateGroup) -> bool {
        match group.operator {
            crate::predicates::LogicalOperator::And => {
                group
                    .predicates
                    .iter()
                    .all(|p| self.evaluate_predicate(context, p))
                    && group
                        .groups
                        .iter()
                        .all(|g| self.matches_criteria(context, g))
            }
            crate::predicates::LogicalOperator::Or => {
                group
                    .predicates
                    .iter()
                    .any(|p| self.evaluate_predicate(context, p))
                    || group
                        .groups
                        .iter()
                        .any(|g| self.matches_criteria(context, g))
            }
        }
    }

    fn evaluate_predicate(&self, context: &UserContext, predicate: &Predicate) -> bool {
        match predicate {
            Predicate::Attribute {
                key,
                operator,
                value,
            } => {
                if let Some(attr) = context.attributes.get(key) {
                    crate::predicates::compare_values(attr, operator, value)
                } else {
                    false
                }
            }
            Predicate::Event {
                event_name,
                count_operator,
                count,
                within_days,
            } => {
                let cutoff = Utc::now() - chrono::Duration::days(*within_days as i64);
                let event_count = context
                    .events
                    .iter()
                    .filter(|e| &e.event_name == event_name && e.timestamp >= cutoff)
                    .count() as u64;
                crate::predicates::compare_numbers(event_count, count_operator, *count)
            }
            Predicate::ComputedProperty {
                key,
                operator,
                value,
            } => {
                if let Some(prop) = context.computed_properties.get(key) {
                    crate::predicates::compare_values(prop, operator, value)
                } else {
                    false
                }
            }
            Predicate::SegmentMembership {
                segment_id,
                is_member,
            } => {
                let in_segment = self.evaluate_user(context).contains(segment_id);
                in_segment == *is_member
            }
        }
    }

    pub fn get_segment(&self, id: &Uuid) -> Option<Segment> {
        self.segments.get(id).map(|s| s.clone())
    }

    pub fn list_segments(&self) -> Vec<Segment> {
        self.segments.iter().map(|s| s.value().clone()).collect()
    }
}

impl Default for SegmentationEngine {
    fn default() -> Self {
        Self::new()
    }
}
