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
                                    alias_pairs.push((alias, canon));
                                }
                            }
                            prelude_paths.push(path_str);
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
    Ok((out, prelude_paths, alias_pairs))
}

/// Profile-aware prelude resolution wrapper (single entrypoint).
/// - Delegates to `collect_using_and_strip` for the first pass.
/// - When AST using is enabled, resolves nested preludes via DFS and injects
///   OperatorBox preludes when available (stringify/compare/add).
/// - All runners call this helper; do not fork resolution logic elsewhere.
pub fn resolve_prelude_paths_profiled(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<(String, Vec<String>, Vec<(String, String)>), String> {
    // First pass: strip using from the main source and collect direct prelude paths
    let (cleaned, direct, alias_pairs) = collect_using_and_strip(runner, code, filename)?;
    // When AST using is enabled、recursively collect nested preludes in DFS order
    let ast_on = std::env::var("NYASH_USING_AST").ok().as_deref() == Some("1");
    if !ast_on {
        return Ok((cleaned, direct, alias_pairs));
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
        let (_cleaned, nested, _aliases) = collect_using_and_strip(runner, &src, &real_path)?;
        for n in nested.iter() {
            dfs(runner, n, out, seen)?;
        }
        out.push(real_path);
        Ok(())
    }
    for p in direct.iter() {
        dfs(runner, p, &mut out, &mut seen)?;
    }
    // Operator Boxes prelude injection（観測“常時ON”のため）
    // stringify/compare/add は常に注入（存在時）。その他（bitwise等）は ALL 指定時のみ。
    let opbox_all = std::env::var("NYASH_OPERATOR_BOX_ALL").ok().as_deref() == Some("1")
        || std::env::var("NYASH_BUILDER_OPERATOR_BOX_ALL_CALL").ok().as_deref() == Some("1");

    if let Ok(root) = std::env::var("NYASH_ROOT") {
        let must_have = [
            "apps/lib/std/operators/stringify.nyash",
            "apps/lib/std/operators/compare.nyash",
            "apps/lib/std/operators/add.nyash",
        ];
        for rel in must_have.iter() {
            let p = std::path::Path::new(&root).join(rel);
            if p.exists() {
                let path = p.to_string_lossy().to_string();
                if !out.iter().any(|x| x == &path) {
                    out.push(path);
                }
            }
        }
    }
    // Inject remaining arithmetic/bitwise/unary operator modules when ALL is requested
    if opbox_all {
        if let Ok(root) = std::env::var("NYASH_ROOT") {
            let rels = vec![
                "apps/lib/std/operators/sub.nyash",
                "apps/lib/std/operators/mul.nyash",
                "apps/lib/std/operators/div.nyash",
                "apps/lib/std/operators/mod.nyash",
                // Shifts / bitwise (parser tokens now supported)
                "apps/lib/std/operators/shl.nyash",
                "apps/lib/std/operators/shr.nyash",
                "apps/lib/std/operators/bitand.nyash",
                "apps/lib/std/operators/bitor.nyash",
                "apps/lib/std/operators/bitxor.nyash",
                "apps/lib/std/operators/neg.nyash",
                "apps/lib/std/operators/not.nyash",
                "apps/lib/std/operators/bitnot.nyash",
            ];
            for rel in rels {
                let p = std::path::Path::new(&root).join(rel);
                if p.exists() {
                    let path = p.to_string_lossy().to_string();
                    if !out.iter().any(|x| x == &path) {
                        out.push(path);
                    }
                }
            }
        }
    }
    Ok((cleaned, out, alias_pairs))
}

/// Parse prelude source files into ASTs (single helper for all runner modes).
/// - Reads each path, strips nested `using`, and parses to AST.
/// - Returns a Vec of Program ASTs (one per prelude file), preserving DFS order.
pub fn parse_preludes_to_asts(
    runner: &NyashRunner,
    prelude_paths: &[String],
) -> Result<Vec<(String, nyash_rust::ast::ASTNode)>, String> {
    let mut out: Vec<(String, nyash_rust::ast::ASTNode)> = Vec::with_capacity(prelude_paths.len());
    for prelude_path in prelude_paths {
        let src = std::fs::read_to_string(prelude_path)
            .map_err(|e| format!("using: error reading {}: {}", prelude_path, e))?;
        let (clean_src, _nested, _aliases) = collect_using_and_strip(runner, &src, prelude_path)?;
        match crate::parser::NyashParser::parse_from_string(&clean_src) {
            Ok(ast) => out.push((prelude_path.clone(), ast)),
            Err(e) => return Err(format!(
                "Parse error in using prelude {}: {}",
                prelude_path, e
            )),
        }
    }
    Ok(out)
}

/// Merge prelude ASTs with the main AST into a single Program node.
/// - Collects statements from each prelude Program in order, then appends
///   statements from the main Program.
/// - If the main AST is not a Program, returns it unchanged (defensive).
pub fn merge_prelude_asts_with_main(
    prelude_asts: Vec<nyash_rust::ast::ASTNode>,
    main_ast: &nyash_rust::ast::ASTNode,
) -> nyash_rust::ast::ASTNode {
    use nyash_rust::ast::{ASTNode, Span};
    let mut combined: Vec<ASTNode> = Vec::new();
    for a in prelude_asts.into_iter() {
        if let ASTNode::Program { statements, .. } = a {
            combined.extend(statements);
        }
    }
    if let ASTNode::Program { statements, .. } = main_ast.clone() {
        let mut all = combined;
        all.extend(statements);
        ASTNode::Program { statements: all, span: Span::unknown() }
    } else {
        // Defensive: unexpected shape; preserve main AST unchanged.
        main_ast.clone()
    }
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
