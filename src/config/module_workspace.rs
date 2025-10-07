//! Module workspace reader (module.toml) — minimal MVP
//!
//! Reads `module.toml` alongside a module directory and exposes public exports
//! as (namespace → path) pairs. Private exports are ignored at this layer.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ModuleManifest {
    pub name: String,
    pub version: Option<String>,
    pub exports: HashMap<String, PathBuf>, // key -> relative path
    pub private: HashMap<String, PathBuf>, // not exported; informational only
    pub dependencies: HashMap<String, String>,
}

pub fn parse_module_toml(path: &Path) -> Option<ModuleManifest> {
    let text = std::fs::read_to_string(path).ok()?;
    let root: toml::Value = toml::from_str(&text).ok()?;
    let tbl = root.as_table()?;
    let module_tbl = tbl.get("module").and_then(|v| v.as_table())?;
    let name = module_tbl.get("name").and_then(|v| v.as_str())?.to_string();
    let version = module_tbl.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
    let mut exports: HashMap<String, PathBuf> = HashMap::new();
    let mut private: HashMap<String, PathBuf> = HashMap::new();
    let mut dependencies: HashMap<String, String> = HashMap::new();
    if let Some(exp_tbl) = tbl.get("exports").and_then(|v| v.as_table()) {
        for (k, v) in exp_tbl.iter() {
            if let Some(s) = v.as_str() {
                exports.insert(k.to_string(), PathBuf::from(s));
            }
        }
    }
    if let Some(prv_tbl) = tbl.get("private").and_then(|v| v.as_table()) {
        for (k, v) in prv_tbl.iter() {
            if let Some(s) = v.as_str() {
                private.insert(k.to_string(), PathBuf::from(s));
            }
        }
    }
    if let Some(dep_tbl) = tbl.get("dependencies").and_then(|v| v.as_table()) {
        for (k, v) in dep_tbl.iter() {
            if let Some(s) = v.as_str() { dependencies.insert(k.to_string(), s.to_string()); }
        }
    }
    Some(ModuleManifest { name, version, exports, private, dependencies })
}

/// Expand simple wildcard like `apps/*/module.toml` (one `*` segment) into concrete paths.
pub fn expand_members_pattern(pat: &str) -> Vec<PathBuf> {
    // Support simple globs: '*', '**', and '?' (filename only)
    // Strategy: split on "**" first (any-depth), then on single '*' (one-level), and match '?' at filename level.
    let pat = pat.to_string();
    if pat.contains("**") {
        let parts: Vec<&str> = pat.split("**").collect();
        let (pre, suf) = (Path::new(parts[0]), parts.get(1).copied().unwrap_or(""));
        let mut out = Vec::new();
        fn walk(cur: &Path, suf: &str, out: &mut Vec<PathBuf>) {
            let cand = Path::new(&format!("{}{}", cur.display(), suf)).to_path_buf();
            if cand.exists() { out.push(cand); }
            if let Ok(rd) = std::fs::read_dir(cur) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() { walk(&p, suf, out); }
                }
            }
        }
        if pre.exists() { walk(pre, suf, &mut out); }
        return out;
    }
    if let Some((prefix, suffix)) = pat.split_once('*') {
        let (pre, suf) = (Path::new(prefix), suffix);
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(pre) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    let candidate = Path::new(&format!("{}{}", p.display(), suf)).to_path_buf();
                    if candidate.exists() { out.push(candidate); }
                } else if suf.contains('?') {
                    // filename pattern match with '?'
                    if let Some(file) = candidate_from_suffix(&p, suf) { if file.exists() { out.push(file); } }
                }
            }
        }
        return out;
    }
    vec![PathBuf::from(pat)]
}

fn candidate_from_suffix(base: &Path, suffix: &str) -> Option<PathBuf> {
    // If suffix is like "/hako_module.toml" just append; if it contains '?', try to match filename
    if !suffix.contains('?') {
        return Some(Path::new(&format!("{}{}", base.display(), suffix)).to_path_buf());
    }
    // naive: only support trailing filename with '?'
    let suf = suffix.trim_start_matches('/');
    let parent = base.parent().unwrap_or(base);
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            let p = e.path();
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if wildcard_match(name, suf) { return Some(p); }
            }
        }
    }
    None
}

fn wildcard_match(name: &str, pat: &str) -> bool {
    if name.len() != pat.len() { return false; }
    name.chars().zip(pat.chars()).all(|(c, pc)| pc == '?' || pc == c)
}

/// Parse minimal metadata from a `module.hako` file.
/// Heuristic: scan for lines like `name: "..."`, `version: "..."`, and an `exports: map({ ... })`
/// with entries of the form `key: "path"` separated by commas.
pub fn parse_module_hako(path: &Path) -> Option<ModuleManifest> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    let mut exports: HashMap<String, PathBuf> = HashMap::new();
    for line in text.lines() {
        let l = line.trim();
        if name.is_none() {
            if let Some(pos) = l.find("name:") {
                let rest = &l[pos+5..];
                if let Some(q1) = rest.find('"') { if let Some(q2) = rest[q1+1..].find('"') {
                    name = Some(rest[q1+1..q1+1+q2].to_string());
                }}
            }
        }
        if version.is_none() {
            if let Some(pos) = l.find("version:") {
                let rest = &l[pos+8..];
                if let Some(q1) = rest.find('"') { if let Some(q2) = rest[q1+1..].find('"') {
                    version = Some(rest[q1+1..q1+1+q2].to_string());
                }}
            }
        }
        // Parse inside an exports: map({ ... }) block in a simplistic way
        if l.starts_with("exports:") {
            // Collect until closing '}'
            // This is a best-effort parser; safe for list-modules preview only.
            let mut buf = String::new();
            buf.push_str(l);
            // Append following lines up to '}'
            // (Stop early if brace closes on same line)
            if !l.contains('}') {
                for line2 in text.lines() {
                    buf.push_str("\n"); buf.push_str(line2);
                    if line2.contains('}') { break; }
                }
            }
            // Extract pairs like key: "path"
            for ent in buf.split(',') {
                let e = ent.trim();
                if let Some(col) = e.find(':') {
                    let k = e[..col].trim().trim_matches(|c: char| c == '"' || c == '{');
                    let vraw = e[col+1..].trim();
                    if let Some(q1) = vraw.find('"') { if let Some(q2) = vraw[q1+1..].find('"') {
                        let v = &vraw[q1+1..q1+1+q2];
                        if !k.is_empty() && !v.is_empty() { exports.insert(k.to_string(), PathBuf::from(v)); }
                    }}
                }
            }
        }
    }
    let name = name?;
    Some(ModuleManifest { name, version, exports, private: HashMap::new(), dependencies: HashMap::new() })
}

/// Build dependency adjacency list from manifests (name -> deps)
pub fn build_dep_graph(manifests: &[ModuleManifest]) -> HashMap<String, Vec<String>> {
    let mut g: HashMap<String, Vec<String>> = HashMap::new();
    for m in manifests.iter() {
        let mut deps = Vec::new();
        for (k, _) in m.dependencies.iter() { deps.push(k.clone()); }
        g.insert(m.name.clone(), deps);
    }
    g
}

/// Detect cycles; returns list of cycles as ordered node names (start→...→start)
pub fn detect_cycles_from_graph(graph: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    fn dfs(u: &str, graph: &HashMap<String, Vec<String>>, temp: &mut HashSet<String>, perm: &mut HashSet<String>, stack: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        if perm.contains(u) { return; }
        if temp.contains(u) {
            // found a cycle; extract from stack
            if let Some(pos) = stack.iter().position(|x| x == u) {
                let mut cyc = stack[pos..].to_vec();
                cyc.push(u.to_string());
                out.push(cyc);
            }
            return;
        }
        temp.insert(u.to_string());
        stack.push(u.to_string());
        if let Some(neis) = graph.get(u) {
            for v in neis.iter() { dfs(v, graph, temp, perm, stack, out); }
        }
        stack.pop();
        temp.remove(u);
        perm.insert(u.to_string());
    }
    let mut out = Vec::new();
    let mut temp = HashSet::new();
    let mut perm = HashSet::new();
    for u in graph.keys() {
        if !perm.contains(u) {
            let mut stack = Vec::new();
            dfs(u, graph, &mut temp, &mut perm, &mut stack, &mut out);
        }
    }
    out
}
