//! Omnichannel ingest and activation engine.
//!
//! Ingest: consumes real-time events from mobile, POS, kiosk, web via NATS queues.
//! Activation: delivers personalized offers to push, SMS, email, paid media, in-store.
//! Channels: in-app messaging, content cards, WhatsApp, web push, and more.

pub mod activation;
pub mod content_cards;
pub mod email;
pub mod in_app;
pub mod ingest;
pub mod sms;
pub mod web_push;
pub mod whatsapp;

pub use activation::ActivationDispatcher;
pub use content_cards::ContentCardEngine;
pub use email::SendGridProvider;
pub use in_app::InAppEngine;
pub use ingest::IngestProcessor;
pub use sms::SmsProvider;
pub use web_push::WebPushProvider;
pub use whatsapp::WhatsAppProvider;
