//! Mobile SDK server-side support — session management, event ingestion,
//! device registration, and SDK configuration for iOS, Android, React Native, Flutter.

pub mod config;
pub mod device;
pub mod events;
pub mod sessions;

pub use config::SdkConfigManager;
pub use device::DeviceRegistry;
pub use events::EventIngester;
pub use sessions::SessionManager;
