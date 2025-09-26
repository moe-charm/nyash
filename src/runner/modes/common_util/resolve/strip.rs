use crate::runner::NyashRunner;

/// Collect using targets and strip using lines, without inlining.
/// Returns (cleaned_source, prelude_paths) where prelude_paths are resolved file paths
/// to be parsed separately and AST-merged (when NYASH_USING_AST=1).
pub fn collect_using_and_strip(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<(String, Vec<String>), String> {
    if !crate::config::env::enable_using() {
        return Ok((code.to_string(), Vec::new()));
    }
    let using_ctx = runner.init_using_context();
    let prod = crate::config::env::using_is_prod();
    let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
    let verbose = crate::config::env::cli_verbose()
        || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1");
    let ctx_dir = std::path::Path::new(filename).parent();

    let mut out = String::with_capacity(code.len());
    let mut prelude_paths: Vec<String> = Vec::new();
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
    for line in code.lines() {
        let t = line.trim_start();
        if t.starts_with("using ") {
            crate::cli_v!("[using] stripped line: {}", line);
            let rest0 = t.strip_prefix("using ").unwrap().trim();
            let rest0 = rest0.split('#').next().unwrap_or(rest0).trim();
            let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
            let (target, _alias) = if let Some(pos) = rest0.find(" as ") {
                (
                    rest0[..pos].trim().to_string(),
                    Some(rest0[pos + 4..].trim().to_string()),
                )
            } else {
                (rest0.to_string(), None)
            };
            let is_path = target.starts_with('"')
                || target.starts_with("./")
                || target.starts_with('/')
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
                prelude_paths.push(p.to_string_lossy().to_string());
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
                                if base.extension().and_then(|s| s.to_str()) == Some("nyash") {
                                    pkg.path.clone()
                                } else {
                                    base.join(m).to_string_lossy().to_string()
                                }
                            } else if base.extension().and_then(|s| s.to_str()) == Some("nyash") {
                                pkg.path.clone()
                            } else {
                                let leaf = base
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(&pkg_name);
                                base.join(format!("{}.nyash", leaf))
                                    .to_string_lossy()
                                    .to_string()
                            };
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
                        // Only file paths are candidates for AST prelude merge
                        if value.ends_with(".nyash") || value.contains('/') || value.contains('\\')
                        {
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
                            prelude_paths.push(p.to_string_lossy().to_string());
                        }
                    }
                    Err(e) => return Err(format!("using: {}", e)),
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
    Ok((out, prelude_paths))
}

/// Profile-aware prelude resolution wrapper.
/// Currently delegates to `collect_using_and_strip`, but provides a single
/// entry point for callers (common and vm_fallback) to avoid logic drift.
pub fn resolve_prelude_paths_profiled(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<(String, Vec<String>), String> {
    // First pass: strip using from the main source and collect direct prelude paths
    let (cleaned, direct) = collect_using_and_strip(runner, code, filename)?;
    // When AST using is enabled、recursively collect nested preludes in DFS order
    let ast_on = std::env::var("NYASH_USING_AST").ok().as_deref() == Some("1");
    if !ast_on {
        return Ok((cleaned, direct));
    }
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    fn normalize_path(path: &str) -> (String, String) {
        use std::path::PathBuf;
        match PathBuf::from(path).canonicalize() {
            Ok(canon) => {
                let s = canon.to_string_lossy().to_string();
                (s.clone(), s)
            }
            Err(_) => {
                // Fall back to the original path representation.
                (path.to_string(), path.to_string())
            }
        }
    }
    fn dfs(
        runner: &NyashRunner,
        path: &str,
        out: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        let (key, real_path) = normalize_path(path);
        if !seen.insert(key.clone()) {
            return Ok(());
        }
        let src = std::fs::read_to_string(&real_path)
            .map_err(|e| format!("using: failed to read '{}': {}", real_path, e))?;
        let (_cleaned, nested) = collect_using_and_strip(runner, &src, &real_path)?;
        for n in nested.iter() {
            dfs(runner, n, out, seen)?;
        }
        out.push(real_path);
        Ok(())
    }
    for p in direct.iter() {
        dfs(runner, p, &mut out, &mut seen)?;
    }
    Ok((cleaned, out))
}

/// Pre-expand line-head `@name[: Type] = expr` into `local name[: Type] = expr`.
/// Minimal, safe, no semantics change. Applies only at line head (after spaces/tabs).
pub fn preexpand_at_local(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'@' {
            // parse identifier
            let mut j = i + 1;
            if j < bytes.len() && ((bytes[j] as char).is_ascii_alphabetic() || bytes[j] == b'_') {
                j += 1;
                while j < bytes.len() {
                    let c = bytes[j] as char;
                    if c.is_ascii_alphanumeric() || c == '_' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                let mut k = j;
                while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b':' {
                    k += 1;
                    while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
                        k += 1;
                    }
                    if k < bytes.len()
                        && ((bytes[k] as char).is_ascii_alphabetic() || bytes[k] == b'_')
                    {
                        k += 1;
                        while k < bytes.len() {
                            let c = bytes[k] as char;
                            if c.is_ascii_alphanumeric() || c == '_' {
                                k += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                let mut eqp = k;
                while eqp < bytes.len() && (bytes[eqp] == b' ' || bytes[eqp] == b'\t') {
                    eqp += 1;
                }
                if eqp < bytes.len() && bytes[eqp] == b'=' {
                    out.push_str(&line[..i]);
                    out.push_str("local ");
                    out.push_str(&line[i + 1..eqp]);
                    out.push_str(" =");
                    out.push_str(&line[eqp + 1..]);
                    out.push('\n');
                    continue;
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
