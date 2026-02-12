//! Intelligent delivery — send-time optimization, frequency capping,
//! rate limiting, quiet hours, and message throttling.

pub mod frequency_capping;
pub mod quiet_hours;
pub mod send_time;
pub mod throttle;

pub use frequency_capping::FrequencyCapEngine;
pub use quiet_hours::QuietHoursEngine;
pub use send_time::SendTimeOptimizer;
pub use throttle::MessageThrottler;
