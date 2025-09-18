//! Dynamic Plugin Loader for Nyash (split module)
//!
//! Refactored into smaller files to improve readability while preserving
//! the original public API surface used across the interpreter:
//! - types.rs: globals and native handles
//! - proxies.rs: Box proxy implementations
//! - loader.rs: public loader entrypoints

mod types;
mod proxies;
mod loader;

// Re-export to preserve original paths like
// crate::interpreter::plugin_loader::{PluginLoader, FileBoxProxy, ..., PLUGIN_CACHE}
pub use loader::PluginLoader;
pub use proxies::{
    FileBoxProxy, MathBoxProxy, RandomBoxProxy, TimeBoxProxy, DateTimeBoxProxy,
};
pub use types::{
    PLUGIN_CACHE, LoadedPlugin, PluginInfo, FileBoxHandle, MathBoxHandle,
    RandomBoxHandle, TimeBoxHandle, DateTimeBoxHandle,
};

