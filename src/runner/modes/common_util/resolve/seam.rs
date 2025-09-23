/// Log tail of inlined prelude chunk for seam inspection.
pub fn log_inlined_tail(path_key: &str, inlined_text: &str, seam_dbg: bool) {
    if !seam_dbg { return; }
    let tail = inlined_text
        .chars()
        .rev()
        .take(120)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    eprintln!(
        "[using][seam][inlined] {} tail=<<<{}>>>",
        path_key,
        tail.replace('\n', "\\n")
    );
}

/// Log the seam between prelude and body for quick visual diff.
pub fn log_prelude_body_seam(prelude_clean: &str, body: &str, seam_dbg: bool) {
    if !seam_dbg { return; }
    let tail = prelude_clean
        .chars()
        .rev()
        .take(160)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let head = body.chars().take(160).collect::<String>();
    eprintln!("[using][seam] prelude_tail=<<<{}>>>", tail.replace('\n', "\\n"));
    eprintln!("[using][seam] body_head   =<<<{}>>>", head.replace('\n', "\\n"));
}

/// Apply optional seam safety: append missing '}' for unmatched '{' in prelude
/// When `trace` is true, emits a short note with delta count.
pub fn fix_prelude_braces_if_enabled(prelude_clean: &str, combined: &mut String, trace: bool) {
    if !crate::config::env::resolve_fix_braces() {
        return;
    }
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
        for _ in 0..delta {
            combined.push('}');
            combined.push('\n');
        }
    }
}
