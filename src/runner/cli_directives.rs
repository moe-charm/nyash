/*!
 * CLI Directives Scanner — early source comments to env plumbing
 *
 * Supports lightweight, file-scoped directives placed in the first lines
 * of a Nyash source file. Current directives:
 *  - // @env KEY=VALUE         → export KEY=VALUE into process env
 *  - // @plugin-builtins      → NYASH_USE_PLUGIN_BUILTINS=1
 *  - // @jit-debug            → enable common JIT debug flags (no-op if JIT unused)
 *  - // @jit-strict           → strict JIT flags (no VM fallback) for experiments
 *
 * Also runs the "fields-at-top" lint delegated to pipeline::lint_fields_top.
 */

pub(super) fn apply_cli_directives_from_source(
    code: &str,
    strict_fields: bool,
    verbose: bool,
) -> Result<(), String> {
    // Scan only the header area (up to the first non-comment content line)
    for (i, line) in code.lines().take(128).enumerate() {
        let l = line.trim();
        if !(l.starts_with("//") || l.starts_with("#!") || l.is_empty()) {
            if i > 0 {
                break;
            }
        }
        if let Some(rest) = l.strip_prefix("//") {
            let rest = rest.trim();
            if let Some(dir) = rest.strip_prefix("@env ") {
                if let Some((k, v)) = dir.split_once('=') {
                    let key = k.trim();
                    let val = v.trim();
                    if !key.is_empty() {
                        std::env::set_var(key, val);
                    }
                }
            } else if rest == "@plugin-builtins" {
                std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
            } else if rest == "@jit-debug" {
                // Safe even if JIT is disabled elsewhere; treated as no-op flags
                std::env::set_var("NYASH_JIT_EXEC", "1");
                std::env::set_var("NYASH_JIT_THRESHOLD", "1");
                std::env::set_var("NYASH_JIT_EVENTS", "1");
                std::env::set_var("NYASH_JIT_EVENTS_COMPILE", "1");
                std::env::set_var("NYASH_JIT_EVENTS_RUNTIME", "1");
                std::env::set_var("NYASH_JIT_SHIM_TRACE", "1");
            } else if rest == "@jit-strict" {
                std::env::set_var("NYASH_JIT_STRICT", "1");
                std::env::set_var("NYASH_JIT_ARGS_HANDLE_ONLY", "1");
                if std::env::var("NYASH_JIT_ONLY").ok().is_none() {
                    std::env::set_var("NYASH_JIT_ONLY", "1");
                }
            }
        }
    }

    // Lint: enforce fields at top-of-box (delegated)
    super::pipeline::lint_fields_top(code, strict_fields, verbose)?;

    // Dev-only guards (strict but opt-in via env)
    // 1) ASI strict: disallow binary operator at end-of-line (line continuation)
    if std::env::var("NYASH_ASI_STRICT").ok().as_deref() == Some("1") {
        // operators to check (suffixes)
        const OP2: [&str; 6] = ["==", "!=", "<=", ">=", "&&", "||"];
        const OP1: [&str; 7] = ["+", "-", "*", "/", "%", "<", ">"]; 
        for (i, line) in code.lines().enumerate() {
            let l = line.trim_end();
            if l.is_empty() { continue; }
            let mut bad = false;
            for op in OP2.iter() { if l.ends_with(op) { bad = true; break; } }
            if !bad { for op in OP1.iter() { if l.ends_with(op) { bad = true; break; } } }
            if bad {
                return Err(format!("Parse error: Strict ASI violation — line {} ends with operator", i + 1));
            }
        }
    }

    // 2) '+' mixed types (String and Number) error (opt-in)
    if std::env::var("NYASH_PLUS_MIX_ERROR").ok().as_deref() == Some("1") {
        for (i, line) in code.lines().enumerate() {
            if let Some((ltok, rtok)) = find_plus_operands(line) {
                let left_is_num = is_number_literal(ltok);
                let right_is_str = is_string_literal(rtok);
                let left_is_str = is_string_literal(ltok);
                let right_is_num = is_number_literal(rtok);
                if (left_is_num && right_is_str) || (left_is_str && right_is_num) {
                    return Err(format!("Type error: '+' mixed String and Number at line {} (use str()/explicit conversion)", i + 1));
                }
            }
        }
    }

    // 3) '==' on likely Box variables: emit guidance to use equals() (opt-in)
    if std::env::var("NYASH_BOX_EQ_GUIDE_ERROR").ok().as_deref() == Some("1") {
        for (i, line) in code.lines().enumerate() {
            let l = line;
            let mut idx = 0usize;
            while let Some(pos) = l[idx..].find("==") {
                let at = idx + pos;
                // find left token end and right token start
                let (left_ok, right_ok) = (peek_ident_left(l, at), peek_ident_right(l, at + 2));
                if left_ok && right_ok {
                    return Err(format!("Type error: '==' on boxes — use equals() (line {})", i + 1));
                }
                idx = at + 2;
            }
        }
    }

    Ok(())
}

// ---- Helpers (very small, no external deps) ----
fn is_number_literal(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && t.chars().all(|c| c.is_ascii_digit())
}
fn is_string_literal(s: &str) -> bool {
    let t = s.trim();
    t.starts_with('"')
}

fn find_plus_operands(line: &str) -> Option<(&str, &str)> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] as char == '+' {
            // extract left token
            let mut l = i;
            while l > 0 && bytes[l - 1].is_ascii_whitespace() { l -= 1; }
            let mut lstart = l;
            while lstart > 0 {
                let c = bytes[lstart - 1] as char;
                if c.is_ascii_alphanumeric() || c == '_' || c == '"' { lstart -= 1; } else { break; }
            }
            let left = &line[lstart..l];
            // extract right token
            let mut r = i + 1;
            while r < bytes.len() && bytes[r].is_ascii_whitespace() { r += 1; }
            let mut rend = r;
            while rend < bytes.len() {
                let c = bytes[rend] as char;
                if c.is_ascii_alphanumeric() || c == '_' || c == '"' { rend += 1; } else { break; }
            }
            if r <= rend { let right = &line[r..rend]; return Some((left, right)); }
            return None;
        }
        i += 1;
    }
    None
}

fn peek_ident_left(s: &str, pos: usize) -> bool {
    // scan left for first non-space token end, then back to token start
    let bytes = s.as_bytes();
    if pos == 0 { return false; }
    let mut i = pos;
    // skip spaces
    while i > 0 && bytes[i - 1].is_ascii_whitespace() { i -= 1; }
    if i == 0 { return false; }
    // now consume identifier chars backwards
    let mut j = i;
    while j > 0 {
        let c = bytes[j - 1] as char;
        if c.is_ascii_alphanumeric() || c == '_' { j -= 1; } else { break; }
    }
    if j == i { return false; }
    // ensure not starting with digit only (avoid numeric literal)
    let tok = &s[j..i];
    !tok.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}
fn peek_ident_right(s: &str, pos: usize) -> bool {
    let bytes = s.as_bytes();
    let mut i = pos;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
    if i >= bytes.len() { return false; }
    let mut j = i;
    while j < bytes.len() {
        let c = bytes[j] as char;
        if c.is_ascii_alphanumeric() || c == '_' { j += 1; } else { break; }
    }
    if j == i { return false; }
    let tok = &s[i..j];
    !tok.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}
