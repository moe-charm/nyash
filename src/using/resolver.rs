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
    // Locate hako.toml/nyash.toml/hakorune.toml relative to CWD; fallback to *_ROOT if present.
    let mut chosen: Option<std::path::PathBuf> = None;
    let force_env_root = std::env::var("NYASH_USING_TEST_FORCE_ENV_ROOT").ok().as_deref() == Some("1");
    // Prefer CWD hako.toml unless tests force env-root mode
    if !force_env_root {
        for name in ["hako.toml", "nyash.toml", "hakorune.toml"] {
            let p = std::path::Path::new(name);
            if p.exists() { chosen = Some(p.to_path_buf()); break; }
        }
    }
    if chosen.is_none() {
        // Try roots: NYASH_ROOT/HAKO_ROOT/HAKU_ROOT/HRN_ROOT
        if let Some(root) = std::env::var("NYASH_ROOT").ok()
            .or_else(|| std::env::var("HAKO_ROOT").ok())
            .or_else(|| std::env::var("HAKU_ROOT").ok())
            .or_else(|| std::env::var("HRN_ROOT").ok())
        {
            for name in ["hako.toml", "nyash.toml", "hakorune.toml"] {
                let p = std::path::Path::new(&root).join(name);
                if p.exists() { chosen = Some(p); break; }
            }
        }
    }
    let path_opt = chosen.as_ref().map(|p| p.as_path());
    let path = if let Some(p) = path_opt { p } else { std::path::Path::new("nyash.toml") };
    if crate::config::env::resolve_trace() {
        if path.exists() {
            if !crate::config::env::cli_quiet() { eprintln!("[using] toml: using {:?}", std::fs::canonicalize(path).ok()); }
        }
        // One-line hint about search priority and root envs considered (for field debugging)
        if !crate::config::env::cli_quiet() {
            let root = std::env::var("NYASH_ROOT").ok()
                .or_else(|| std::env::var("HAKO_ROOT").ok())
                .or_else(|| std::env::var("HAKU_ROOT").ok())
                .or_else(|| std::env::var("HRN_ROOT").ok());
            eprintln!("[using] toml search: [./hako.toml > ./nyash.toml > ./hakorune.toml] or roots {:?}", root);
        }
    }
    if !path.exists() {
        return Ok(policy);
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| UsingError::ReadToml(e.to_string()))?;
    let doc = match toml::from_str::<toml::Value>(&text) {
        Ok(v) => v,
        Err(e) => {
            // When tracing is enabled, surface TOML parse issues to help diagnose
            // missing [using]/[modules] during development.
            if crate::config::env::resolve_trace() && !crate::config::env::cli_quiet() {
                eprintln!(
                    "[using] toml parse error at {:?}: {}",
                    std::fs::canonicalize(path).ok(),
                    e
                );
            }
            return Ok(policy);
        }
    };
    // One-line summary (trace). Detailed dumps only when NYASH_RESOLVE_DEBUG=1
    if crate::config::env::resolve_trace() {
        let mut mods_count = 0usize;
        if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) { mods_count = mods.len(); }
        let using_tbl = doc.get("using").and_then(|v| v.as_table());
        let paths_count = using_tbl.and_then(|t| t.get("paths").and_then(|v| v.as_array())).map(|a| a.len()).unwrap_or(0);
        let aliases_count = using_tbl.and_then(|t| t.get("aliases").and_then(|v| v.as_table())).map(|t| t.len()).unwrap_or(0);
        let pkgs_count = using_tbl.map(|t| t.len()).unwrap_or(0).saturating_sub(2 /*paths+aliases*/);
        if !crate::config::env::cli_quiet() {
            eprintln!("[using] loaded: {:?}; modules:{} paths:{} aliases:{} packages:{}", std::fs::canonicalize(path).ok(), mods_count, paths_count, aliases_count, pkgs_count);
        }
        if std::env::var("NYASH_RESOLVE_DEBUG").ok().as_deref() == Some("1") {
            if let Some(tbl) = doc.as_table() {
                let keys: Vec<_> = tbl.keys().cloned().collect();
                if !crate::config::env::cli_quiet() { eprintln!("[using] toml: root keys = {:?}", keys); }
            }
        }
    }

    // [modules] table flatten: supports nested namespaces (a.b.c = "path")
    if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) {
        // main table
        let mut base = crate::common::using_core::flatten_modules_table(mods);
        pending_modules.append(&mut base);
        // [modules.workspace] — load module.toml manifests and export public entries
        if let Some(ws_tbl) = mods.get("workspace").and_then(|v| v.as_table()) {
            if let Some(arr) = ws_tbl.get("members").and_then(|v| v.as_array()) {
                let mut ws_manifests: Vec<crate::config::module_workspace::ModuleManifest> = Vec::new();
                for m in arr.iter() {
                    if let Some(raw) = m.as_str() {
                        // Accept either a module.toml path or a directory containing it; expand simple patterns like apps/*/module.toml
                        let mut cand_paths: Vec<std::path::PathBuf> = Vec::new();
                        if raw.contains('*') {
                            cand_paths.extend(crate::config::module_workspace::expand_members_pattern(raw));
                        } else {
                            let p = std::path::Path::new(raw);
                            if p.is_dir() {
                                // Prefer hako_module.toml; accept legacy module.toml
                                cand_paths.insert(cand_paths.len(), p.join("hako_module.toml"));
                                cand_paths.insert(cand_paths.len(), p.join("module.toml"));
                            } else { cand_paths.push(p.to_path_buf()); }
                        }
                        for mp in cand_paths.into_iter() {
                            if !mp.exists() { continue; }
                            if let Some(man) = crate::config::module_workspace::parse_module_toml(&mp) {
                                ws_manifests.push(man.clone());
                                let base_ns = man.name;
                                let base_dir = mp.parent().unwrap_or(std::path::Path::new("."));
                                for (key, rel) in man.exports.into_iter() {
                                    let ns = format!("{}.{}", base_ns, key);
                                    let sp = base_dir.join(rel);
                                    if let Some(sp_s) = sp.to_str() {
                                        pending_modules.push((ns, sp_s.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
                // Cycle detection (warn only for now)
                let g = crate::config::module_workspace::build_dep_graph(&ws_manifests);
                let cycles = crate::config::module_workspace::detect_cycles_from_graph(&g);
                if !cycles.is_empty() {
                    // Emit diagnostics JSON + optional human-readable
                    for cyc in cycles.iter() {
                        eprintln!("{}", crate::common::diagnostics::modules_error::cycle(cyc));
                        if !crate::config::env::cli_quiet() { eprintln!("[deps] cycle: {}", cyc.join(" -> ")); }
                    }
                    let strict = std::env::var("NYASH_USING_CHECKS_STRICT").ok().as_deref() == Some("1");
                    if strict {
                        let joined = cycles.get(0).map(|c| c.join(" -> ")).unwrap_or_else(|| "cycle".to_string());
                        return Err(crate::using::errors::UsingError::Cycle(joined));
                    }
                }
            }
        }
        // [modules.overrides] takes precedence (append; later wins in map usage)
        if let Some(ovr) = mods.get("overrides").and_then(|v| v.as_table()) {
            let mut v = crate::common::using_core::flatten_modules_table(ovr);
            pending_modules.append(&mut v);
        }
        // [modules.aliases] injects alias redirects (recorded in aliases map)
        if let Some(alias_tbl) = mods.get("aliases").and_then(|v| v.as_table()) {
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
