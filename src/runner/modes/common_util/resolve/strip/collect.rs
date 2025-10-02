use crate::runner::NyashRunner;

/// Collect using targets and strip using lines (no inlining).
/// Returns (cleaned_source, prelude_paths) where `prelude_paths` are resolved
/// file paths to be parsed separately and AST-merged (when `NYASH_USING_AST=1`).
///
/// Notes
/// - This function enforces profile policies (prod: disallow file-using; only
///   packages/aliases from nyash.toml are accepted).
/// - SSOT: Resolution sources and aliases come exclusively from nyash.toml.
/// - All runner modes use this static path to avoid logic drift.
pub fn collect_using_and_strip(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<(String, Vec<String>, Vec<(String, String)>), String> {
    if !crate::config::env::enable_using() {
        return Ok((code.to_string(), Vec::new(), Vec::new()));
    }
    let using_ctx = runner.init_using_context();
    let prod = crate::config::env::using_is_prod();
    let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
    let verbose = crate::config::env::cli_verbose()
        || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1");
    let ctx_dir = std::path::Path::new(filename).parent();

    let mut out = String::with_capacity(code.len());
    let mut prelude_paths: Vec<String> = Vec::new();
    let mut alias_pairs: Vec<(String, String)> = Vec::new(); // (alias, canon_path)
    // Duplicate-using detection (same target imported multiple times or alias rebound): error in all profiles
    use std::collections::HashMap;
    let mut seen_paths: HashMap<String, (String, usize)> = HashMap::new(); // canon_path -> (alias/label, first_line)
    let mut seen_aliases: HashMap<String, (String, usize)> = HashMap::new(); // alias -> (canon_path, first_line)
    // Determine if this file is inside a declared package root; if so, allow
    // internal file-using within the package even when file-using is globally disallowed.
    let filename_canon = std::fs::canonicalize(filename).ok();
    let mut inside_pkg = false;
    if let Some(ref fc) = filename_canon {
        for (_name, pkg) in &using_ctx.packages {
            let base = std::path::Path::new(&pkg.path);
            if let Ok(root) = std::fs::canonicalize(base) {
                if fc.starts_with(&root) {
                    inside_pkg = true;
                    break;
                }
            }
        }
    }
    // Dev-friendly guard: treat any file under NYASH_ROOT/apps/lib or NYASH_ROOT/apps as inside a package
    if !inside_pkg {
        if let (Some(ref fc), Ok(root)) = (&filename_canon, std::env::var("NYASH_ROOT")) {
            let r = std::path::Path::new(&root);
            let libroot = r.join("apps").join("lib");
            let appr = r.join("apps");
            if fc.starts_with(&libroot) || fc.starts_with(&appr) {
                inside_pkg = true;
            }
        }
    }
    for (lineno0, line) in code.lines().enumerate() {
        let line_no = lineno0 + 1;
        let t = line.trim_start();
        if t.starts_with("using ") {
            crate::cli_v!("[using] stripped line: {}", line);
            let rest0 = t.strip_prefix("using ").unwrap().trim();
            let rest0 = rest0.split('#').next().unwrap_or(rest0).trim();
            let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
            let (target, alias_name) = if let Some(pos) = rest0.find(" as ") {
                (
                    rest0[..pos].trim().to_string(),
                    Some(rest0[pos + 4..].trim().to_string()),
                )
            } else {
                (rest0.to_string(), None)
            };
            if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                eprintln!("[using] alias-trace: using target='{}' alias={:?}", target, alias_name);
            }
            let is_win_abs = target.len() >= 2
                && target.as_bytes()[0].is_ascii_alphabetic()
                && target.as_bytes()[1] == b':' ;
            let is_path = target.starts_with('"')
                || target.starts_with("./")
                || target.starts_with('/')
                || is_win_abs
                || target.ends_with(".hako")
                || target.ends_with(".nyash");
            if is_path {
                // SSOT: Disallow file-using at top-level; allow only for sources located
                // under a declared package root (internal package wiring), so that packages
                // can organize their modules via file paths.
                if (prod || !crate::config::env::allow_using_file()) && !inside_pkg {
                    return Err(format!(
                        "using: file paths are disallowed in this profile. Add it to nyash.toml [using] (packages/aliases) and reference by name: {}",
                        target
                    ));
                }
                let path = target.trim_matches('"').to_string();
                // Resolve relative to current file dir
                let mut p = std::path::PathBuf::from(&path);
                if p.is_relative() {
                    if let Some(dir) = ctx_dir {
                        let cand = dir.join(&p);
                        if cand.exists() {
                            p = cand;
                        }
                    }
                    // Also try NYASH_ROOT when available (repo-root relative like "apps/...")
                    if p.is_relative() {
                        if let Ok(root) = std::env::var("NYASH_ROOT") {
                            let cand = std::path::Path::new(&root).join(&p);
                            if cand.exists() {
                                p = cand;
                            }
                        } else {
                            // Fallback: guess project root from executable path (target/release/nyash)
                            if let Ok(exe) = std::env::current_exe() {
                                if let Some(root) = exe
                                    .parent()
                                    .and_then(|p| p.parent())
                                    .and_then(|p| p.parent())
                                {
                                    let cand = root.join(&p);
                                    if cand.exists() {
                                        p = cand;
                                    }
                                }
                            }
                        }
                    }
                }
                if verbose {
                    crate::runner::trace::log(format!(
                        "[using/resolve] file '{}' -> '{}'",
                        target,
                        p.display()
                    ));
                }
                let path_str = p.to_string_lossy().to_string();
                // Duplicate detection
                let canon = std::fs::canonicalize(&path_str)
                    .ok()
                    .map(|pb| pb.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_str.clone());
                if let Some((prev_alias, prev_line)) = seen_paths.get(&canon) {
                    return Err(format!(
                        "using: duplicate import of '{}' at {}:{} (previous alias: '{}' first seen at line {})",
                        canon,
                        filename,
                        line_no,
                        prev_alias,
                        prev_line
                    ));
                } else {
                    seen_paths.insert(canon.clone(), (alias_name.clone().unwrap_or_else(|| "<none>".into()), line_no));
                }
                if let Some(alias) = alias_name.clone() {
                    if let Some((prev_path, prev_line)) = seen_aliases.get(&alias) {
                        if prev_path != &canon {
                            return Err(format!(
                                "using: alias '{}' rebound at {}:{} (was '{}' first seen at line {})",
                                alias,
                                filename,
                                line_no,
                                prev_path,
                                prev_line
                            ));
                        }
                    } else {
                        seen_aliases.insert(alias.clone(), (canon.clone(), line_no));
                        alias_pairs.push((alias, canon));
                    }
                }
                prelude_paths.push(path_str);
                continue;
            }
            // Resolve namespaces/packages
            if prod {
                // prod: only allow names present in aliases/packages (toml)
                let mut pkg_name: String = target.clone();
                if let Some(v) = using_ctx.aliases.get(&target) {
                    pkg_name = v.clone();
                }
                if let Some(pkg) = using_ctx.packages.get(&pkg_name) {
                    use crate::using::spec::PackageKind;
                    match pkg.kind {
                        PackageKind::Dylib => {
                            // dylib: nothing to prelude-parse; runtime loader handles it.
                        }
                        PackageKind::Package => {
                            let base = std::path::Path::new(&pkg.path);
                            let out = if let Some(m) = &pkg.main {
                                if matches!(base.extension().and_then(|s| s.to_str()), Some("hako") | Some("nyash")) {
                                    pkg.path.clone()
                                } else {
                                    base.join(m).to_string_lossy().to_string()
                                }
                            } else if matches!(base.extension().and_then(|s| s.to_str()), Some("hako") | Some("nyash")) {
                                pkg.path.clone()
                            } else {
                                let leaf = base
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(&pkg_name);
                                let cand_h = base.join(format!("{}.hako", leaf));
                                if cand_h.exists() {
                                    cand_h.to_string_lossy().to_string()
                                } else {
                                    base.join(format!("{}.nyash", leaf))
                                        .to_string_lossy()
                                        .to_string()
                                }
                            };
                            // Duplicate detection for prod package alias resolution
                            let canon = std::fs::canonicalize(&out)
                                .ok()
                                .map(|pb| pb.to_string_lossy().to_string())
                                .unwrap_or_else(|| out.clone());
                            if let Some((prev_alias, prev_line)) = seen_paths.get(&canon) {
                                return Err(format!(
                                    "using: duplicate import of '{}' at {}:{} (previous alias: '{}' first seen at line {})",
                                    canon,
                                    filename,
                                    line_no,
                                    prev_alias,
                                    prev_line
                                ));
                            } else {
                                seen_paths.insert(canon.clone(), (alias_name.clone().unwrap_or_else(|| "<none>".into()), line_no));
                            }
                            if let Some(alias) = alias_name.clone() {
                                if let Some((prev_path, prev_line)) = seen_aliases.get(&alias) {
                                    if prev_path != &canon {
                                        return Err(format!(
                                            "using: alias '{}' rebound at {}:{} (was '{}' first seen at line {})",
                                            alias, filename, line_no, prev_path, prev_line
                                        ));
                                    }
                                } else {
                                    seen_aliases.insert(alias, (canon, line_no));
                                }
                            }
                            prelude_paths.push(out);
                        }
                    }
                } else {
                    return Err(format!(
                        "using: '{}' not found in nyash.toml [using]. Define a package or alias and use its name (prod profile)",
                        target
                    ));
                }
            } else {
                // dev/ci: allow broader resolution via resolver
                match crate::runner::pipeline::resolve_using_target(
                    &target,
                    false,
                    &using_ctx.pending_modules,
                    &using_ctx.using_paths,
                    &using_ctx.aliases,
                    &using_ctx.packages,
                    ctx_dir,
                    strict,
                    verbose,
                ) {
                    Ok(value) => {
                        // Only file paths are candidates for AST prelude merge.
                        // Ignore special marker tokens like "dylib:<path>" (loader handles them).
                if !value.starts_with("dylib:") && (value.ends_with(".hako") || value.ends_with(".nyash") || value.contains('/') || value.contains('\\')) {
                            // Resolve relative
                            let mut p = std::path::PathBuf::from(&value);
                            if p.is_relative() {
                                if let Some(dir) = ctx_dir {
                                    let cand = dir.join(&p);
                                    if cand.exists() {
                                        p = cand;
                                    }
                                }
                                if p.is_relative() {
                                    if let Ok(root) = std::env::var("NYASH_ROOT") {
                                        let cand = std::path::Path::new(&root).join(&p);
                                        if cand.exists() {
                                            p = cand;
                                        }
                                    } else {
                                        if let Ok(exe) = std::env::current_exe() {
                                            if let Some(root) = exe
                                                .parent()
                                                .and_then(|p| p.parent())
                                                .and_then(|p| p.parent())
                                            {
                                                let cand = root.join(&p);
                                                if cand.exists() {
                                                    p = cand;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if verbose {
                                crate::runner::trace::log(format!(
                                    "[using/resolve] dev-file '{}' -> '{}'",
                                    value,
                                    p.display()
                                ));
                            }
                            let path_str = p.to_string_lossy().to_string();
                            let canon = std::fs::canonicalize(&path_str)
                                .ok()
                                .map(|pb| pb.to_string_lossy().to_string())
                                .unwrap_or_else(|| path_str.clone());
                            if let Some((prev_alias, prev_line)) = seen_paths.get(&canon) {
                                return Err(format!(
                                    "using: duplicate import of '{}' at {}:{} (previous alias: '{}' first seen at line {})",
                                    canon,
                                    filename,
                                    line_no,
                                    prev_alias,
                                    prev_line
                                ));
                            } else {
                                seen_paths.insert(canon.clone(), (alias_name.clone().unwrap_or_else(|| "<none>".into()), line_no));
                            }
                            if let Some(alias) = alias_name.clone() {
                                if let Some((prev_path, prev_line)) = seen_aliases.get(&alias) {
                                    if prev_path != &canon {
                                        return Err(format!(
                                            "using: alias '{}' rebound at {}:{} (was '{}' first seen at line {})",
                                            alias, filename, line_no, prev_path, prev_line
                                        ));
                                    }
                                } else {
                                    seen_aliases.insert(alias.clone(), (canon.clone(), line_no));
                                    if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                                        crate::runner::trace::log(format!("[using/alias] push pair alias='{}' canon='{}'", alias, canon));
                                    }
                                    alias_pairs.push((alias, canon));
                                }
                            }
                            prelude_paths.push(path_str);
                        } else {
                            // Non-path token. When resolver returns the input unchanged (unresolved),
                            // attempt a dev fallback using alias name to locate a prelude file.
                            if value == target {
                                if let Some(alias) = alias_name.clone() {
                                    if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                                        eprintln!("[using] alias-trace: unresolved token '{}', try alias scan for '{}'", value, alias);
                                    }
                                    // Reuse the same alias-scan helper as in Err branch
                                    // Build candidate roots: context dir, NYASH_ROOT/{apps,lib}, and using paths
                                    let mut roots: Vec<std::path::PathBuf> = Vec::new();
                                    if let Some(dir) = ctx_dir { roots.push(dir.to_path_buf()); }
                                    if let Ok(root) = std::env::var("NYASH_ROOT") {
                                        roots.push(std::path::Path::new(&root).join("apps"));
                                        roots.push(std::path::Path::new(&root).join("lib"));
                                    }
                                    for p in &using_ctx.using_paths {
                                        let pb = std::path::PathBuf::from(p);
                                        if pb.exists() { roots.push(pb); }
                                        if let Ok(root) = std::env::var("NYASH_ROOT") {
                                            let cand = std::path::Path::new(&root).join(p);
                                            if cand.exists() { roots.push(cand); }
                                        }
                                    }
                                    use std::collections::HashSet;
                                    let mut seen_dirs: HashSet<String> = HashSet::new();
                                    let mut uniq_roots: Vec<std::path::PathBuf> = Vec::new();
                                    for r in roots.into_iter() {
                                        let key = std::fs::canonicalize(&r).ok().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| r.to_string_lossy().to_string());
                                        if seen_dirs.insert(key) { uniq_roots.push(r); }
                                    }
                                    fn scan_for_alias(root: &std::path::Path, alias: &str, max_depth: usize) -> Option<String> {
                                        if max_depth == 0 { return None; }
                                        let rd = std::fs::read_dir(root).ok()?;
                                        for ent in rd.flatten() {
                                            let p = ent.path();
                                            if p.is_dir() {
                                                if let Some(hit) = scan_for_alias(&p, alias, max_depth - 1) { return Some(hit); }
                                            } else if matches!(p.extension().and_then(|s| s.to_str()), Some("hako") | Some("nyash")) {
                                                if let Ok(mut f) = std::fs::File::open(&p) {
                                                    use std::io::{BufRead, BufReader};
                                                    let br = BufReader::new(&mut f);
                                                    let needle = format!("static box {}", alias);
                                                    for line in br.lines().flatten().take(400) {
                                                        if line.contains(&needle) {
                                                            return Some(p.to_string_lossy().to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        None
                                    }
                                    let mut found: Option<String> = None;
                                    for r in uniq_roots.iter() {
                                        if let Some(hit) = scan_for_alias(r, &alias, 6) { found = Some(hit); break; }
                                    }
                                    if let Some(path_str) = found {
                                        let canon = std::fs::canonicalize(&path_str)
                                            .ok()
                                            .map(|pb| pb.to_string_lossy().to_string())
                                            .unwrap_or_else(|| path_str.clone());
                                        if let Some((prev_path, prev_line)) = seen_aliases.get(&alias) {
                                            if prev_path != &canon {
                                                return Err(format!(
                                                    "using: alias '{}' rebound at {}:{} (was '{}' first seen at line {})",
                                                    alias, filename, line_no, prev_path, prev_line
                                                ));
                                            }
                                        } else {
                                            seen_aliases.insert(alias.clone(), (canon.clone(), line_no));
                                            if verbose { eprintln!("[using] alias-trace: alias '{}' -> '{}' (scan)", alias, canon); }
                                            alias_pairs.push((alias, canon));
                                        }
                                        prelude_paths.push(path_str);
                                    }
                                }
                            }
                        }
                    }
                    Err(_e) => {
                        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                            eprintln!("[using/fallback] entering dev fallback for target='{}' alias={:?}", target, alias_name);
                        }
                        // Dev fallback: if alias provided and resolver failed, try to locate a .nyash file
                        // that defines a static box with the alias name under known search paths.
                        if let Some(alias) = alias_name.clone() {
                            if verbose {
                                crate::runner::trace::log(format!("[using/fallback] try alias '{}' via content scan", alias));
                            }
                            // Build candidate roots: context dir, NYASH_ROOT/apps, NYASH_ROOT/lib, and using paths
                            let mut roots: Vec<std::path::PathBuf> = Vec::new();
                            if let Some(dir) = ctx_dir { roots.push(dir.to_path_buf()); }
                            if let Ok(root) = std::env::var("NYASH_ROOT") {
                                roots.push(std::path::Path::new(&root).join("apps"));
                                roots.push(std::path::Path::new(&root).join("lib"));
                            }
                            for p in &using_ctx.using_paths {
                                let pb = std::path::PathBuf::from(p);
                                if pb.exists() { roots.push(pb); }
                                if let Ok(root) = std::env::var("NYASH_ROOT") {
                                    let cand = std::path::Path::new(&root).join(p);
                                    if cand.exists() { roots.push(cand); }
                                }
                            }
                            // Dedup roots (by canonical path when possible)
                            use std::collections::HashSet;
                            let mut seen_dirs: HashSet<String> = HashSet::new();
                            let mut uniq_roots: Vec<std::path::PathBuf> = Vec::new();
                            for r in roots.into_iter() {
                                let key = std::fs::canonicalize(&r).ok().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| r.to_string_lossy().to_string());
                                if seen_dirs.insert(key) { uniq_roots.push(r); }
                            }
                            // Walk shallowly (depth <= 6), stop at first match
                            fn scan_for_alias(root: &std::path::Path, alias: &str, max_depth: usize) -> Option<String> {
                                if max_depth == 0 { return None; }
                                let rd = std::fs::read_dir(root).ok()?;
                                for ent in rd.flatten() {
                                    let p = ent.path();
                                    if p.is_dir() {
                                        if let Some(hit) = scan_for_alias(&p, alias, max_depth - 1) { return Some(hit); }
                                    } else if matches!(p.extension().and_then(|s| s.to_str()), Some("hako") | Some("nyash")) {
                                        if let Ok(mut f) = std::fs::File::open(&p) {
                                            use std::io::{BufRead, BufReader};
                                            let br = BufReader::new(&mut f);
                                            let needle = format!("static box {}", alias);
                                            for line in br.lines().flatten().take(400) {
                                                if line.contains(&needle) {
                                                    return Some(p.to_string_lossy().to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                None
                            }
                            let mut found: Option<String> = None;
                            for r in uniq_roots.iter() {
                                if let Some(hit) = scan_for_alias(r, &alias, 6) { found = Some(hit); break; }
                            }
                            if let Some(path_str) = found {
                                // Canonicalize for stability
                                let canon = std::fs::canonicalize(&path_str)
                                    .ok()
                                    .map(|pb| pb.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path_str.clone());
                                if let Some((prev_path, prev_line)) = seen_aliases.get(&alias) {
                                    if prev_path != &canon {
                                        return Err(format!(
                                            "using: alias '{}' rebound at {}:{} (was '{}' first seen at line {})",
                                            alias, filename, line_no, prev_path, prev_line
                                        ));
                                    }
                                } else {
                                    seen_aliases.insert(alias.clone(), (canon.clone(), line_no));
                                    if verbose { crate::runner::trace::log(format!("[using/fallback] alias='{}' -> '{}'", alias, canon)); }
                                    alias_pairs.push((alias, canon));
                                }
                                prelude_paths.push(path_str);
                                continue;
                            }
                        }
                        // No fallback; return original error
                        return Err(format!("using: failed to resolve '{}' (dev path)", target));
                    },
                }
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    // Optional prelude boundary comment (helps manual inspection; parser ignores comments)
    if std::env::var("NYASH_RESOLVE_SEAM_DEBUG").ok().as_deref() == Some("1") {
        let mut with_marker = String::with_capacity(out.len() + 64);
        with_marker.push_str("\n/* --- using boundary (AST) --- */\n");
        with_marker.push_str(&out);
        out = with_marker;
    }
    Ok((out, prelude_paths, alias_pairs))
}
