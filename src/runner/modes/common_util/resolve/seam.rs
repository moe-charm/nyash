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

// Legacy brace fix function removed (Phase 15 cleanup)
// AST-based integration handles syntax errors properly without text-level hacks
