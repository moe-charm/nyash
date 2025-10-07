use crate::using::errors::UsingError;
use crate::using::policy::UsingPolicy;
use crate::using::spec::{PackageKind, UsingPackage};
use std::collections::HashMap;

/// Populate using context vectors from configuration file (hako.toml preferred; compat nyash.toml/hakorune.toml).
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

    // Locate configuration file
    let chosen = super::config_locator::locate_config_file();
    let path = chosen
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| std::path::Path::new("nyash.toml"));

    // Emit trace information
    super::config_locator::trace_config_location(path);

    // Early return if file doesn't exist
    if !path.exists() {
        return Ok(policy);
    }

    // Load and parse TOML
    let doc = super::toml_parser::load_and_parse_toml(path)?;

    // [modules] table flatten: supports nested namespaces (a.b.c = "path")
    if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) {
        // main table: no direct flatten (use workspace/overrides for clarity)
        // [modules.workspace] — load module.toml manifests and export public entries
        if let Some(ws_tbl) = mods.get("workspace").and_then(|v| v.as_table()) {
            let (ws_manifests, ws_versions, mut ws_modules) =
                super::workspace_loader::load_workspace_members(ws_tbl, path);

            // Append workspace modules
            pending_modules.append(&mut ws_modules);

            // Analyze dependency graph (cycle detection + version checks)
            super::dep_graph_analyzer::analyze_dependencies(&ws_manifests, &ws_versions)?;
        }
        // [modules.overrides] takes precedence (append; later wins in map usage)
        if let Some(ovr) = mods.get("overrides").and_then(|v| v.as_table()) {
            let mut v = crate::common::using_core::flatten_modules_table(ovr);
            pending_modules.append(&mut v);
        }
        // [modules.aliases] (DEPRECATED): injects alias redirects (recorded in aliases map)
        if let Some(alias_tbl) = mods.get("aliases").and_then(|v| v.as_table()) {
            if !alias_tbl.is_empty() && !crate::config::env::cli_quiet() { eprintln!("[deprecate] [modules.aliases] is deprecated; use [modules.overrides] instead"); }
            for (k, v) in alias_tbl.iter() {
                if let Some(target) = v.as_str() {
                    aliases.insert(k.to_string(), target.to_string());
                }
            }
        }
        // [modules.options] → discovery/env オプション（簡易実装）
        if let Some(opts_tbl) = mods.get("options").and_then(|v| v.as_table()) {
            // enable_discovery
            if let Some(b) = opts_tbl.get("enable_discovery").and_then(|v| v.as_bool()) {
                std::env::set_var("NYASH_DISCOVER_MODULES", if b { "1" } else { "0" });
            }
            // roots (array of strings)
            if let Some(arr) = opts_tbl.get("roots").and_then(|v| v.as_array()) {
                let mut roots: Vec<String> = Vec::new();
                for e in arr.iter() { if let Some(s) = e.as_str() { let s = s.trim(); if !s.is_empty() { roots.push(s.to_string()); } } }
                if !roots.is_empty() {
                    std::env::set_var("NYASH_DISCOVER_ROOTS", roots.join(":"));
                }
            }
            // exclude array（ヒューリスティック: キーワードでON）
            if let Some(arr) = opts_tbl.get("exclude").and_then(|v| v.as_array()) {
                let mut ex_archive = None;
                let mut ex_uscore = None;
                let mut ex_test = None;
                let mut ex_example = None;
                for e in arr.iter() {
                    if let Some(s) = e.as_str() {
                        let t = s.to_ascii_lowercase();
                        if t.contains("archive") { ex_archive = Some(true); }
                        if t.contains("_*/") || t.contains("/_*") { ex_uscore = Some(true); }
                        if t.contains("test_") { ex_test = Some(true); }
                        if t.contains("example_") { ex_example = Some(true); }
                    }
                }
                if let Some(b) = ex_archive { std::env::set_var("NYASH_DISCOVER_EXCLUDE_ARCHIVE", if b {"1"} else {"0"}); }
                if let Some(b) = ex_uscore  { std::env::set_var("NYASH_DISCOVER_EXCLUDE_UNDERSCORE_DIRS", if b {"1"} else {"0"}); }
                if let Some(b) = ex_test    { std::env::set_var("NYASH_DISCOVER_EXCLUDE_TEST_PREFIX", if b {"1"} else {"0"}); }
                if let Some(b) = ex_example { std::env::set_var("NYASH_DISCOVER_EXCLUDE_EXAMPLE_PREFIX", if b {"1"} else {"0"}); }
            }
            // boolean 個別指定も許可（exclude_*）
            if let Some(b) = opts_tbl.get("exclude_archive").and_then(|v| v.as_bool()) {
                std::env::set_var("NYASH_DISCOVER_EXCLUDE_ARCHIVE", if b {"1"} else {"0"});
            }
            if let Some(b) = opts_tbl.get("exclude_underscore_dirs").and_then(|v| v.as_bool()) {
                std::env::set_var("NYASH_DISCOVER_EXCLUDE_UNDERSCORE_DIRS", if b {"1"} else {"0"});
            }
            if let Some(b) = opts_tbl.get("exclude_test_prefix").and_then(|v| v.as_bool()) {
                std::env::set_var("NYASH_DISCOVER_EXCLUDE_TEST_PREFIX", if b {"1"} else {"0"});
            }
            if let Some(b) = opts_tbl.get("exclude_example_prefix").and_then(|v| v.as_bool()) {
                std::env::set_var("NYASH_DISCOVER_EXCLUDE_EXAMPLE_PREFIX", if b {"1"} else {"0"});
            }
        // Namespace conflict detection across accumulated pending_modules (warn/strict)
        super::conflict_detector::detect_conflicts_from_modules(pending_modules)?;

        }
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
            if !alias_tbl.is_empty() && !crate::config::env::cli_quiet() { eprintln!("[deprecate] [modules.aliases] is deprecated; use [modules.overrides] instead"); }
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

    
    
    // [private] minimal: record patterns/options into env for pipeline enforcement
    if let Some(priv_tbl) = doc.get("private").and_then(|v| v.as_table()) {
        if let Some(arr) = priv_tbl.get("patterns").and_then(|v| v.as_array()) {
            let mut pats: Vec<String> = Vec::new();
            for e in arr.iter() { if let Some(s) = e.as_str() { let t = s.trim(); if !t.is_empty() { pats.push(t.to_string()); } } }
            if !pats.is_empty() {
                std::env::set_var("NYASH_PRIVATE_PATTERNS", pats.join(";"));
            }
        }
        if let Some(opts) = priv_tbl.get("options").and_then(|v| v.as_table()) {
            if let Some(mode) = opts.get("on_violation").and_then(|v| v.as_str()) { std::env::set_var("NYASH_PRIVATE_ON_VIOLATION", mode); }
            if let Some(diag) = opts.get("enable_diagnostics").and_then(|v| v.as_bool()) { std::env::set_var("NYASH_PRIVATE_DIAG", if diag {"1"} else {"0"}); }
        }
    }
// Directory-as-Namespace fallback (optional; env-gated)
    // When NYASH_USING_DIR_NS=1, scan apps/ for *.hako and append (ns->path) pairs as
    // lowest-precedence candidates. Duplicates are suppressed; conflicts may error in STRICT.
    if std::env::var("NYASH_USING_DIR_NS").ok().as_deref() == Some("1") {
        use std::collections::{HashMap as Map, HashSet};
        let root = std::env::var("NYASH_ROOT").ok().unwrap_or_else(|| ".".to_string());
        let apps = std::path::Path::new(&root).join("apps");
        if apps.exists() {
            let opts = crate::config::module_discovery::ModuleDiscoveryOptions::default();
            let auto = crate::config::module_discovery::discover_entries_under(&apps, &opts);
            // build maps of existing namespaces and their paths (for conflict detection)
            let mut existing: HashSet<String> = HashSet::new();
            let mut existing_map: Map<String, String> = Map::new();
            for (ns, p) in pending_modules.iter() { existing.insert(ns.clone()); existing_map.insert(ns.clone(), p.clone()); }
            // Accumulate multiplicities (ns -> {paths}) for conflict detection
            let mut multi: Map<String, HashSet<String>> = Map::new();
            for (ns, p) in pending_modules.iter() { multi.entry(ns.clone()).or_default().insert(p.clone()); }
            // Merge auto entries; if an ns already exists, record both existing and auto paths for strict conflict check
            for (ns, p) in auto.into_iter() {
                if !existing.contains(&ns) {
                    if let Some(ps) = p.to_str() { pending_modules.push((ns.clone(), ps.to_string())); multi.entry(ns).or_default().insert(ps.to_string()); }
                } else {
                    if let Some(ps) = p.to_str() {
                        multi.entry(ns.clone()).or_default().insert(ps.to_string());
                        if let Some(prev) = existing_map.get(&ns) { multi.entry(ns.clone()).or_default().insert(prev.clone()); }
                    }
                }
            }
            // Lightweight conflict check on the accumulated set (warn/strict)
            super::conflict_detector::detect_conflicts(&multi)?;
        }
    }
    // Final duplicate detection across all accumulated entries (always on; strict -> error)
    super::conflict_detector::detect_conflicts_from_modules(pending_modules)?;
    Ok(policy)
}


#[cfg(test)]
mod tests {

    #[test]
    fn e2e_workspace_modules_exports() {
        let dir = std::env::temp_dir().join(format!("nyash_workspace_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        std::fs::create_dir_all(&dir).unwrap();
        // Layout: apps/selfhost/vm/module.toml + boxes/mini_vm_entry.hako
        let mod_dir = dir.join("apps/selfhost/vm");
        std::fs::create_dir_all(mod_dir.join("boxes")).unwrap();
        std::fs::write(mod_dir.join("boxes/mini_vm_entry.hako"), "// stub").unwrap();
        let module_toml = r#"[module]
name = "selfhost.vm"
version = "1.0.0"

[exports]
entry = "boxes/mini_vm_entry.hako"
"#;
        std::fs::write(mod_dir.join("module.toml"), module_toml).unwrap();
        // Write hako.toml with workspace members
        let toml_text = r#"[modules.workspace]
members = ["apps/selfhost/vm/module.toml"]
"#;
        std::fs::write(dir.join("hako.toml"), toml_text).unwrap();
        std::env::set_var("NYASH_USING_TEST_FORCE_ENV_ROOT", "1");
        std::env::set_var("NYASH_ROOT", dir.to_str().unwrap());
        let mut using_paths = Vec::new();
        let mut pending_modules = Vec::new();
        let mut aliases = std::collections::HashMap::new();
        let mut packages = std::collections::HashMap::new();
        let _policy = super::populate_from_toml(&mut using_paths, &mut pending_modules, &mut aliases, &mut packages).unwrap();
        pending_modules.sort();
        assert!(pending_modules.iter().any(|(ns, p)| ns == "selfhost.vm.entry" && p.ends_with("apps/selfhost/vm/boxes/mini_vm_entry.hako")));
    }

    use super::*;
    use std::fs;
    #[test]
    fn e2e_flatten_modules_from_runner_env_root() {
        // Create temp dir
        let dir = std::env::temp_dir().join(format!("nyash_using_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        fs::create_dir_all(&dir).unwrap();
        // Write nyash.toml with nested modules
        let toml_text = r#"
[modules.a.b]
c = "x/path"
d = "y/path"
[modules.e]
f = "z/path"
"#;
        fs::write(dir.join("nyash.toml"), toml_text).unwrap();
        // Force resolver to use env root
        std::env::set_var("NYASH_USING_TEST_FORCE_ENV_ROOT", "1");
        std::env::set_var("NYASH_ROOT", dir.to_str().unwrap());
        let mut using_paths = Vec::new();
        let mut pending_modules = Vec::new();
        let mut aliases = std::collections::HashMap::new();
        let mut packages = std::collections::HashMap::new();
        let _policy = super::populate_from_toml(&mut using_paths, &mut pending_modules, &mut aliases, &mut packages).unwrap();
        pending_modules.sort();
        assert_eq!(pending_modules, vec![
            ("a.b.c".into(), "x/path".into()),
            ("a.b.d".into(), "y/path".into()),
            ("e.f".into(), "z/path".into()),
        ]);
    }
}
