//! Module discovery (spec v2, proposal)
//!
//! Default: convention over configuration — discover `apps/**/*.hako` and
//! turn paths into namespaces. hako.toml provides overrides/aliases.
//!
//! Note: this is a stub (non‑wired). The runner keeps using existing
//! resolver until we flip the switch. This file documents function shapes.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;

/// Convert a path under `apps/` into a dotted namespace.
/// Normalization rules (Dir-as-NS v2.2):
/// - Directory: replace '-' with '.' (e.g., `selfhost-compiler` → `selfhost.compiler`)
/// - File: strip `.hako`, then strip optional `_box` suffix (e.g., `json_minify_box.hako` → `json_minify`)
/// Example: apps/selfhost-compiler/pipeline_v2/json_minify_box.hako → selfhost.compiler.pipeline_v2.json_minify
pub fn path_to_namespace<P: AsRef<Path>>(apps_root: P, file: P) -> Option<String> {
    let root = apps_root.as_ref();
    let f = file.as_ref();
    let rel = f.strip_prefix(root).ok()?;
    // Module-first policy: prefer module manifests (hako_module.toml/module.toml)
    if crate::config::env::ns_policy_module_first() {
        if let Some(ns) = try_module_manifest_namespace(root, f) { return Some(ns); }
        // Fallback: simplified Dir-as-NS — join path with dots, only strip .hako and optional _box suffix
        let mut parts: Vec<String> = Vec::new();
        for c in rel.components() { parts.push(c.as_os_str().to_str()?.to_string()); }
        if parts.is_empty() { return None; }
        if let Some(last) = parts.last_mut() {
            if last.ends_with(".hako") { *last = last.trim_end_matches(".hako").to_string(); }
            if last.ends_with("_box") { *last = last.trim_end_matches("_box").to_string(); }
        }
        return Some(parts.join("."));
    }
    // path-first (default): legacy Dir-as-NS rules
    let mut parts: Vec<String> = Vec::new();
    for c in rel.components() {
        let s = c.as_os_str().to_str()?;
        parts.push(s.to_string());
    }
    if parts.is_empty() { return None; }
    for i in 0..(parts.len().saturating_sub(1)) {
        if parts[i].contains('-') { parts[i] = parts[i].replace('-', "."); }
    }
    if let Some(last) = parts.last_mut() {
        if last.ends_with(".hako") { *last = last.trim_end_matches(".hako").to_string(); }
        if last.ends_with("_box") { *last = last.trim_end_matches("_box").to_string(); }
    }
    Some(parts.join("."))
}

/// Try resolve namespace using nearest module manifest around the file.
fn try_module_manifest_namespace(root: &Path, file: &Path) -> Option<String> {
    let mut dir = file.parent()?;
    // Walk up to apps_root
    loop {
        let cand_hako = dir.join("hako_module.toml");
        let cand = if cand_hako.exists() { cand_hako } else { dir.join("module.toml") };
        if cand.exists() {
            if let Some(man) = crate::config::module_workspace::parse_module_toml(&cand) {
                // Match export path
                let base = cand.parent().unwrap_or(dir);
                let canon_file = std::fs::canonicalize(file).ok()?;
                for (key, rel) in man.exports.iter() {
                    let abs = std::fs::canonicalize(base.join(rel)).ok();
                    if let Some(a) = abs { if a == canon_file { return Some(format!("{}.{}", man.name, key)); } }
                }
            }
        }
        if dir == root { break; }
        if let Some(up) = dir.parent() { dir = up; } else { break; }
    }
    None
}

/// Very small recursive discovery without external crates.
/// Discovery options (simple heuristic excludes; no glob dependency).
#[derive(Clone, Debug)]
pub struct ModuleDiscoveryOptions {
    pub exclude_archive: bool,
    pub exclude_underscore_dirs: bool,
    pub exclude_test_prefix: bool,
    pub exclude_example_prefix: bool,
}

impl Default for ModuleDiscoveryOptions {
    fn default() -> Self {
        Self {
            exclude_archive: true,
            exclude_underscore_dirs: true,
            exclude_test_prefix: true,
            exclude_example_prefix: true,
        }
    }
}

fn should_exclude(path: &Path, opts: &ModuleDiscoveryOptions) -> bool {
    if opts.exclude_archive {
        if path.components().any(|c| c.as_os_str().to_str() == Some("archive")) { return true; }
    }
    if opts.exclude_underscore_dirs {
        for c in path.components() {
            if let Some(s) = c.as_os_str().to_str() {
                if s.starts_with('_') && path.is_dir() { return true; }
            }
        }
    }
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if opts.exclude_test_prefix && name.starts_with("test_") { return true; }
        if opts.exclude_example_prefix && name.starts_with("example_") { return true; }
    }
    false
}

pub fn discover_entries_under<P: AsRef<Path>>(apps_root: P, opts: &ModuleDiscoveryOptions) -> Vec<(String, PathBuf)> {
    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    fn walk(cur: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>, opts: &ModuleDiscoveryOptions) {
        if let Ok(rd) = fs::read_dir(cur) {
            for e in rd.flatten() {
                let p = e.path();
                if should_exclude(&p, opts) { continue; }
                if p.is_dir() {
                    walk(&p, root, out, opts);
                } else if let Some(ext) = p.extension() { if ext == "hako" {
                    if let Some(ns) = super::module_discovery::path_to_namespace(root, &p) {
                        out.push((ns, p.clone()));
                    }
                }}
            }
        }
    }
    let root = apps_root.as_ref();
    walk(root, root, &mut entries, opts);
    entries
}

pub fn discover_hako_under<P: AsRef<Path>>(apps_root: P) -> HashMap<String, PathBuf> {
    let opts = ModuleDiscoveryOptions::default();
    let entries = discover_entries_under(apps_root, &opts);
    let mut map = HashMap::new();
    for (ns, p) in entries { map.insert(ns, p); }
    map
}

/// Merge with overrides and aliases. Discovery→overrides→aliases の順に適用。
pub fn merge_with_overrides(
    discovered: HashMap<String, PathBuf>,
    overrides: &HashMap<String, PathBuf>,
    aliases: &HashMap<String, String>,
) -> HashMap<String, PathBuf> {
    let mut out = discovered;
    for (k, v) in overrides.iter() { out.insert(k.clone(), v.clone()); }
    for (alias, target) in aliases.iter() {
        if let Some(real) = out.get(target) { out.insert(alias.clone(), real.clone()); }
    }
    out
}

/// Detect namespace conflicts (same namespace claimed by different files).
pub fn detect_conflicts(entries: &[(String, PathBuf)]) -> Vec<String> {
    let mut multi: HashMap<String, HashSet<PathBuf>> = HashMap::new();
    for (ns, p) in entries.iter() {
        multi.entry(ns.clone()).or_default().insert(p.clone());
    }
    let mut out = Vec::new();
    for (ns, set) in multi.into_iter() {
        if set.len() > 1 {
            let mut paths: Vec<String> = set.into_iter().map(|p| p.display().to_string()).collect();
            paths.sort();
            out.push(format!("conflict: {} ← [{}]", ns, paths.join(", ")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_to_namespace_basic() {
        let ns = path_to_namespace("apps", "apps/selfhost/vm/boxes/mir_vm_min.hako").unwrap();
        assert_eq!(ns, "selfhost.vm.boxes.mir_vm_min");
    }
}
