#![warn(clippy::unwrap_used)]

pub mod channel_rest;
pub mod dsp_rest;
pub mod grpc;
pub mod loyalty_rest;
pub mod rest;
pub mod server;
pub mod swagger;

pub use server::ApiServer;
pub use swagger::ApiDoc;
