#![warn(clippy::unwrap_used)]

pub mod client;
pub mod local;

pub use client::RedisCache;
pub use local::LocalCache;
