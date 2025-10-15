//! [using] section processing: paths, aliases, and packages

use crate::using::policy::UsingPolicy;
use crate::using::spec::{PackageKind, UsingPackage};
use std::collections::HashMap;

/// Process [using.paths] array
pub fn process_paths(
    paths_arr: &[toml::Value],
    using_paths: &mut Vec<String>,
    policy: &mut UsingPolicy,
) {
    for p in paths_arr {
        if let Some(s) = p.as_str() {
            let s = s.trim();
            if !s.is_empty() {
                using_paths.push(s.to_string());
                policy.search_paths.push(s.to_string());
            }
        }
    }
}

/// Process [using.aliases] table (DEPRECATED)
pub fn process_aliases(aliases_tbl: &toml::Table, aliases: &mut HashMap<String, String>) {
    if !aliases_tbl.is_empty() && !crate::config::env::cli_quiet() {
        eprintln!("[deprecate] [modules.aliases] is deprecated; use [modules.overrides] instead");
    }

    for (k, v) in aliases_tbl.iter() {
        if let Some(target) = v.as_str() {
            aliases.insert(k.to_string(), target.to_string());
        }
    }
}

/// Process named packages in [using] section
/// Any subtable not "paths" or "aliases" is treated as a package definition
pub fn process_packages(using_tbl: &toml::Table, packages: &mut HashMap<String, UsingPackage>) {
    for (k, v) in using_tbl.iter() {
        // Skip reserved keys
        if k == "paths" || k == "aliases" {
            continue;
        }

        if let Some(tbl) = v.as_table() {
            let kind = tbl
                .get("kind")
                .and_then(|x| x.as_str())
                .map(PackageKind::from_str)
                .unwrap_or(PackageKind::Package);

            // path is required
            if let Some(path_s) = tbl.get("path").and_then(|x| x.as_str()) {
                let path = path_s.to_string();
                let main = tbl.get("main").and_then(|x| x.as_str()).map(|s| s.to_string());
                let bid = tbl.get("bid").and_then(|x| x.as_str()).map(|s| s.to_string());
                packages.insert(
                    k.to_string(),
                    UsingPackage {
                        kind,
                        path,
                        main,
                        bid,
                    },
                );
            }
        }
    }
}

/// Process legacy top-level [aliases] table (migration compatibility)
pub fn process_legacy_aliases(
    aliases_tbl: &toml::Table,
    aliases: &mut HashMap<String, String>,
) {
    for (k, v) in aliases_tbl.iter() {
        if let Some(target) = v.as_str() {
            aliases.insert(k.to_string(), target.to_string());
        }
    }
}
