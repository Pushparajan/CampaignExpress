//! Hardware-specific inference backends.
//!
//! Each module implements [`campaign_core::inference::CoLaNetProvider`] for a
//! different execution environment, allowing the platform to run identically
//! on developer laptops (CPU), cloud accelerators (Inferentia, Groq), ARM
//! servers (Ampere Altra), and custom silicon (Tenstorrent).

pub mod ampere;
pub mod cpu;
pub mod groq;
pub mod inferentia;
pub mod tenstorrent;
