use crate::runner::NyashRunner;
use std::collections::HashSet;

/// Strip `using` lines and register modules/aliases into the runtime registry.
/// Returns cleaned source. No-op when `NYASH_ENABLE_USING` is not set.
#[allow(dead_code)]
pub fn strip_using_and_register(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<String, String> {
    if !crate::config::env::enable_using() {
        return Ok(code.to_string());
    }
    // Optional external combiner (default OFF): NYASH_USING_COMBINER=1
    if std::env::var("NYASH_USING_COMBINER").ok().as_deref() == Some("1") {
        let fix_braces = std::env::var("NYASH_RESOLVE_FIX_BRACES").ok().as_deref() == Some("1");
        let dedup_box = std::env::var("NYASH_RESOLVE_DEDUP_BOX").ok().as_deref() == Some("1");
        let dedup_fn = std::env::var("NYASH_RESOLVE_DEDUP_FN").ok().as_deref() == Some("1");
        let seam_dbg = std::env::var("NYASH_RESOLVE_SEAM_DEBUG").ok().as_deref() == Some("1");
        let mut cmd = std::process::Command::new("python3");
        cmd.arg("tools/using_combine.py")
            .arg("--entry").arg(filename);
        if fix_braces { cmd.arg("--fix-braces"); }
        if dedup_box { cmd.arg("--dedup-box"); }
        if dedup_fn { cmd.arg("--dedup-fn"); }
        if seam_dbg { cmd.arg("--seam-debug"); }
        match cmd.output() {
            Ok(out) => {
                if out.status.success() {
                    let combined = String::from_utf8_lossy(&out.stdout).to_string();
                    return Ok(preexpand_at_local(&combined));
                } else {
                    let err = String::from_utf8_lossy(&out.stderr);
                    return Err(format!("using combiner failed: {}", err));
                }
            }
            Err(e) => {
                return Err(format!("using combiner spawn error: {}", e));
            }
        }
    }
    fn strip_and_inline(
        runner: &NyashRunner,
        code: &str,
        filename: &str,
        visited: &mut HashSet<String>,
    ) -> Result<String, String> {
        let mut out = String::with_capacity(code.len());
        let mut prelude = String::new();
        let mut used: Vec<(String, Option<String>)> = Vec::new();
        for line in code.lines() {
            let t = line.trim_start();
            if t.starts_with("using ") {
                crate::cli_v!("[using] stripped line: {}", line);
                let rest0 = t.strip_prefix("using ").unwrap().trim();
                let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
                let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                    (rest0[..pos].trim().to_string(), Some(rest0[pos + 4..].trim().to_string()))
                } else {
                    (rest0.to_string(), None)
                };
                let is_path = target.starts_with('"')
                    || target.starts_with("./")
                    || target.starts_with('/')
                    || target.ends_with(".nyash");
                if is_path {
                    let path = target.trim_matches('"').to_string();
                    let name = alias.clone().unwrap_or_else(|| {
                        std::path::Path::new(&path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("module")
                            .to_string()
                    });
                    used.push((name, Some(path)));
                } else {
                    used.push((target, alias));
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        // Register and inline
        let using_ctx = runner.init_using_context();
        let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
        let verbose = crate::config::env::cli_verbose();
        let ctx_dir = std::path::Path::new(filename).parent();
        let trace = verbose || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1");
        let seam_dbg = std::env::var("NYASH_RESOLVE_SEAM_DEBUG").ok().as_deref() == Some("1");
        if trace {
            eprintln!(
                "[using] ctx: modules={} using_paths={}",
                using_ctx.pending_modules.len(),
                using_ctx.using_paths.join(":")
            );
        }
        for (ns, alias_opt) in used {
            // Two forms:
            //  - using path "..." [as Alias] → handled earlier (stored as (name, Some(path)))
            //  - using namespace.with.dots [as Alias] → resolve ns → register alias → inline
            let resolved_path = if let Some(alias) = alias_opt {
                // alias case: resolve namespace to a concrete path
                let mut found: Option<String> = using_ctx
                    .pending_modules
                    .iter()
                    .find(|(n, _)| n == &ns)
                    .map(|(_, p)| p.clone());
                if trace {
                    if let Some(f) = &found {
                        eprintln!("[using] hit modules: {} -> {}", ns, f);
                    } else {
                        eprintln!("[using] miss modules: {}", ns);
                    }
                }
                if found.is_none() {
                    if let Ok(text) = std::fs::read_to_string("nyash.toml") {
                        if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                            if let Some(mut cur) = doc.get("modules").and_then(|v| v.as_table()) {
                                let mut segs = ns.split('.').peekable();
                                let mut hit: Option<String> = None;
                                while let Some(seg) = segs.next() {
                                    if let Some(next) = cur.get(seg) {
                                        if let Some(t) = next.as_table() {
                                            cur = t;
                                            continue;
                                        }
                                        if segs.peek().is_none() {
                                            if let Some(s) = next.as_str() {
                                                hit = Some(s.to_string());
                                            }
                                        }
                                    }
                                    break;
                                }
                                if hit.is_some() {
                                    if trace {
                                        eprintln!("[using] hit nyash.toml: {} -> {}", ns, hit.as_ref().unwrap());
                                    }
                                    found = hit;
                                }
                            }
                        }
                    }
                }
                if found.is_none() {
                    match crate::runner::pipeline::resolve_using_target(
                        &ns,
                        false,
                        &using_ctx.pending_modules,
                        &using_ctx.using_paths,
                        &using_ctx.aliases,
                        ctx_dir,
                        strict,
                        verbose,
                    ) {
                        Ok(v) => {
                            // Treat unchanged token (namespace) as unresolved
                            if v == ns {
                                found = None;
                            } else {
                                found = Some(v)
                            }
                        }
                        Err(e) => return Err(format!("using: {}", e)),
                    }
                }
                if let Some(value) = found.clone() {
                    let sb = crate::box_trait::StringBox::new(value.clone());
                    crate::runtime::modules_registry::set(alias.clone(), Box::new(sb));
                    let sb2 = crate::box_trait::StringBox::new(value.clone());
                    crate::runtime::modules_registry::set(ns.clone(), Box::new(sb2));
                } else if trace {
                    eprintln!("[using] still unresolved: {} as {}", ns, alias);
                }
                found
            } else {
                // direct namespace without alias
                match crate::runner::pipeline::resolve_using_target(
                    &ns,
                    false,
                    &using_ctx.pending_modules,
                    &using_ctx.using_paths,
                    &using_ctx.aliases,
                    ctx_dir,
                    strict,
                    verbose,
                ) {
                    Ok(value) => {
                        let sb = crate::box_trait::StringBox::new(value.clone());
                        crate::runtime::modules_registry::set(ns, Box::new(sb));
                        Some(value)
                    }
                    Err(e) => return Err(format!("using: {}", e)),
                }
            };
            if let Some(path) = resolved_path {
                // Resolve relative to current file dir
                // Guard: skip obvious namespace tokens (ns.ns without extension)
                if (!path.contains('/') && !path.contains('\\')) && !path.ends_with(".nyash") && path.contains('.') {
                    if verbose {
                        eprintln!("[using] unresolved '{}' (namespace token, skip inline)", path);
                    }
                    continue;
                }
                let mut p = std::path::PathBuf::from(&path);
                if p.is_relative() {
                    // If the raw relative path exists from CWD, use it.
                    // Otherwise, try relative to the current file's directory.
                    if !p.exists() {
                        if let Some(dir) = std::path::Path::new(filename).parent() {
                            let cand = dir.join(&p);
                            if cand.exists() {
                                p = cand;
                            }
                        }
                    }
                }
                // normalize to absolute to stabilize de-dup
                if let Ok(abs) = std::fs::canonicalize(&p) { p = abs; }
                let key = p.to_string_lossy().to_string();
                if visited.contains(&key) {
                    continue;
                }
                visited.insert(key.clone());
                if let Ok(text) = std::fs::read_to_string(&p) {
                    let inlined = strip_and_inline(runner, &text, &key, visited)?;
                    prelude.push_str(&inlined);
                    prelude.push_str("\n");
                    if seam_dbg {
                        let tail = inlined.chars().rev().take(120).collect::<String>().chars().rev().collect::<String>();
                        eprintln!("[using][seam][inlined] {} tail=<<<{}>>>", key, tail.replace('\n', "\\n"));
                    }
                } else if verbose {
                    eprintln!("[using] warn: could not read {}", p.display());
                }
            }
        }
        // Prepend inlined modules so their boxes are defined before use
        // Seam guard: collapse consecutive blank lines at the join (prelude || body) to a single blank line
        if prelude.is_empty() {
            return Ok(out);
        }
        // Optionally deduplicate repeated static boxes in prelude by name (default OFF)
        let mut prelude_text = prelude;
        if std::env::var("NYASH_RESOLVE_DEDUP_BOX").ok().as_deref() == Some("1") {
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut out_txt = String::with_capacity(prelude_text.len());
            let bytes: Vec<char> = prelude_text.chars().collect();
            let mut i = 0usize;
            while i < bytes.len() {
                // naive scan for "static box "
                if i + 12 < bytes.len() && bytes[i..].iter().take(11).collect::<String>() == "static box " {
                    // read name token
                    let mut j = i + 11;
                    let mut name = String::new();
                    while j < bytes.len() {
                        let c = bytes[j];
                        if c.is_alphanumeric() || c == '_' { name.push(c); j += 1; } else { break; }
                    }
                    // find opening brace '{'
                    while j < bytes.len() && bytes[j].is_whitespace() { j += 1; }
                    if j < bytes.len() && bytes[j] == '{' {
                        // scan to matching closing brace for this box
                        let mut k = j;
                        let mut depth = 0i32;
                        while k < bytes.len() {
                            let c = bytes[k];
                            if c == '{' { depth += 1; }
                            if c == '}' { depth -= 1; if depth == 0 { k += 1; break; } }
                            k += 1;
                        }
                        // decide
                        if seen.contains(&name) {
                            // skip duplicate box
                            i = k; // drop this block
                            continue;
                        } else {
                            seen.insert(name);
                            // keep this block as-is
                            out_txt.push_str(&bytes[i..k].iter().collect::<String>());
                            i = k;
                            continue;
                        }
                    }
                }
                // default: copy one char
                out_txt.push(bytes[i]);
                i += 1;
            }
            prelude_text = out_txt;
        }
        // Optional: de-duplicate repeated function definitions inside specific boxes (default OFF)
        if std::env::var("NYASH_RESOLVE_DEDUP_FN").ok().as_deref() == Some("1") {
            // Currently target MiniVmPrints.print_prints_in_slice only (low risk)
            let mut out_txt = String::with_capacity(prelude_text.len());
            let bytes: Vec<char> = prelude_text.chars().collect();
            let mut i = 0usize;
            while i < bytes.len() {
                // scan for "static box "
                let ahead: String = bytes[i..bytes.len().min(i + 12)].iter().collect();
                if ahead.starts_with("static box ") {
                    // parse box name
                    let mut j = i + 11; // len("static box ") == 11
                    let mut name = String::new();
                    while j < bytes.len() {
                        let c = bytes[j];
                        if c.is_ascii_alphanumeric() || c == '_' { name.push(c); j += 1; } else { break; }
                    }
                    // skip ws to '{'
                    while j < bytes.len() && bytes[j].is_whitespace() { j += 1; }
                    if j < bytes.len() && bytes[j] == '{' {
                        // find matching closing '}' for the box body
                        let mut k = j;
                        let mut depth = 0i32;
                        let mut in_str = false;
                        while k < bytes.len() {
                            let c = bytes[k];
                            if in_str {
                                if c == '\\' { k += 2; continue; }
                                if c == '"' { in_str = false; }
                                k += 1;
                                continue;
                            } else {
                                if c == '"' { in_str = true; k += 1; continue; }
                                if c == '{' { depth += 1; }
                                if c == '}' { depth -= 1; if depth == 0 { k += 1; break; } }
                                k += 1;
                            }
                        }
                        // write header up to body start '{'
                        out_txt.push_str(&bytes[i..(j + 1)].iter().collect::<String>());
                        // process body (limited dedup for MiniVmPrints.print_prints_in_slice)
                        let body_end = k.saturating_sub(1);
                        if name == "MiniVmPrints" {
                            let mut kept = false;
                            let mut p = j + 1;
                            while p <= body_end {
                                // find next line start
                                let mut ls = p;
                                if ls > j + 1 {
                                    while ls <= body_end && bytes[ls - 1] != '\n' { ls += 1; }
                                }
                                if ls > body_end { break; }
                                // skip spaces
                                let mut q = ls;
                                while q <= body_end && bytes[q].is_whitespace() && bytes[q] != '\n' { q += 1; }
                                // check for function definition of print_prints_in_slice
                                let rem: String = bytes[q..(body_end + 1).min(q + 64)].iter().collect();
                                if rem.starts_with("print_prints_in_slice(") {
                                    // find ')'
                                    let mut r = q;
                                    let mut dp = 0i32;
                                    let mut in_s = false;
                                    while r <= body_end {
                                        let c = bytes[r];
                                        if in_s { if c == '\\' { r += 2; continue; } if c == '"' { in_s = false; } r += 1; continue; }
                                        if c == '"' { in_s = true; r += 1; continue; }
                                        if c == '(' { dp += 1; r += 1; continue; }
                                        if c == ')' { dp -= 1; r += 1; if dp <= 0 { break; } continue; }
                                        r += 1;
                                    }
                                    while r <= body_end && bytes[r].is_whitespace() { r += 1; }
                                    if r <= body_end && bytes[r] == '{' {
                                        // find body end
                                        let mut t = r;
                                        let mut d2 = 0i32;
                                        let mut in_s2 = false;
                                        while t <= body_end {
                                            let c2 = bytes[t];
                                            if in_s2 { if c2 == '\\' { t += 2; continue; } if c2 == '"' { in_s2 = false; } t += 1; continue; }
                                            if c2 == '"' { in_s2 = true; t += 1; continue; }
                                            if c2 == '{' { d2 += 1; }
                                            if c2 == '}' { d2 -= 1; if d2 == 0 { t += 1; break; } }
                                            t += 1;
                                        }
                                        // start-of-line
                                        let mut sol = q;
                                        while sol > j + 1 && bytes[sol - 1] != '\n' { sol -= 1; }
                                        if !kept {
                                            out_txt.push_str(&bytes[sol..t].iter().collect::<String>());
                                            kept = true;
                                        }
                                        p = t;
                                        continue;
                                    }
                                }
                                // copy this line
                                let mut eol = ls;
                                while eol <= body_end && bytes[eol] != '\n' { eol += 1; }
                                out_txt.push_str(&bytes[ls..(eol.min(body_end + 1))].iter().collect::<String>());
                                if eol <= body_end && bytes[eol] == '\n' { out_txt.push('\n'); }
                                p = eol + 1;
                            }
                        } else {
                            // copy body as-is
                            out_txt.push_str(&bytes[(j + 1)..=body_end].iter().collect::<String>());
                        }
                        // write closing '}'
                        out_txt.push('}');
                        i = k;
                        continue;
                    }
                }
                // default: copy one char
                out_txt.push(bytes[i]);
                i += 1;
            }
            prelude_text = out_txt;
        }
        let prelude_clean = prelude_text.trim_end_matches(['\n', '\r']);
        if seam_dbg {
            let tail = prelude_clean.chars().rev().take(160).collect::<String>().chars().rev().collect::<String>();
            let head = out.chars().take(160).collect::<String>();
            eprintln!("[using][seam] prelude_tail=<<<{}>>>", tail.replace('\n', "\\n"));
            eprintln!("[using][seam] body_head   =<<<{}>>>", head.replace('\n', "\\n"));
        }
        let mut combined = String::with_capacity(prelude_clean.len() + out.len() + 1);
        combined.push_str(prelude_clean);
        combined.push('\n');
        // Optional seam safety: append missing '}' for unmatched '{' in prelude
        if std::env::var("NYASH_RESOLVE_FIX_BRACES").ok().as_deref() == Some("1") {
            // compute { } delta ignoring strings and comments
            let mut delta: i32 = 0;
            let mut it = prelude_clean.chars().peekable();
            let mut in_str = false;
            let mut in_sl = false;
            let mut in_ml = false;
            while let Some(c) = it.next() {
                if in_sl {
                    if c == '\n' { in_sl = false; }
                    continue;
                }
                if in_ml {
                    if c == '*' {
                        if let Some('/') = it.peek().copied() {
                            // consume '/'
                            it.next();
                            in_ml = false;
                        }
                    }
                    continue;
                }
                if in_str {
                    if c == '\\' { it.next(); continue; }
                    if c == '"' { in_str = false; }
                    continue;
                }
                if c == '"' { in_str = true; continue; }
                if c == '/' {
                    match it.peek().copied() {
                        Some('/') => { in_sl = true; it.next(); continue; }
                        Some('*') => { in_ml = true; it.next(); continue; }
                        _ => {}
                    }
                }
                if c == '{' { delta += 1; }
                if c == '}' { delta -= 1; }
            }
            if delta > 0 {
                if trace { eprintln!("[using][seam] fix: appending {} '}}' before body", delta); }
                for _ in 0..delta { combined.push('}'); combined.push('\n'); }
            }
        }
        combined.push_str(&out);
        Ok(combined)
    }
    let mut visited = HashSet::new();
    let combined = strip_and_inline(runner, code, filename, &mut visited)?;
    // Dev sugar: always pre-expand @name[:T] = expr at line-head to keep sources readable
    Ok(preexpand_at_local(&combined))
}

/// Pre-expand line-head `@name[: Type] = expr` into `local name[: Type] = expr`.
/// Minimal, safe, no semantics change. Applies only at line head (after spaces/tabs).
pub(crate) fn preexpand_at_local(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') { i += 1; }
        if i < bytes.len() && bytes[i] == b'@' {
            // parse identifier
            let mut j = i + 1;
            // first char [A-Za-z_]
            if j < bytes.len() && ((bytes[j] as char).is_ascii_alphabetic() || bytes[j] == b'_') {
                j += 1;
                while j < bytes.len() {
                    let c = bytes[j] as char;
                    if c.is_ascii_alphanumeric() || c == '_' { j += 1; } else { break; }
                }
                // optional type: spaces ':' spaces ident
                let mut k = j;
                while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') { k += 1; }
                if k < bytes.len() && bytes[k] == b':' {
                    k += 1;
                    while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') { k += 1; }
                    // simple type ident
                    if k < bytes.len() && ((bytes[k] as char).is_ascii_alphabetic() || bytes[k] == b'_') {
                        k += 1;
                        while k < bytes.len() {
                            let c = bytes[k] as char;
                            if c.is_ascii_alphanumeric() || c == '_' { k += 1; } else { break; }
                        }
                    }
                }
                // consume spaces to '='
                let mut eqp = k;
                while eqp < bytes.len() && (bytes[eqp] == b' ' || bytes[eqp] == b'\t') { eqp += 1; }
                if eqp < bytes.len() && bytes[eqp] == b'=' {
                    // build transformed line: prefix + 'local ' + rest from after '@' up to '=' + ' =' + remainder
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
