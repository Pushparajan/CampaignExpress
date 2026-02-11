//! Journey orchestration — multi-step user experience flows with branching,
//! waits, actions, and A/B splits for the CampaignExpress ad platform.

pub mod engine;
pub mod evaluator;
pub mod state_machine;
pub mod types;

pub use engine::JourneyEngine;
pub use evaluator::JourneyEvaluator;
