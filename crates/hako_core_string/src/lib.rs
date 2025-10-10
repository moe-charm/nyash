//! hako_core_string — shared string algorithms for builtin and plugins
//!
//! Byte‑semantics by default to match current VM/plugin behavior.

/// Return length in bytes (UTF‑8 byte length)
pub fn length_bytes(s: &str) -> i64 {
    s.len() as i64
}

/// isEmpty: true when byte length is zero
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// indexOf with optional starting position (byte index; negatives clamped to 0)
pub fn index_of(s: &str, needle: &str, from: i64) -> i64 {
    let start = from.max(0) as usize;
    if needle.is_empty() {
        return 0; // match current plugin/VM semantics
    }
    if start >= s.len() {
        return -1;
    }
    match s[start..].find(needle) {
        Some(i) => (start + i) as i64,
        None => -1,
    }
}

/// lastIndexOf with optional bound (inclusive). Empty needle returns min(len, from.clamp(0,len)).
pub fn last_index_of(s: &str, needle: &str, from: i64) -> i64 {
    if needle.is_empty() {
        let pos = (s.len() as i64).min(from.max(0));
        return pos;
    }
    let bound = from.max(0) as usize;
    let bound = bound.min(s.len());
    let slice = &s[..bound];
    match slice.rfind(needle) {
        Some(i) => i as i64,
        None => -1,
    }
}

/// substring by byte indices [start, end) clamped to [0,len]
pub fn substring_bytes(s: &str, start: i64, end: i64) -> String {
    let len = s.len() as i64;
    let i0 = start.max(0).min(len) as usize;
    let i1 = end.max(0).min(len) as usize;
    if i0 > i1 {
        return String::new();
    }
    let bytes = s.as_bytes();
    String::from_utf8_lossy(&bytes[i0..i1]).to_string()
}

/// charAt by byte index; returns empty string when out of range
pub fn char_at_byte(s: &str, idx: i64) -> String {
    let i = idx.max(0) as usize;
    if i >= s.len() {
        return String::new();
    }
    let bytes = s.as_bytes();
    String::from_utf8_lossy(&bytes[i..i + 1]).to_string()
}

/// replace all occurrences of `from` with `to`
pub fn replace_all(s: &str, from: &str, to: &str) -> String {
    if from.is_empty() {
        return s.to_string();
    }
    s.replace(from, to)
}
