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

    // Top-level direct mappings under [modules] (excluding known subsections)
    // Example:
    // [modules]
    // selfhost.shared.mir.schema = "selfhost/shared/mir/mir_schema_box.hako"
    // selfhost.shared.mir.builder = "selfhost/shared/mir/block_builder_box.hako"
    // These were previously ignored; include them by flattening a filtered view.
    {
        let mut filtered: toml::value::Table = toml::value::Table::new();
        for (k, v) in modules_tbl.iter() {
            // Skip known subsections that have dedicated processors
            if k == "workspace" || k == "overrides" || k == "aliases" || k == "options" {
                continue;
            }
            filtered.insert(k.clone(), v.clone());
        }
        if !filtered.is_empty() {
            let mut flat = crate::common::using_core::flatten_modules_table(&filtered);
            pending_modules.append(&mut flat);
        }
    }

    // [modules.aliases] (DEPRECATED)
    if let Some(alias_tbl) = modules_tbl.get("aliases").and_then(|v| v.as_table()) {
        super::modules_processor::process_aliases(alias_tbl, aliases);
    }

    // [modules.options]
    if let Some(opts_tbl) = modules_tbl.get("options").and_then(|v| v.as_table()) {
        super::modules_processor::process_options(opts_tbl);
    }

// Normalize and deduplicate module paths before conflict detection
    normalize_and_dedup_modules(config_path, pending_modules);
    // Conflict detection after all modules processing (post-normalization)
    super::conflict_detector::detect_conflicts_from_modules(pending_modules)?;

    Ok(())
}

/// Normalize module paths to absolute (based on config_path dir) and
/// deduplicate entries that resolve to the same canonical path per namespace.
fn normalize_and_dedup_modules(config_path: &std::path::Path, pending_modules: &mut Vec<(String, String)>) {
    let base_dir = config_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut out: Vec<(String, String)> = Vec::with_capacity(pending_modules.len());
    use std::collections::{HashMap, HashSet};
    let mut seen: HashMap<String, HashSet<String>> = HashMap::new(); // ns -> {canon_path}
    for (ns, p) in pending_modules.iter() {
        let resolved = if std::path::Path::new(p).is_absolute() {
            std::path::PathBuf::from(p)
        } else {
            base_dir.join(p)
        };
        // Best-effort canonicalization; fall back to normalized path if it fails
        let canon = std::fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());
        let canon_s = canon.display().to_string();
        let entry = seen.entry(ns.clone()).or_insert_with(HashSet::new);
        if entry.contains(&canon_s) {
            continue; // duplicate of existing resolved path for this namespace
        }
        entry.insert(canon_s);
        out.push((ns.clone(), resolved.display().to_string()));
    }
    *pending_modules = out;
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
