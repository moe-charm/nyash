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
use crate::using::spec::{UsingPackage, PackageKind};

/// Using/module resolution context accumulated from config/env/nyash.toml
pub(super) struct UsingContext {
    pub using_paths: Vec<String>,
    pub pending_modules: Vec<(String, String)>,
    pub aliases: std::collections::HashMap<String, String>,
    pub packages: std::collections::HashMap<String, UsingPackage>,
}

impl NyashRunner {
    /// Initialize using/module context from defaults, nyash.toml and env
    pub(super) fn init_using_context(&self) -> UsingContext {
        let mut using_paths: Vec<String> = Vec::new();
        let mut pending_modules: Vec<(String, String)> = Vec::new();
        let mut aliases: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut packages: std::collections::HashMap<String, UsingPackage> =
            std::collections::HashMap::new();

        // Defaults
        using_paths.extend(["apps", "lib", "."].into_iter().map(|s| s.to_string()));

        // nyash.toml: delegate to using resolver (keeps existing behavior)
        let _ = crate::using::resolver::populate_from_toml(
            &mut using_paths,
            &mut pending_modules,
            &mut aliases,
            &mut packages,
        );
        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
            use crate::runner::trace::log as tlog;
            if !crate::config::env::cli_quiet() {
                tlog(format!("[using] ctx: paths={:?}", using_paths));
                tlog(format!("[using] ctx: aliases={:?}", aliases));
                tlog(format!("[using] ctx: packages={:?}", packages.keys().collect::<Vec<_>>()));
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
                if !s.is_empty() {
                    using_paths.push(s.to_string());
                }
            }
        }
        // Env aliases: comma-separated k=v pairs
        if let Ok(raw) = std::env::var("NYASH_ALIASES") {
            for ent in raw.split(',') {
                if let Some((k, v)) = ent.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    if !k.is_empty() && !v.is_empty() {
                        aliases.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }

        UsingContext {
            using_paths,
            pending_modules,
            aliases,
            packages,
        }
    }
}

/// Suggest candidate files by leaf name within limited bases (apps/lib/.)
pub(super) fn suggest_in_base(base: &str, leaf: &str, out: &mut Vec<String>) {
    use std::fs;
    fn walk(dir: &std::path::Path, leaf: &str, out: &mut Vec<String>, depth: usize) {
        if depth == 0 || out.len() >= 5 {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let path = e.path();
                if path.is_dir() {
                    walk(&path, leaf, out, depth - 1);
                    if out.len() >= 5 {
                        return;
                    }
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "nyash" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem == leaf {
                                out.push(path.to_string_lossy().to_string());
                                if out.len() >= 5 {
                                    return;
                                }
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
    packages: &HashMap<String, UsingPackage>,
    context_dir: Option<&std::path::Path>,
    strict: bool,
    verbose: bool,
) -> Result<String, String> {
    // Invalidate and rebuild index/cache if env or nyash.toml changed
    super::box_index::rebuild_if_env_changed();
    if is_path {
        return Ok(tgt.to_string());
    }
    let trace = verbose || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1");
    let idx = super::box_index::get_box_index();
    let mut strict_effective = strict || idx.plugins_require_prefix_global;
    if std::env::var("NYASH_PLUGIN_REQUIRE_PREFIX").ok().as_deref() == Some("1") {
        strict_effective = true;
    }
    let meta_for_target = idx.plugin_meta_by_box.get(tgt).cloned();
    let mut require_prefix_target = meta_for_target
        .as_ref()
        .map(|m| m.require_prefix)
        .unwrap_or(false);
    if let Some(m) = &meta_for_target {
        if !m.expose_short_names {
            require_prefix_target = true;
        }
    }
    let mut is_plugin_short = meta_for_target.is_some();
    if !is_plugin_short {
        is_plugin_short = idx.plugin_boxes.contains(tgt)
            || super::box_index::BoxIndex::is_known_plugin_short(tgt);
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
        format!(
            "{}|{}|{}|{}",
            tgt,
            base,
            strict as i32,
            using_paths.join(":")
        )
    };
    if let Some(hit) = crate::runner::box_index::cache_get(&key) {
        if trace {
            crate::runner::trace::log(format!("[using/cache] '{}' -> '{}'", tgt, hit));
        }
        return Ok(hit);
    }
    // Resolve aliases early (provided map) — and then recursively resolve the target
    if let Some(v) = aliases.get(tgt) {
        if trace {
            if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using/resolve] alias '{}' -> '{}'", tgt, v)); }
        }
        // Recurse to resolve the alias target into a concrete path/token
        let rec = resolve_using_target(v, false, modules, using_paths, aliases, packages, context_dir, strict, verbose)?;
        crate::runner::trace::log_json_using(tgt, Some(&rec), &[], "alias");
        crate::runner::box_index::cache_put(&key, rec.clone());
        return Ok(rec);
    }
    // Named packages (nyash.toml [using.<name>])
    if let Some(pkg) = packages.get(tgt) {
        match pkg.kind {
            PackageKind::Dylib => {
                // Return a marker token to avoid inlining attempts; loader will consume later stages
                let out = format!("dylib:{}", pkg.path);
                if trace {
                    if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using/resolve] dylib '{}' -> '{}'", tgt, out)); }
                }
                crate::runner::trace::log_json_using(tgt, Some(&out), &[], "dylib");
                crate::runner::box_index::cache_put(&key, out.clone());
                return Ok(out);
            }
            PackageKind::Package => {
                // Compute entry: main or <dir_last>.hako (preferred) / .nyash (legacy)
                let base = std::path::Path::new(&pkg.path);
                let out = if let Some(m) = &pkg.main {
                    if matches!(base.extension().and_then(|s| s.to_str()), Some("hako") | Some("nyash")) {
                        // path is a file; ignore main and use as-is
                        pkg.path.clone()
                    } else {
                        base.join(m).to_string_lossy().to_string()
                    }
                } else {
                    if matches!(base.extension().and_then(|s| s.to_str()), Some("hako") | Some("nyash")) {
                        pkg.path.clone()
                    } else {
                        let leaf = base.file_name().and_then(|s| s.to_str()).unwrap_or(tgt);
                        let cand_h = base.join(format!("{}.hako", leaf));
                        if cand_h.exists() {
                            cand_h.to_string_lossy().to_string()
                        } else {
                            base.join(format!("{}.nyash", leaf)).to_string_lossy().to_string()
                        }
                    }
                };
                if trace {
                    if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using/resolve] package '{}' -> '{}'", tgt, out)); }
                }
                crate::runner::trace::log_json_using(tgt, Some(&out), &[], "package");
                crate::runner::box_index::cache_put(&key, out.clone());
                return Ok(out);
            }
        }
    }
    // Also consult env aliases
    if let Ok(raw) = std::env::var("NYASH_ALIASES") {
        for ent in raw.split(',') {
            if let Some((k, v)) = ent.split_once('=') {
                if k.trim() == tgt {
                    let out = v.trim().to_string();
                    if trace {
                        crate::runner::trace::log(format!(
                            "[using/resolve] env-alias '{}' -> '{}'",
                            tgt, out
                        ));
                    }
                    crate::runner::trace::log_json_using(tgt, Some(&out), &[], "env-alias");
                    crate::runner::box_index::cache_put(&key, out.clone());
                    return Ok(out);
                }
            }
        }
    }
    // 1) modules mapping
    if let Some((_, p)) = modules.iter().find(|(n, _)| n == tgt) {
        let out = p.clone();
        if trace {
            if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using/resolve] modules '{}' -> '{}'", tgt, out)); }
        }
        crate::runner::trace::log_json_using(tgt, Some(&out), &[], "modules");
        crate::runner::box_index::cache_put(&key, out.clone());
        return Ok(out);
    }
    // 2) Special handling for built-in namespaces
    if tgt == "nyashstd" {
        let out = "builtin:nyashstd".to_string();
        if trace {
            if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using/resolve] builtin '{}' -> '{}'", tgt, out)); }
        }
        crate::runner::trace::log_json_using(tgt, Some(&out), &[], "builtin");
        crate::runner::box_index::cache_put(&key, out.clone());
        return Ok(out);
    }
    // 3) build candidate list: relative then using-paths (prefer .hako, fallback .nyash)
    let rel_h = tgt.replace('.', "/") + ".hako";
    let rel_ny = tgt.replace('.', "/") + ".nyash";
    let mut cand: Vec<String> = Vec::new();
    if let Some(dir) = context_dir {
        let c1 = dir.join(&rel_h);
        let c2 = dir.join(&rel_ny);
        if c1.exists() { cand.push(c1.to_string_lossy().to_string()); }
        if c2.exists() { cand.push(c2.to_string_lossy().to_string()); }
    }
    for base in using_paths {
        let c1 = std::path::Path::new(base).join(&rel_h);
        let c2 = std::path::Path::new(base).join(&rel_ny);
        if c1.exists() { cand.push(c1.to_string_lossy().to_string()); }
        if c2.exists() { cand.push(c2.to_string_lossy().to_string()); }
    }
    if cand.is_empty() {
        // Always emit a concise unresolved note to aid diagnostics in smokes
        let leaf = tgt.split('.').last().unwrap_or(tgt);
        let mut cands: Vec<String> = Vec::new();
        suggest_in_base("apps", leaf, &mut cands);
        if cands.len() < 5 { suggest_in_base("lib", leaf, &mut cands); }
        if cands.len() < 5 { suggest_in_base(".", leaf, &mut cands); }
        if trace {
            if cands.is_empty() {
                if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using] unresolved '{}' (searched: rel+paths)", tgt)); }
            } else {
                if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using] unresolved '{}' (searched: rel+paths) candidates: {}", tgt, cands.join(", ")))};
            }
        } else {
            if !crate::config::env::cli_quiet() { eprintln!("[using] not found: '{}'", tgt); }
        }
        crate::runner::trace::log_json_using(tgt, None, &cands, "unresolved");
        return Ok(tgt.to_string());
    }
    if cand.len() > 1 && strict {
        return Err(format!("ambiguous using '{}': {}", tgt, cand.join(", ")));
    }
    let out = cand.remove(0);
    if trace {
        if !crate::config::env::cli_quiet() { crate::runner::trace::log(format!("[using/resolve] '{}' -> '{}'", tgt, out)); }
    }
    crate::runner::trace::log_json_using(tgt, Some(&out), &[], "fs");
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
            let after = if let Some(rest) = trimmed.strip_prefix("static box ") {
                rest
            } else {
                trimmed.strip_prefix("box ").unwrap_or("")
            };
            for ch in after.chars() {
                if ch.is_alphanumeric() || ch == '_' {
                    name.push(ch);
                } else {
                    break;
                }
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
                        while let Some(c) = it.next() {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c.is_alphabetic() || c == '_' {
                                ident.push(c);
                                break;
                            } else {
                                break;
                            }
                        }
                        while let Some(c) = it.next() {
                            if c.is_alphanumeric() || c == '_' {
                                ident.push(c);
                            } else {
                                break;
                            }
                        }
                        trimmed.contains('(') && trimmed.ends_with('{') && !ident.is_empty()
                    };
                    if is_method {
                        seen_method = true;
                    }

                    // Detect field: ident ':' Type (rough heuristic)
                    let is_field = {
                        let parts: Vec<&str> = trimmed.split(':').collect();
                        if parts.len() == 2 {
                            let lhs = parts[0].trim();
                            let rhs = parts[1].trim();
                            let lhs_ok = !lhs.is_empty()
                                && lhs
                                    .chars()
                                    .next()
                                    .map(|c| c.is_alphabetic() || c == '_')
                                    .unwrap_or(false);
                            let rhs_ok = !rhs.is_empty()
                                && rhs
                                    .chars()
                                    .next()
                                    .map(|c| c.is_alphabetic() || c == '_')
                                    .unwrap_or(false);
                            lhs_ok && rhs_ok && !trimmed.contains('(') && !trimmed.contains(')')
                        } else {
                            false
                        }
                    };
                    if is_field && seen_method {
                        violations.push((lno, trimmed.to_string(), cur_box.clone()));
                    }
                }
            }
            // Exit box when closing brace reduces depth below box_depth
            let post_brace = pre_brace + opens - closes;
            if post_brace < box_depth {
                in_box = false;
                cur_box.clear();
            }
        }

        // Update brace after processing
        brace += opens - closes;
    }

    if violations.is_empty() {
        return Ok(());
    }
    if strict {
        // Compose error message
        let mut msg =
            String::from("Field declarations must appear at the top of box. Violations:\n");
        for (lno, fld, bx) in violations.iter().take(10) {
            msg.push_str(&format!(
                "  line {} in box {}: '{}",
                lno,
                if bx.is_empty() { "<unknown>" } else { bx },
                fld
            ));
            msg.push_str("'\n");
        }
        if violations.len() > 10 {
            msg.push_str(&format!("  ... and {} more\n", violations.len() - 10));
        }
        return Err(msg);
    }
    if verbose || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
        for (lno, fld, bx) in violations {
            eprintln!(
                "[lint] fields-top: line {} in box {} -> {}",
                lno,
                if bx.is_empty() { "<unknown>" } else { &bx },
                fld
            );
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
            &std::collections::HashMap::<String, crate::using::spec::UsingPackage>::new(),
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
