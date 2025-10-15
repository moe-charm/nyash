use crate::runner::NyashRunner;

use super::collect::collect_using_and_strip;

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
    let mut out_alias_pairs: Vec<(String, String)> = alias_pairs.clone();
    let mut alias_seen: std::collections::HashSet<(String, String)> =
        alias_pairs.iter().cloned().collect();
    // Aliasはトップレベル using のみに適用（ネストは元名を維持）
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
        out_alias_pairs: &mut Vec<(String, String)>,
        alias_seen: &mut std::collections::HashSet<(String, String)>,
    ) -> Result<(), String> {
        let (key, real_path) = normalize_path(path);
        if !seen.insert(key.clone()) {
            return Ok(());
        }
        let src = std::fs::read_to_string(&real_path)
            .map_err(|e| format!("using: failed to read '{}': {}", real_path, e))?;
        let (_cleaned, nested, local_aliases) = collect_using_and_strip(runner, &src, &real_path)?;
        // Accumulate nested alias pairs as well (so we can rename/desugar inside prelude code)
        for (a, c) in local_aliases.into_iter() {
            if alias_seen.insert((a.clone(), c.clone())) {
                out_alias_pairs.push((a, c));
            }
        }
        // Exclude legacy selfhost path to avoid conflicts with new selfhost-compiler tree
        let is_legacy = real_path.contains("/apps/selfhost/compiler/");
        if !is_legacy {
            // Guard: avoid duplicate insertion into `out` even if canonicalization
            // varied across DFS entry points.
            if !out.iter().any(|p| p == &real_path) {
                out.push(real_path.clone());
            }
        }
        for n in nested.iter() {
            dfs(runner, n, out, seen, out_alias_pairs, alias_seen)?;
        }
        Ok(())
    }
    for p in direct.iter() {
        dfs(runner, p, &mut out, &mut seen, &mut out_alias_pairs, &mut alias_seen)?;
    }
    // Operator Boxes prelude injection（観測"常時ON"のため）
    // stringify/compare/add は常に注入（存在時）。その他（bitwise等）は ALL 指定時のみ。
    // Opt-out for smokes/dev: set NYASH_OPERATOR_BOX_PRELUDE=0|off|false to skip injection.
    let prelude_enabled = match std::env::var("NYASH_OPERATOR_BOX_PRELUDE").ok().as_deref() {
        Some(v) if ["0","off","false"].contains(&v.to_ascii_lowercase().as_str()) => false,
        _ => true,
    };
    let opbox_all = std::env::var("NYASH_OPERATOR_BOX_ALL").ok().as_deref() == Some("1")
        || std::env::var("NYASH_BUILDER_OPERATOR_BOX_ALL_CALL").ok().as_deref() == Some("1");

    if prelude_enabled {
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
    Ok((cleaned, out, out_alias_pairs))
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
