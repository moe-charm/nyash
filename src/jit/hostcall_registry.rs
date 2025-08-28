//! Minimal hostcall registry (v0): classify symbols as read-only or mutating

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostcallKind { ReadOnly, Mutating }

pub fn classify(symbol: &str) -> HostcallKind {
    match symbol {
        // Read-only (safe under read_only policy)
        "nyash.array.len_h" | "nyash.any.length_h" | "nyash.any.is_empty_h" |
        "nyash.map.size_h" | "nyash.map.get_h" | "nyash.string.charCodeAt_h" |
        "nyash.array.get_h" => HostcallKind::ReadOnly,
        // Mutating
        "nyash.array.push_h" | "nyash.array.set_h" | "nyash.map.set_h" => HostcallKind::Mutating,
        // Default to read-only to be permissive in v0
        _ => HostcallKind::ReadOnly,
    }
}

