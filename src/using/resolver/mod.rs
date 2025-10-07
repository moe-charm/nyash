//! Using resolver submodules

pub mod conflict_detector;
pub mod config_locator;
pub mod toml_parser;
pub mod workspace_loader;
pub mod dep_graph_analyzer;
pub mod core;

// Re-export main function for backwards compatibility
pub use core::populate_from_toml;
