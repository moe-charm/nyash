//! PreLexBox — shared pre-lexical normalization for sugar-sensitive forms.
//!
//! Goals (sugar ON only):
//! - Raw strings: r"…", r#"…"#, r##"…"## → normalize into regular strings (quotes/backslashes escaped).
//! - Numeric separators: collapse underscores in decimal/float literals (e.g., 1_000_000 → 1000000).
//!
//! Notes:
//! - Runs before the tokenizer to keep all runner modes consistent (VM/LLVM/PyVM).
//! - Comment and normal-string contents remain untouched; only raw strings are rewritten.
//! - Keep changes conservative and reversible; when sugar is OFF, return input unchanged.

pub fn sugar_enabled() -> bool {
    // Default ON unless explicitly disabled
    match std::env::var("NYASH_SYNTAX_SUGAR_LEVEL").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            lv != "off" && lv != "0" && lv != "false"
        }
        None => true,
    }
}

/// Normalize source for sugar-sensitive tokens.
pub fn prelex_normalize(src: &str) -> String {
    if !sugar_enabled() {
        return src.to_string();
    }

    let mut out = String::with_capacity(src.len());
    let mut it = src.chars().peekable();
    let mut in_line = false;
    let mut in_block = false;
    let mut in_str = false; // normal string only

    while let Some(c) = it.next() {
        // Respect existing comment/string contexts
        if in_line {
            out.push(c);
            if c == '\n' { in_line = false; }
            continue;
        }
        if in_block {
            out.push(c);
            if c == '*' && matches!(it.peek(), Some('/')) { out.push('/'); it.next(); in_block = false; }
            continue;
        }
        if in_str {
            out.push(c);
            if c == '\\' { if let Some(nc) = it.next() { out.push(nc); } continue; }
            if c == '"' { in_str = false; }
            continue;
        }

        match c {
            // Raw strings: r"…" or r##"…"## → rewrite to normal string.
            'r' if matches!(it.peek(), Some('"')) || matches!(it.peek(), Some('#')) => {
                // Count hashes after 'r'
                let mut tmp = String::new();
                let mut look = it.clone();
                let mut hashes = 0usize;
                while let Some('#') = look.peek().copied() {
                    hashes += 1;
                    look.next();
                }
                // Next must be '"'
                if look.peek().copied() != Some('"') {
                    // Not a raw string; emit 'r' and continue
                    out.push('r');
                    continue;
                }
                // Commit consumption: we had 'r' already; now eat up to and including opening '"'
                for _ in 0..hashes { let _ = it.next(); } // consume '#'
                let _ = it.next(); // consume opening '"'

                // Read until closing '"' followed by the same number of '#'
                loop {
                    match it.next() {
                        None => break, // unterminated; emit what we have (fail-soft)
                        Some('"') => {
                            // verify hashes
                            let mut ok = true;
                            for _ in 0..hashes {
                                if it.peek().copied() != Some('#') { ok = false; break; }
                                // stage-verify only; do not consume here
                            }
                            if ok {
                                // emit as normal string now
                                out.push('"');
                                for ch in tmp.chars() {
                                    match ch {
                                        '"' | '\\' => { out.push('\\'); out.push(ch); }
                                        _ => out.push(ch),
                                    }
                                }
                                out.push('"');
                                // Consume the closing '"' hashes
                                for _ in 0..hashes { let _ = it.next(); }
                                break;
                            } else {
                                tmp.push('"');
                            }
                        }
                        Some(ch) => tmp.push(ch),
                    }
                }
            }
            // Enter comments/strings (normal strings only)
            '/' => {
                match it.peek() { Some('/') => { out.push('/'); out.push('/'); it.next(); in_line = true; }, Some('*') => { out.push('/'); out.push('*'); it.next(); in_block = true; }, _ => out.push('/') }
            }
            '"' => { in_str = true; out.push('"'); }

            // Numeric separators: collapse sequences like 1_000_000 and 3.141_592
            d if d.is_ascii_digit() => {
                out.push(d);
                loop {
                    match it.peek().copied() {
                        Some('_') => { let _ = it.next(); /* skip underscore */ }
                        Some(nd) if nd.is_ascii_digit() => { out.push(nd); let _ = it.next(); }
                        Some('.') => {
                            // allow '.' only when followed by a digit
                            let mut look = it.clone();
                            let _ = look.next(); // consume '.'
                            if matches!(look.peek(), Some(nd) if nd.is_ascii_digit()) {
                                out.push('.');
                                let _ = it.next(); // commit '.'
                                // continue reading fractional part
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
            }
            _ => out.push(c),
        }
    }

    out
}
