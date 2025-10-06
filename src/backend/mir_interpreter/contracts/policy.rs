//! LifecycleContractsBox — central contract/log gating for New/birth
//!
//! Provides small helpers used by interpreter lifecycle to decide log emission.

pub fn contracts_enabled() -> bool { crate::config::env::check_contracts() }

pub fn emit_new(class: &str, argc: usize, key: i64) {
    if !contracts_enabled() { return; }
    eprintln!(r#"{{"kind":"contracts_newbox","class":"{}","argc":{},"key":{}}}"#, class, argc, key);
}

pub fn emit_birth(seen_new: bool, duplicate: bool, argc_new: usize, argc_birth: usize, key: i64) {
    if !contracts_enabled() { return; }
    eprintln!(
        r#"{{"kind":"contracts_birth","seen_new":{},"duplicate":{},"argc_new":{},"argc_birth":{},"argc_match":{},"key":{}}}"#,
        if seen_new { 1 } else { 0 },
        if duplicate { 1 } else { 0 },
        argc_new,
        argc_birth,
        if argc_new == argc_birth { 1 } else { 0 },
        key
    );
}
