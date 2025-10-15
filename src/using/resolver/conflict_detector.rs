//! Namespace conflict detection utilities for using resolver

use std::collections::{HashMap, HashSet};
use crate::using::errors::UsingError;

/// Build a map of namespace -> {paths} from pending_modules list
pub fn build_namespace_map(
    pending_modules: &[(String, String)]
) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    for (ns, p) in pending_modules.iter() {
        let canon = match std::fs::canonicalize(p) {
            Ok(pb) => pb.display().to_string(),
            Err(_) => {
                if let Ok(root) = std::env::var("NYASH_ROOT") {
                    let alt = std::path::Path::new(&root).join(p);
                    std::fs::canonicalize(&alt)
                        .map(|pb| pb.display().to_string())
                        .unwrap_or_else(|_| p.clone())
                } else { p.clone() }
            }
        };
        map.entry(ns.clone()).or_default().insert(canon);
    }
    map
}

/// Detect namespace conflicts in the given map
/// Returns error in strict mode (NYASH_USING_CHECKS_STRICT=1), warns otherwise
pub fn detect_conflicts(
    map: &HashMap<String, HashSet<String>>
) -> Result<(), UsingError> {
    for (ns, paths_set) in map.iter() {
        if paths_set.len() > 1 {
            // Prepare sorted path list once
            let mut paths: Vec<String> = paths_set.iter().cloned().collect();
            paths.sort();
            // Optional suppression for dev/profiles: do not print noise when suppressed
            let suppress = std::env::var("NYASH_SUPPRESS_MODULE_CONFLICT").ok().as_deref() == Some("1")
                || std::env::var("HAKO_SUPPRESS_MODULE_CONFLICT").ok().as_deref() == Some("1");
            if !suppress {
                eprintln!("{}",
                    crate::common::diagnostics::modules_error::conflict(ns, &paths));
            }

            if std::env::var("NYASH_USING_CHECKS_STRICT")
                .ok()
                .as_deref() == Some("1")
            {
                return Err(UsingError::Conflict {
                    ns: ns.clone(),
                    paths: paths.join(",")
                });
            }
        }
    }
    Ok(())
}

/// Convenience function: build map and detect conflicts in one call
pub fn detect_conflicts_from_modules(
    pending_modules: &[(String, String)]
) -> Result<(), UsingError> {
    let map = build_namespace_map(pending_modules);
    detect_conflicts(&map)
}
