//! Simple ModuleRegistry for Phase 1 diagnostics
//! Collects published symbols (top-level `static box Name`) from using targets.

use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static CACHE: Lazy<Mutex<HashMap<String, HashSet<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Return candidate using names whose exported symbols contain `symbol`.
/// Uses runtime::modules_registry snapshot (name -> path token) and scans files.
pub fn suggest_using_for_symbol(symbol: &str) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let snap = crate::runtime::modules_registry::snapshot_names_and_strings();
    let wanted = symbol.trim();
    if wanted.is_empty() { return results; }

    for (name, path_token) in snap {
        // Skip builtin/dylib marker tokens
        if path_token.starts_with("builtin:") || path_token.starts_with("dylib:") {
            continue;
        }
        // Ensure cache for this key
        let mut guard = CACHE.lock().ok();
        let set = guard
            .as_mut()
            .map(|m| m.entry(name.clone()).or_insert_with(HashSet::new))
            .expect("module cache poisoned");
        if set.is_empty() {
            if let Some(p) = resolve_path(&path_token) {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    let syms = scan_static_boxes(&content);
                    for s in syms { set.insert(s); }
                }
            }
        }
        if set.contains(wanted) {
            results.push(name);
        }
    }
    results.sort();
    results.dedup();
    results
}

fn resolve_path(token: &str) -> Option<std::path::PathBuf> {
    let mut p = std::path::PathBuf::from(token);
    if p.is_relative() {
        if let Ok(abs) = std::fs::canonicalize(&p) { p = abs; }
    }
    if p.exists() { Some(p) } else { None }
}

fn scan_static_boxes(content: &str) -> Vec<String> {
    // Very simple lexer: find lines like `static box Name {`
    // Avoid matching inside comments by skipping lines that start with //
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with("//") { continue; }
        if let Some(rest) = t.strip_prefix("static box ") {
            let mut name = String::new();
            for ch in rest.chars() {
                if ch.is_ascii_alphanumeric() || ch == '_' { name.push(ch); } else { break; }
            }
            if !name.is_empty() { out.push(name); }
        }
    }
    out
}
