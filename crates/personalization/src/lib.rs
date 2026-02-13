//! Personalization engine — Liquid-like templating, connected content,
//! product recommendations, and catalog management.

pub mod catalog;
pub mod connected_content;
pub mod recommendations;
pub mod templating;

pub use catalog::CatalogEngine;
pub use connected_content::ConnectedContentEngine;
pub use recommendations::RecommendationEngine;
pub use templating::TemplateEngine;
