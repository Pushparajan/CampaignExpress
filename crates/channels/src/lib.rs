//! Omnichannel ingest and activation engine.
//!
//! Ingest: consumes real-time events from mobile, POS, kiosk, web via NATS queues.
//! Activation: delivers personalized offers to push, SMS, email, paid media, in-store.

pub mod activation;
pub mod email;
pub mod ingest;

pub use activation::ActivationDispatcher;
pub use email::SendGridProvider;
pub use ingest::IngestProcessor;
