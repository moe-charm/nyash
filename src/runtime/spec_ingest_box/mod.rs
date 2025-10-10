//! SpecIngestBox — centralize nyash_box/hako_box ingestion near libraries
//!
//! Responsibility
//! - Discover spec files near a plugin library (walk up to max depth)
//! - Ingest type_id/method ids into v2 loader box_specs
//!
use std::path::{Path, PathBuf};

/// Find a nearby spec file next to the library path.
pub fn find_near_spec(lib_path: &Path, max_depth: usize) -> Option<PathBuf> {
    let mut base = lib_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    for _ in 0..max_depth {
        let hako_box = base.join("hako_box.toml");
        let nyash_box = base.join("nyash_box.toml");
        if hako_box.exists() { return Some(hako_box); }
        if nyash_box.exists() { return Some(nyash_box); }
        if let Some(parent) = base.parent() { base = parent.to_path_buf(); } else { break; }
    }
    None
}

/// Thin wrapper to ingest specs from a given spec file path.
/// This indirection allows callers to depend on the Box instead of `loader::specs` directly.
pub fn ingest_from_path(
    loader: &crate::runtime::PluginLoaderV2,
    lib_name: &str,
    box_names: &[String],
    spec_path: &Path,
){
    // Delegate to loader method (keeps specs private to loader)
    loader.ingest_specs_from_file(lib_name, box_names, spec_path);
}
