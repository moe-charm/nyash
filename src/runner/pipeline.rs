/*!
 * Runner pipeline helpers — using/modules/env pre-processing
 *
 * Extracts the early-phase setup from runner/mod.rs:
 *  - load nyash.toml [modules] and [using.paths]
 *  - merge with defaults and env overrides
 *  - expose context (using_paths, pending_modules) for downstream resolution
 */

use super::*;

/// Using/module resolution context accumulated from config/env/nyash.toml
pub(super) struct UsingContext {
    pub using_paths: Vec<String>,
    pub pending_modules: Vec<(String, String)>,
}

impl NyashRunner {
    /// Initialize using/module context from defaults, nyash.toml and env
    pub(super) fn init_using_context(&self) -> UsingContext {
        let mut using_paths: Vec<String> = Vec::new();
        let mut pending_modules: Vec<(String, String)> = Vec::new();

        // Defaults
        using_paths.extend(["apps", "lib", "."].into_iter().map(|s| s.to_string()));

        // nyash.toml: [modules] and [using.paths]
        if std::path::Path::new("nyash.toml").exists() {
            if let Ok(text) = std::fs::read_to_string("nyash.toml") {
                if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                    if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) {
                        for (k, v) in mods.iter() {
                            if let Some(path) = v.as_str() {
                                pending_modules.push((k.to_string(), path.to_string()));
                            }
                        }
                    }
                    if let Some(using_tbl) = doc.get("using").and_then(|v| v.as_table()) {
                        if let Some(paths_arr) = using_tbl.get("paths").and_then(|v| v.as_array()) {
                            for p in paths_arr {
                                if let Some(s) = p.as_str() {
                                    let s = s.trim();
                                    if !s.is_empty() { using_paths.push(s.to_string()); }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Env overrides: modules and using paths
        if let Ok(ms) = std::env::var("NYASH_MODULES") {
            for ent in ms.split(',') {
                if let Some((k, v)) = ent.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    if !k.is_empty() && !v.is_empty() {
                        pending_modules.push((k.to_string(), v.to_string()));
                    }
                }
            }
        }
        if let Ok(p) = std::env::var("NYASH_USING_PATH") {
            for s in p.split(':') {
                let s = s.trim();
                if !s.is_empty() { using_paths.push(s.to_string()); }
            }
        }

        UsingContext { using_paths, pending_modules }
    }
}

/// Suggest candidate files by leaf name within limited bases (apps/lib/.)
pub(super) fn suggest_in_base(base: &str, leaf: &str, out: &mut Vec<String>) {
    use std::fs;
    fn walk(dir: &std::path::Path, leaf: &str, out: &mut Vec<String>, depth: usize) {
        if depth == 0 || out.len() >= 5 { return; }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let path = e.path();
                if path.is_dir() {
                    walk(&path, leaf, out, depth - 1);
                    if out.len() >= 5 { return; }
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "nyash" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem == leaf {
                                out.push(path.to_string_lossy().to_string());
                                if out.len() >= 5 { return; }
                            }
                        }
                    }
                }
            }
        }
    }
    let p = std::path::Path::new(base);
    walk(p, leaf, out, 4);
}

/// Resolve a using target according to priority: modules > relative > using-paths
/// Returns Ok(resolved_path_or_token). On strict mode, ambiguous matches cause error.
pub(super) fn resolve_using_target(
    tgt: &str,
    is_path: bool,
    modules: &[(String, String)],
    using_paths: &[String],
    context_dir: Option<&std::path::Path>,
    strict: bool,
    verbose: bool,
) -> Result<String, String> {
    if is_path { return Ok(tgt.to_string()); }
    // 1) modules mapping
    if let Some((_, p)) = modules.iter().find(|(n, _)| n == tgt) { return Ok(p.clone()); }
    // 2) build candidate list: relative then using-paths
    let rel = tgt.replace('.', "/") + ".nyash";
    let mut cand: Vec<String> = Vec::new();
    if let Some(dir) = context_dir { let c = dir.join(&rel); if c.exists() { cand.push(c.to_string_lossy().to_string()); } }
    for base in using_paths {
        let c = std::path::Path::new(base).join(&rel);
        if c.exists() { cand.push(c.to_string_lossy().to_string()); }
    }
    if cand.is_empty() {
        if verbose { eprintln!("[using] unresolved '{}' (searched: rel+paths)", tgt); }
        return Ok(tgt.to_string());
    }
    if cand.len() > 1 && strict {
        return Err(format!("ambiguous using '{}': {}", tgt, cand.join(", ")));
    }
    Ok(cand.remove(0))
}
