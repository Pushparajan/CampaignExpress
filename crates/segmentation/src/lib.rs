//! Real-time segmentation engine — computed properties, behavioral triggers,
//! predictive segments, and SQL-like segment builder.

pub mod builder;
pub mod computed;
pub mod engine;
pub mod predicates;

pub use builder::SegmentBuilder;
pub use computed::ComputedPropertyEngine;
pub use engine::SegmentationEngine;
