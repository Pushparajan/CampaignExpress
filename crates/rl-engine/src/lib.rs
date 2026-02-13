//! Reinforcement Learning engine — multi-armed bandits (Thompson Sampling, UCB1,
//! Epsilon-Greedy), contextual bandits (LinUCB), holdout groups, guardrails,
//! and AI explainability dashboard.

pub mod bandits;
pub mod contextual;
pub mod explainability;
pub mod guardrails;
pub mod holdout;
pub mod offerfit;

pub use bandits::BanditEngine;
pub use contextual::ContextualBanditEngine;
pub use explainability::ExplainabilityEngine;
pub use guardrails::GuardrailsEngine;
pub use holdout::HoldoutManager;
pub use offerfit::OfferFitClient;
