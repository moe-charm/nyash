use crate::runner::NyashRunner;
use std::collections::HashSet;

/// Generate content for built-in namespaces like builtin:nyashstd
fn generate_builtin_namespace_content(namespace_key: &str) -> String {
    match namespace_key {
        "builtin:nyashstd" => {
            // Generate Nyash code that provides nyashstd functionality
            // This exposes the built-in stdlib boxes as regular Nyash static boxes
            format!(r#"
// Built-in nyashstd namespace (auto-generated)
static box string {{
    create(text) {{
        return new StringBox(text)
    }}
    upper(str) {{
        return new StringBox(str.upper())
    }}
}}

static box integer {{
    create(value) {{
        return new IntegerBox(value)
    }}
}}

static box bool {{
    create(value) {{
        return new BoolBox(value)
    }}
}}

static box array {{
    create() {{
        return new ArrayBox()
    }}
}}

static box console {{
    log(message) {{
        print(message)
        return null
    }}
}}
"#)
        }
        _ => {
            // Unknown built-in namespace
            format!("// Unknown built-in namespace: {}\n", namespace_key)
        }
    }
}

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
        let fix_braces = crate::config::env::resolve_fix_braces();
        let dedup_box = std::env::var("NYASH_RESOLVE_DEDUP_BOX").ok().as_deref() == Some("1");
        let dedup_fn = std::env::var("NYASH_RESOLVE_DEDUP_FN").ok().as_deref() == Some("1");
        let seam_dbg = std::env::var("NYASH_RESOLVE_SEAM_DEBUG").ok().as_deref() == Some("1");
        let mut cmd = std::process::Command::new("python3");
        cmd.arg("tools/using_combine.py").arg("--entry").arg(filename);
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
            Err(e) => return Err(format!("using combiner spawn error: {}", e)),
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
                // Strip trailing inline comments
                let rest0 = rest0.split('#').next().unwrap_or(rest0).trim();
                let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
                let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                    (rest0[..pos].trim().to_string(), Some(rest0[pos + 4..].trim().to_string()))
                } else { (rest0.to_string(), None) };
                let is_path = target.starts_with('"') || target.starts_with("./") || target.starts_with('/') || target.ends_with(".nyash");
                if is_path {
                    let path = target.trim_matches('"').to_string();
                    let name = alias.clone().unwrap_or_else(|| {
                        std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string()
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
            //  - using path "..." [as Alias]
            //  - using namespace.with.dots [as Alias]
            let resolved_path = if let Some(alias_or_path) = alias_opt {
                // Disambiguate: when alias_opt looks like a file path, treat it as direct path.
                let is_path_hint = alias_or_path.ends_with(".nyash")
                    || alias_or_path.contains('/')
                    || alias_or_path.contains('\\');
                if is_path_hint {
                    // Direct path provided (e.g., using "path/file.nyash" as Name)
                    let value = alias_or_path.clone();
                    // Register: Name -> path
                    let sb = crate::box_trait::StringBox::new(value.clone());
                    crate::runtime::modules_registry::set(ns.clone(), Box::new(sb));
                    Some(value)
                } else {
                    // alias string for a namespace (e.g., using ns.token as Alias)
                    let alias = alias_or_path;
                    // alias case: resolve namespace to a concrete path
                    let mut found: Option<String> = using_ctx
                        .pending_modules
                        .iter()
                        .find(|(n, _)| n == &ns)
                        .map(|(_, p)| p.clone());
                    if trace {
                        if let Some(f) = &found {
                            eprintln!("[using/resolve] alias '{}' -> '{}'", ns, f);
                        }
                    }
                    if found.is_none() {
                        match crate::runner::pipeline::resolve_using_target(
                            &ns,
                            false,
                            &using_ctx.pending_modules,
                            &using_ctx.using_paths,
                            &using_ctx.aliases,
                            &using_ctx.packages,
                            ctx_dir,
                            strict,
                            verbose,
                        ) {
                            Ok(v) => {
                                // Treat unchanged token (namespace) as unresolved
                                if v == ns { found = None; } else { found = Some(v) }
                            }
                            Err(e) => return Err(format!("using: {}", e)),
                        }
                    }
                    if let Some(value) = found.clone() {
                        let sb = crate::box_trait::StringBox::new(value.clone());
                        crate::runtime::modules_registry::set(alias.clone(), Box::new(sb));
                        let sb2 = crate::box_trait::StringBox::new(value.clone());
                        crate::runtime::modules_registry::set(ns.clone(), Box::new(sb2));
                        // Optional: autoload dylib when using kind="dylib" and NYASH_USING_DYLIB_AUTOLOAD=1
                        if value.starts_with("dylib:") && std::env::var("NYASH_USING_DYLIB_AUTOLOAD").ok().as_deref() == Some("1") {
                            let lib_path = value.trim_start_matches("dylib:");
                            // Derive lib name from file stem (strip leading 'lib')
                            let p = std::path::Path::new(lib_path);
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                let mut lib_name = stem.to_string();
                                if lib_name.starts_with("lib") { lib_name = lib_name.trim_start_matches("lib").to_string(); }
                                // Determine box list from using packages (prefer [using.<ns>].bid)
                                let mut boxes: Vec<String> = Vec::new();
                                if let Some(pkg) = using_ctx.packages.get(&ns) {
                                    if let Some(b) = &pkg.bid { boxes.push(b.clone()); }
                                }
                                if verbose { eprintln!("[using] autoload dylib: {} as {} boxes=[{}]", lib_path, lib_name, boxes.join(",")); }
                                let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                                let _ = host.read().unwrap().load_library_direct(&lib_name, lib_path, &boxes);
                            }
                        }
                    } else if trace {
                        eprintln!("[using] still unresolved: {} as {}", ns, alias);
                    }
                    found
                }
            } else {
                // direct namespace without alias
                match crate::runner::pipeline::resolve_using_target(
                    &ns,
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
                        let sb = crate::box_trait::StringBox::new(value.clone());
                        let ns_clone = ns.clone();
                        crate::runtime::modules_registry::set(ns_clone, Box::new(sb));
                        // Optional: autoload dylib when using kind="dylib"
                        if value.starts_with("dylib:") && std::env::var("NYASH_USING_DYLIB_AUTOLOAD").ok().as_deref() == Some("1") {
                            let lib_path = value.trim_start_matches("dylib:");
                            let p = std::path::Path::new(lib_path);
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                let mut lib_name = stem.to_string();
                                if lib_name.starts_with("lib") { lib_name = lib_name.trim_start_matches("lib").to_string(); }
                                let mut boxes: Vec<String> = Vec::new();
                                if let Some(pkg) = using_ctx.packages.get(&ns) {
                                    if let Some(b) = &pkg.bid { boxes.push(b.clone()); }
                                }
                                if verbose { eprintln!("[using] autoload dylib: {} as {} boxes=[{}]", lib_path, lib_name, boxes.join(",")); }
                                let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                                let _ = host.read().unwrap().load_library_direct(&lib_name, lib_path, &boxes);
                            }
                        }
                        Some(value)
                    }
                    Err(e) => return Err(format!("using: {}", e)),
                }
            };
            if let Some(path) = resolved_path {
                // Resolve relative to current file dir
                // Guard: skip obvious namespace tokens (ns.ns without extension)
                if (!path.contains('/') && !path.contains('\\')) && !path.ends_with(".nyash") && path.contains('.') {
                    if verbose { eprintln!("[using] unresolved '{}' (namespace token, skip inline)", path); }
                    continue;
                }
                let mut p = std::path::PathBuf::from(&path);
                if p.is_relative() {
                    if !p.exists() {
                        if let Some(dir) = std::path::Path::new(filename).parent() {
                            let cand = dir.join(&p);
                            if cand.exists() { p = cand; }
                        }
                    }
                }
                if let Ok(abs) = std::fs::canonicalize(&p) { p = abs; }
                let key = p.to_string_lossy().to_string();
                if visited.contains(&key) { 
                    if verbose { eprintln!("[using] skipping already visited: {}", key); }
                    continue; 
                }
                visited.insert(key.clone());
                if let Ok(text) = std::fs::read_to_string(&p) {
                    let inlined = strip_and_inline(runner, &text, &key, visited)?;
                    prelude.push_str(&inlined);
                    prelude.push_str("\n");
                    crate::runner::modes::common_util::resolve::seam::log_inlined_tail(&key, &inlined, seam_dbg);
                } else if key.starts_with("builtin:") {
                    // Handle built-in namespaces like builtin:nyashstd
                    let builtin_content = generate_builtin_namespace_content(&key);
                    prelude.push_str(&builtin_content);
                    prelude.push_str("\n");
                    if verbose {
                        eprintln!("[using] loaded builtin namespace: {}", key);
                    }
                } else if verbose {
                    eprintln!("[using] warn: could not read {}", p.display());
                }
            }
        }
        if prelude.is_empty() { return Ok(out); }
        // Optional de-dup of static boxes by name
        let mut prelude_text = prelude;
        if std::env::var("NYASH_RESOLVE_DEDUP_BOX").ok().as_deref() == Some("1") {
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut out_txt = String::with_capacity(prelude_text.len());
            let bytes: Vec<char> = prelude_text.chars().collect();
            let mut i = 0usize;
            while i < bytes.len() {
                if i + 12 < bytes.len() && bytes[i..].iter().take(11).collect::<String>() == "static box " {
                    let mut j = i + 11;
                    let mut name = String::new();
                    while j < bytes.len() {
                        let c = bytes[j];
                        if c.is_alphanumeric() || c == '_' { name.push(c); j += 1; } else { break; }
                    }
                    while j < bytes.len() && bytes[j].is_whitespace() { j += 1; }
                    if j < bytes.len() && bytes[j] == '{' {
                        let mut k = j;
                        let mut depth = 0i32;
                        while k < bytes.len() {
                            let c = bytes[k];
                            if c == '{' { depth += 1; }
                            if c == '}' { depth -= 1; if depth == 0 { k += 1; break; } }
                            k += 1;
                        }
                        if seen.contains(&name) { i = k; continue; } else {
                            seen.insert(name);
                            out_txt.push_str(&bytes[i..k].iter().collect::<String>());
                            i = k; continue;
                        }
                    }
                }
                out_txt.push(bytes[i]);
                i += 1;
            }
            prelude_text = out_txt;
        }
        // Optional: function dedup (MiniVmPrints.print_prints_in_slice)
        if std::env::var("NYASH_RESOLVE_DEDUP_FN").ok().as_deref() == Some("1") {
            let mut out_txt = String::with_capacity(prelude_text.len());
            let bytes: Vec<char> = prelude_text.chars().collect();
            let mut i = 0usize;
            while i < bytes.len() {
                let ahead: String = bytes[i..bytes.len().min(i + 12)].iter().collect();
                if ahead.starts_with("static box ") {
                    let mut j = i + 11;
                    let mut name = String::new();
                    while j < bytes.len() { let c = bytes[j]; if c.is_ascii_alphanumeric() || c == '_' { name.push(c); j += 1; } else { break; } }
                    while j < bytes.len() && bytes[j].is_whitespace() { j += 1; }
                    if j < bytes.len() && bytes[j] == '{' {
                        let mut k = j;
                        let mut depth = 0i32;
                        let mut in_str = false;
                        while k < bytes.len() {
                            let c = bytes[k];
                            if in_str { if c == '\\' { k += 2; continue; } if c == '"' { in_str = false; } k += 1; continue; } else { if c == '"' { in_str = true; k += 1; continue; } if c == '{' { depth += 1; } if c == '}' { depth -= 1; if depth == 0 { k += 1; break; } } k += 1; }
                        }
                        out_txt.push_str(&bytes[i..(j + 1)].iter().collect::<String>());
                        let body_end = k.saturating_sub(1);
                        if name == "MiniVmPrints" {
                            let mut kept = false;
                            let mut p = j + 1;
                            while p <= body_end {
                                let mut ls = p; if ls > j + 1 { while ls <= body_end && bytes[ls - 1] != '\n' { ls += 1; } }
                                if ls > body_end { break; }
                                let mut q = ls; while q <= body_end && bytes[q].is_whitespace() && bytes[q] != '\n' { q += 1; }
                                let rem: String = bytes[q..(body_end + 1).min(q + 64)].iter().collect();
                                if rem.starts_with("print_prints_in_slice(") {
                                    let mut r = q; let mut dp = 0i32; let mut instr = false;
                                    while r <= body_end {
                                        let c = bytes[r];
                                        if instr { if c == '\\' { r += 2; continue; } if c == '"' { instr = false; } r += 1; continue; }
                                        if c == '"' { instr = true; r += 1; continue; }
                                        if c == '(' { dp += 1; }
                                        if c == ')' { dp -= 1; if dp == 0 { r += 1; break; } }
                                        if dp == 0 && c == '{' { break; }
                                        r += 1;
                                    }
                                    while r <= body_end && bytes[r].is_whitespace() { r += 1; }
                                    if r <= body_end && bytes[r] == '{' {
                                        let mut s = r; let mut bd = 0i32; let mut is2 = false;
                                        while s <= body_end {
                                            let c = bytes[s];
                                            if is2 { if c == '\\' { s += 2; continue; } if c == '"' { is2 = false; } s += 1; continue; }
                                            if c == '"' { is2 = true; s += 1; continue; }
                                            if c == '{' { bd += 1; }
                                            if c == '}' { bd -= 1; if bd == 0 { s += 1; break; } }
                                            s += 1;
                                        }
                                        if !kept {
                                            out_txt.push_str(&bytes[q..s].iter().collect::<String>());
                                            kept = true;
                                        }
                                        // advance outer scanner to the end of this function body
                                        i = s;
                                        let _ = i; // mark as read to satisfy unused_assignments lint
                                        continue;
                                    }
                                }
                                out_txt.push(bytes[p]); p += 1;
                            }
                            if !kept { out_txt.push_str(&bytes[j + 1..=body_end].iter().collect::<String>()); }
                            out_txt.push('}'); out_txt.push('\n'); i = k; continue;
                        } else { out_txt.push_str(&bytes[j + 1..k].iter().collect::<String>()); i = k; continue; }
                    }
                }
                out_txt.push(bytes[i]); i += 1;
            }
            prelude_text = out_txt;
        }
        // Seam join + optional fix
        let prelude_clean = prelude_text.trim_end_matches('\n');
        crate::runner::modes::common_util::resolve::seam::log_prelude_body_seam(prelude_clean, &out, seam_dbg);
        let mut combined = String::with_capacity(prelude_clean.len() + out.len() + 1);
        combined.push_str(prelude_clean);
        combined.push('\n');
        crate::runner::modes::common_util::resolve::seam::fix_prelude_braces_if_enabled(prelude_clean, &mut combined, trace);
        combined.push_str(&out);
        if std::env::var("NYASH_RESOLVE_SEAM_DEBUG").ok().as_deref() == Some("1") {
            let _ = std::fs::write("/tmp/nyash_using_combined.nyash", &combined);
        }
        Ok(combined)
    }

    let mut visited = HashSet::new();
    let combined = strip_and_inline(runner, code, filename, &mut visited)?;
    Ok(preexpand_at_local(&combined))
}

/// Pre-expand line-head `@name[: Type] = expr` into `local name[: Type] = expr`.
/// Minimal, safe, no semantics change. Applies only at line head (after spaces/tabs).
pub fn preexpand_at_local(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') { i += 1; }
        if i < bytes.len() && bytes[i] == b'@' {
            // parse identifier
            let mut j = i + 1;
            if j < bytes.len() && ((bytes[j] as char).is_ascii_alphabetic() || bytes[j] == b'_') {
                j += 1;
                while j < bytes.len() { let c = bytes[j] as char; if c.is_ascii_alphanumeric() || c == '_' { j += 1; } else { break; } }
                let mut k = j; while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') { k += 1; }
                if k < bytes.len() && bytes[k] == b':' {
                    k += 1; while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') { k += 1; }
                    if k < bytes.len() && ((bytes[k] as char).is_ascii_alphabetic() || bytes[k] == b'_') {
                        k += 1; while k < bytes.len() { let c = bytes[k] as char; if c.is_ascii_alphanumeric() || c == '_' { k += 1; } else { break; } }
                    }
                }
                let mut eqp = k; while eqp < bytes.len() && (bytes[eqp] == b' ' || bytes[eqp] == b'\t') { eqp += 1; }
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
