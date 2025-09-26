use crate::using::errors::UsingError;
use crate::using::policy::UsingPolicy;
use crate::using::spec::{PackageKind, UsingPackage};
use std::collections::HashMap;

/// Populate using context vectors from nyash.toml (if present).
/// Keeps behavior aligned with existing runner pipeline:
///  - Adds [using.paths] entries to `using_paths`
///  - Flattens [modules] into (name, path) pairs appended to `pending_modules`
///  - Reads optional [aliases] table (k -> v)
pub fn populate_from_toml(
    using_paths: &mut Vec<String>,
    pending_modules: &mut Vec<(String, String)>,
    aliases: &mut HashMap<String, String>,
    packages: &mut HashMap<String, UsingPackage>,
) -> Result<UsingPolicy, UsingError> {
    let mut policy = UsingPolicy::default();
    let path = std::path::Path::new("nyash.toml");
    if !path.exists() {
        return Ok(policy);
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| UsingError::ReadToml(e.to_string()))?;
    let doc = toml::from_str::<toml::Value>(&text)
        .map_err(|e| UsingError::ParseToml(e.to_string()))?;

    // [modules] table flatten: supports nested namespaces (a.b.c = "path")
    if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) {
        fn visit(prefix: &str, tbl: &toml::value::Table, out: &mut Vec<(String, String)>) {
            for (k, v) in tbl.iter() {
                let name = if prefix.is_empty() { k.to_string() } else { format!("{}.{}", prefix, k) };
                if let Some(s) = v.as_str() {
                    out.push((name, s.to_string()));
                } else if let Some(t) = v.as_table() {
                    visit(&name, t, out);
                }
            }
        }
        visit("", mods, pending_modules);
    }

    // [using.paths] array
    if let Some(using_tbl) = doc.get("using").and_then(|v| v.as_table()) {
        // paths
        if let Some(paths_arr) = using_tbl.get("paths").and_then(|v| v.as_array()) {
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
        // aliases
        if let Some(alias_tbl) = using_tbl.get("aliases").and_then(|v| v.as_table()) {
            for (k, v) in alias_tbl.iter() {
                if let Some(target) = v.as_str() {
                    aliases.insert(k.to_string(), target.to_string());
                }
            }
        }
        // named packages: any subtable not paths/aliases is a package
        for (k, v) in using_tbl.iter() {
            if k == "paths" || k == "aliases" { continue; }
            if let Some(tbl) = v.as_table() {
                let kind = tbl.get("kind").and_then(|x| x.as_str()).map(PackageKind::from_str).unwrap_or(PackageKind::Package);
                // path is required
                if let Some(path_s) = tbl.get("path").and_then(|x| x.as_str()) {
                    let path = path_s.to_string();
                    let main = tbl.get("main").and_then(|x| x.as_str()).map(|s| s.to_string());
                    let bid = tbl.get("bid").and_then(|x| x.as_str()).map(|s| s.to_string());
                    packages.insert(k.to_string(), UsingPackage { kind, path, main, bid });
                }
            }
        }
    }

    // legacy top-level [aliases] also accepted (migration)
    if let Some(alias_tbl) = doc.get("aliases").and_then(|v| v.as_table()) {
        for (k, v) in alias_tbl.iter() {
            if let Some(target) = v.as_str() {
                aliases.insert(k.to_string(), target.to_string());
            }
        }
    }

    Ok(policy)
}
