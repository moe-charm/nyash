//! MethodIdsBox — Special method names and default IDs
//!
//! Responsibility
//! - Define special method set (birth/toString/fini)
//! - Provide default method IDs when not specified in specs

#[inline]
pub fn is_special(method_name: &str) -> bool {
    matches!(method_name, "birth" | "fini" | "toString")
}

#[inline]
pub fn default_id(method_name: &str) -> Option<u32> {
    match method_name {
        "birth" => Some(1),
        "toString" => Some(100),
        "fini" => Some(999),
        _ => None,
    }
}

