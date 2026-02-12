//! Segment builder — fluent API for constructing segment criteria.

use crate::engine::{Segment, SegmentType};
use crate::predicates::{ComparisonOperator, LogicalOperator, Predicate, PredicateGroup};
use uuid::Uuid;

pub struct SegmentBuilder {
    name: String,
    description: Option<String>,
    segment_type: SegmentType,
    predicates: Vec<Predicate>,
    groups: Vec<PredicateGroup>,
    operator: LogicalOperator,
    tags: Vec<String>,
    is_dynamic: bool,
}

impl SegmentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            segment_type: SegmentType::Custom,
            predicates: Vec::new(),
            groups: Vec::new(),
            operator: LogicalOperator::And,
            tags: Vec::new(),
            is_dynamic: true,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn segment_type(mut self, st: SegmentType) -> Self {
        self.segment_type = st;
        self
    }

    pub fn with_or(mut self) -> Self {
        self.operator = LogicalOperator::Or;
        self
    }

    pub fn attribute_equals(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.predicates.push(Predicate::Attribute {
            key: key.into(),
            operator: ComparisonOperator::Equals,
            value,
        });
        self
    }

    pub fn attribute_gt(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.predicates.push(Predicate::Attribute {
            key: key.into(),
            operator: ComparisonOperator::GreaterThan,
            value,
        });
        self
    }

    pub fn did_event(
        mut self,
        event_name: impl Into<String>,
        min_count: u64,
        within_days: u32,
    ) -> Self {
        self.predicates.push(Predicate::Event {
            event_name: event_name.into(),
            count_operator: ComparisonOperator::GreaterThanOrEqual,
            count: min_count,
            within_days,
        });
        self
    }

    pub fn did_not_do_event(mut self, event_name: impl Into<String>, within_days: u32) -> Self {
        self.predicates.push(Predicate::Event {
            event_name: event_name.into(),
            count_operator: ComparisonOperator::Equals,
            count: 0,
            within_days,
        });
        self
    }

    pub fn in_segment(mut self, segment_id: Uuid) -> Self {
        self.predicates.push(Predicate::SegmentMembership {
            segment_id,
            is_member: true,
        });
        self
    }

    pub fn not_in_segment(mut self, segment_id: Uuid) -> Self {
        self.predicates.push(Predicate::SegmentMembership {
            segment_id,
            is_member: false,
        });
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn build(self) -> Segment {
        let now = chrono::Utc::now();
        Segment {
            id: Uuid::new_v4(),
            name: self.name,
            description: self.description,
            segment_type: self.segment_type,
            criteria: PredicateGroup {
                operator: self.operator,
                predicates: self.predicates,
                groups: self.groups,
            },
            estimated_size: None,
            actual_size: None,
            created_at: now,
            updated_at: now,
            is_dynamic: self.is_dynamic,
            refresh_interval_seconds: Some(300),
            tags: self.tags,
        }
    }
}
