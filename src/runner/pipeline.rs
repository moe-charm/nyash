/*!
 * Runner pipeline helpers — using/modules/env pre-processing
 *
 * Extracts the early-phase setup from runner/mod.rs:
 *  - load nyash.toml [modules] and [using.paths]
 *  - merge with defaults and env overrides
 *  - expose context (using_paths, pending_modules) for downstream resolution
 */

use super::*;
use std::collections::HashMap;

/// Using/module resolution context accumulated from config/env/nyash.toml
pub(super) struct UsingContext {
    pub using_paths: Vec<String>,
    pub pending_modules: Vec<(String, String)>,
    pub aliases: std::collections::HashMap<String, String>,
}

impl NyashRunner {
    /// Initialize using/module context from defaults, nyash.toml and env
    pub(super) fn init_using_context(&self) -> UsingContext {
        let mut using_paths: Vec<String> = Vec::new();
        let mut pending_modules: Vec<(String, String)> = Vec::new();
        let mut aliases: std::collections::HashMap<String, String> = std::collections::HashMap::new();

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
                    // Optional: [aliases] table maps short name -> path or namespace token
                    if let Some(alias_tbl) = doc.get("aliases").and_then(|v| v.as_table()) {
                        for (k, v) in alias_tbl.iter() {
                            if let Some(target) = v.as_str() {
                                aliases.insert(k.to_string(), target.to_string());
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
        // Env aliases: comma-separated k=v pairs
        if let Ok(raw) = std::env::var("NYASH_ALIASES") {
            for ent in raw.split(',') {
                if let Some((k,v)) = ent.split_once('=') {
                    let k = k.trim(); let v = v.trim();
                    if !k.is_empty() && !v.is_empty() { aliases.insert(k.to_string(), v.to_string()); }
                }
            }
        }

        UsingContext { using_paths, pending_modules, aliases }
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
    aliases: &HashMap<String, String>,
    context_dir: Option<&std::path::Path>,
    strict: bool,
    verbose: bool,
) -> Result<String, String> {
    if is_path { return Ok(tgt.to_string()); }
    let trace = verbose || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1");
    let idx = super::box_index::get_box_index();
    let mut strict_effective = strict || idx.plugins_require_prefix_global;
    if std::env::var("NYASH_PLUGIN_REQUIRE_PREFIX").ok().as_deref() == Some("1") { strict_effective = true; }
    let meta_for_target = idx.plugin_meta_by_box.get(tgt).cloned();
    let mut require_prefix_target = meta_for_target.as_ref().map(|m| m.require_prefix).unwrap_or(false);
    if let Some(m) = &meta_for_target { if !m.expose_short_names { require_prefix_target = true; } }
    let mut is_plugin_short = meta_for_target.is_some();
    if !is_plugin_short {
        is_plugin_short = idx.plugin_boxes.contains(tgt) || super::box_index::BoxIndex::is_known_plugin_short(tgt);
    }
    if (strict_effective || require_prefix_target) && is_plugin_short && !tgt.contains('.') {
        let mut msg = format!("plugin short name '{}' requires prefix", tgt);
        if let Some(meta) = &meta_for_target {
            if let Some(pref) = &meta.prefix {
                msg.push_str(&format!(" (use '{}.{}')", pref, tgt));
            }
        }
        return Err(msg);
    }
    let key = {
        let base = context_dir.and_then(|p| p.to_str()).unwrap_or("");
        format!("{}|{}|{}|{}", tgt, base, strict as i32, using_paths.join(":"))
    };
    if let Some(hit) = crate::runner::box_index::cache_get(&key) {
        if trace { eprintln!("[using/cache] '{}' -> '{}'", tgt, hit); }
        return Ok(hit);
    }
    // Resolve aliases early (provided map)
    if let Some(v) = aliases.get(tgt) {
        if trace { eprintln!("[using/resolve] alias '{}' -> '{}'", tgt, v); }
        crate::runner::box_index::cache_put(&key, v.clone());
        return Ok(v.clone());
    }
    // Also consult env aliases
    if let Ok(raw) = std::env::var("NYASH_ALIASES") {
        for ent in raw.split(',') {
            if let Some((k,v)) = ent.split_once('=') {
                if k.trim() == tgt {
                    let out = v.trim().to_string();
                    if trace { eprintln!("[using/resolve] env-alias '{}' -> '{}'", tgt, out); }
                    crate::runner::box_index::cache_put(&key, out.clone());
                    return Ok(out);
                }
            }
        }
    }
    // 1) modules mapping
    if let Some((_, p)) = modules.iter().find(|(n, _)| n == tgt) {
        let out = p.clone();
        if trace { eprintln!("[using/resolve] modules '{}' -> '{}'", tgt, out); }
        crate::runner::box_index::cache_put(&key, out.clone());
        return Ok(out);
    }
    // 2) build candidate list: relative then using-paths
    let rel = tgt.replace('.', "/") + ".nyash";
    let mut cand: Vec<String> = Vec::new();
    if let Some(dir) = context_dir { let c = dir.join(&rel); if c.exists() { cand.push(c.to_string_lossy().to_string()); } }
    for base in using_paths {
        let c = std::path::Path::new(base).join(&rel);
        if c.exists() { cand.push(c.to_string_lossy().to_string()); }
    }
    if cand.is_empty() {
        if trace {
            // Try suggest candidates by leaf across bases (apps/lib/.)
            let leaf = tgt.split('.').last().unwrap_or(tgt);
            let mut cands: Vec<String> = Vec::new();
            suggest_in_base("apps", leaf, &mut cands);
            if cands.len() < 5 { suggest_in_base("lib", leaf, &mut cands); }
            if cands.len() < 5 { suggest_in_base(".", leaf, &mut cands); }
            if cands.is_empty() {
                eprintln!("[using] unresolved '{}' (searched: rel+paths)", tgt);
            } else {
                eprintln!("[using] unresolved '{}' (searched: rel+paths) candidates: {}", tgt, cands.join(", "));
            }
        }
        return Ok(tgt.to_string());
    }
    if cand.len() > 1 && strict {
        return Err(format!("ambiguous using '{}': {}", tgt, cand.join(", ")));
    }
    let out = cand.remove(0);
    if trace { eprintln!("[using/resolve] '{}' -> '{}'", tgt, out); }
    crate::runner::box_index::cache_put(&key, out.clone());
    Ok(out)
}

/// Lint: enforce "fields must be at the top of box" rule.
/// - Warns by default (when verbose); when `strict` is true, returns Err on any violation.
pub(super) fn lint_fields_top(code: &str, strict: bool, verbose: bool) -> Result<(), String> {
    let mut brace: i32 = 0;
    let mut in_box = false;
    let mut box_depth: i32 = 0;
    let mut seen_method = false;
    let mut cur_box: String = String::new();
    let mut violations: Vec<(usize, String, String)> = Vec::new(); // (line, field, box)

    for (idx, line) in code.lines().enumerate() {
        let lno = idx + 1;
        let pre_brace = brace;
        let trimmed = line.trim();
        // Count braces for this line
        let opens = line.matches('{').count() as i32;
        let closes = line.matches('}').count() as i32;

        // Enter box on same-line K&R style: `box Name {` or `static box Name {`
        if !in_box && trimmed.starts_with("box ") || trimmed.starts_with("static box ") {
            // capture name
            let mut name = String::new();
            let after = if let Some(rest) = trimmed.strip_prefix("static box ") { rest } else { trimmed.strip_prefix("box ").unwrap_or("") };
            for ch in after.chars() {
                if ch.is_alphanumeric() || ch == '_' { name.push(ch); } else { break; }
            }
            // require K&R brace on same line to start tracking
            if opens > 0 {
                in_box = true;
                cur_box = name;
                box_depth = pre_brace + 1; // assume one level for box body
                seen_method = false;
            }
        }

        if in_box {
            // Top-level inside box only
            if pre_brace == box_depth {
                // Skip empty/comment lines
                if !trimmed.is_empty() && !trimmed.starts_with("//") {
                    // Detect method: name(args) {
                    let is_method = {
                        // starts with identifier then '(' and later '{'
                        let mut it = trimmed.chars();
                        let mut ident = String::new();
                        while let Some(c) = it.next() { if c.is_whitespace() { continue; } if c.is_alphabetic() || c=='_' { ident.push(c); break; } else { break; } }
                        while let Some(c) = it.next() { if c.is_alphanumeric() || c=='_' { ident.push(c); } else { break; } }
                        trimmed.contains('(') && trimmed.ends_with('{') && !ident.is_empty()
                    };
                    if is_method { seen_method = true; }

                    // Detect field: ident ':' Type (rough heuristic)
                    let is_field = {
                        let parts: Vec<&str> = trimmed.split(':').collect();
                        if parts.len() == 2 {
                            let lhs = parts[0].trim();
                            let rhs = parts[1].trim();
                            let lhs_ok = !lhs.is_empty() && lhs.chars().next().map(|c| c.is_alphabetic() || c=='_').unwrap_or(false);
                            let rhs_ok = !rhs.is_empty() && rhs.chars().next().map(|c| c.is_alphabetic() || c=='_').unwrap_or(false);
                            lhs_ok && rhs_ok && !trimmed.contains('(') && !trimmed.contains(')')
                        } else { false }
                    };
                    if is_field && seen_method {
                        violations.push((lno, trimmed.to_string(), cur_box.clone()));
                    }
                }
            }
            // Exit box when closing brace reduces depth below box_depth
            let post_brace = pre_brace + opens - closes;
            if post_brace < box_depth { in_box = false; cur_box.clear(); }
        }

        // Update brace after processing
        brace += opens - closes;
    }

    if violations.is_empty() {
        return Ok(());
    }
    if strict {
        // Compose error message
        let mut msg = String::from("Field declarations must appear at the top of box. Violations:\n");
        for (lno, fld, bx) in violations.iter().take(10) {
            msg.push_str(&format!("  line {} in box {}: '{}" , lno, if bx.is_empty(){"<unknown>"} else {bx}, fld));
            msg.push_str("'\n");
        }
        if violations.len() > 10 { msg.push_str(&format!("  ... and {} more\n", violations.len()-10)); }
        return Err(msg);
    }
    if verbose || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
        for (lno, fld, bx) in violations {
            eprintln!("[lint] fields-top: line {} in box {} -> {}", lno, if bx.is_empty(){"<unknown>"} else {&bx}, fld);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn plugin_meta_requires_prefix_even_when_relaxed() {
        let dir = tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir.path()).expect("chdir");
        let toml = r#"
[plugins]
require_prefix = false

[plugins."test-plugin"]
prefix = "test"
require_prefix = true
expose_short_names = false
boxes = ["ArrayBox"]
"#;
        std::fs::write("nyash.toml", toml).expect("write nyash.toml");
        crate::runner::box_index::refresh_box_index();
        crate::runner::box_index::cache_clear();

        let res = resolve_using_target(
            "ArrayBox",
            false,
            &[],
            &[],
            &HashMap::new(),
            None,
            false,
            false,
        );
        assert!(res.is_err(), "expected prefix enforcement");
        let err = res.err().unwrap();
        assert!(err.contains("requires prefix"));
        assert!(err.contains("test."));

        std::env::set_current_dir(old).expect("restore cwd");
        crate::runner::box_index::refresh_box_index();
        crate::runner::box_index::cache_clear();
    }
}
