//! High-level section processors for TOML configuration

use crate::using::errors::UsingError;
use crate::using::policy::UsingPolicy;
use crate::using::spec::UsingPackage;
use std::collections::HashMap;
use std::path::Path;

/// Process entire [modules] section
///
/// Handles:
/// - [modules.workspace]: workspace members
/// - [modules.overrides]: override entries
/// - [modules.aliases]: deprecated alias entries
/// - [modules.options]: discovery and exclude options
/// - Conflict detection after processing
pub fn process_modules_section(
    modules_tbl: &toml::Table,
    config_path: &Path,
    pending_modules: &mut Vec<(String, String)>,
    aliases: &mut HashMap<String, String>,
) -> Result<(), UsingError> {
    // [modules.workspace]
    if let Some(ws_tbl) = modules_tbl.get("workspace").and_then(|v| v.as_table()) {
        let (ws_manifests, ws_versions, mut ws_modules) =
            super::workspace_loader::load_workspace_members(ws_tbl, config_path);

        pending_modules.append(&mut ws_modules);
        super::dep_graph_analyzer::analyze_dependencies(&ws_manifests, &ws_versions)?;
    }

    // [modules.overrides]
    if let Some(ovr) = modules_tbl.get("overrides").and_then(|v| v.as_table()) {
        super::modules_processor::process_overrides(ovr, pending_modules);
    }

    // [modules.aliases] (DEPRECATED)
    if let Some(alias_tbl) = modules_tbl.get("aliases").and_then(|v| v.as_table()) {
        super::modules_processor::process_aliases(alias_tbl, aliases);
    }

    // [modules.options]
    if let Some(opts_tbl) = modules_tbl.get("options").and_then(|v| v.as_table()) {
        super::modules_processor::process_options(opts_tbl);
    }

    // Conflict detection after all modules processing
    super::conflict_detector::detect_conflicts_from_modules(pending_modules)?;

    Ok(())
}

/// Process entire [using] section
///
/// Handles:
/// - [using.paths]: search paths
/// - [using.aliases]: deprecated alias entries
/// - Named packages (any other subtables)
pub fn process_using_section(
    using_tbl: &toml::Table,
    using_paths: &mut Vec<String>,
    aliases: &mut HashMap<String, String>,
    packages: &mut HashMap<String, UsingPackage>,
    policy: &mut UsingPolicy,
) {
    // [using.paths]
    if let Some(paths_arr) = using_tbl.get("paths").and_then(|v| v.as_array()) {
        super::using_processor::process_paths(paths_arr, using_paths, policy);
    }

    // [using.aliases] (DEPRECATED)
    if let Some(alias_tbl) = using_tbl.get("aliases").and_then(|v| v.as_table()) {
        super::using_processor::process_aliases(alias_tbl, aliases);
    }

    // Named packages
    super::using_processor::process_packages(using_tbl, packages);
}
