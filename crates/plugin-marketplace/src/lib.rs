//! Plugin marketplace — discovery, installation, sandboxing, developer portal,
//! and plugin lifecycle management.

pub mod developer;
pub mod registry;
pub mod sandbox;
pub mod store;

pub use developer::DeveloperPortal;
pub use registry::PluginRegistry;
pub use sandbox::PluginSandbox;
pub use store::PluginStore;
