#![warn(clippy::unwrap_used)]

pub mod channels;
pub mod config;
pub mod dsp;
pub mod error;
pub mod event_bus;
pub mod experimentation;
pub mod inference;
pub mod journey;
pub mod loyalty;
pub mod openrtb;
pub mod templates;
pub mod types;

pub use config::AppConfig;
pub use error::{CampaignError, CampaignResult};
