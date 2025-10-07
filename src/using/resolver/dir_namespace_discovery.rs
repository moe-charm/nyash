//! Directory-as-Namespace discovery: automatic module detection from filesystem

use crate::using::errors::UsingError;
use std::collections::{HashMap, HashSet};

/// Discover and append modules from directory structure (NYASH_USING_DIR_NS=1)
///
/// Scans apps/ directory for *.hako files and automatically adds them as
/// lowest-precedence namespace candidates. Conflicts trigger warnings or errors
/// in strict mode.
pub fn discover_and_append_if_enabled(
    pending_modules: &mut Vec<(String, String)>,
) -> Result<(), UsingError> {
    // Early return if directory-as-namespace discovery is disabled
    if std::env::var("NYASH_USING_DIR_NS").ok().as_deref() != Some("1") {
        return Ok(());
    }

    let root = std::env::var("NYASH_ROOT")
        .ok()
        .unwrap_or_else(|| ".".to_string());
    let apps = std::path::Path::new(&root).join("apps");

    if !apps.exists() {
        return Ok(());
    }

    let opts = crate::config::module_discovery::ModuleDiscoveryOptions::default();
    let auto = crate::config::module_discovery::discover_entries_under(&apps, &opts);

    // Build maps of existing namespaces and their paths (for conflict detection)
    let mut existing: HashSet<String> = HashSet::new();
    let mut existing_map: HashMap<String, String> = HashMap::new();
    for (ns, p) in pending_modules.iter() {
        existing.insert(ns.clone());
        existing_map.insert(ns.clone(), p.clone());
    }

    // Accumulate multiplicities (ns -> {paths}) for conflict detection
    let mut multi: HashMap<String, HashSet<String>> = HashMap::new();
    for (ns, p) in pending_modules.iter() {
        multi.entry(ns.clone()).or_default().insert(p.clone());
    }

    // Merge auto entries; if an ns already exists, record both existing and auto paths
    // for strict conflict check
    for (ns, p) in auto.into_iter() {
        if !existing.contains(&ns) {
            if let Some(ps) = p.to_str() {
                pending_modules.push((ns.clone(), ps.to_string()));
                multi.entry(ns).or_default().insert(ps.to_string());
            }
        } else {
            if let Some(ps) = p.to_str() {
                multi.entry(ns.clone()).or_default().insert(ps.to_string());
                if let Some(prev) = existing_map.get(&ns) {
                    multi.entry(ns.clone()).or_default().insert(prev.clone());
                }
            }
        }
    }

    // Lightweight conflict check on the accumulated set (warn/strict)
    super::conflict_detector::detect_conflicts(&multi)?;

    Ok(())
}
