//! Extern interface registry (env.*) for diagnostics and optional slotting
//!
//! 目的: ExternCallの未登録/未対応時に候補提示やSTRICT診断を改善する。

use once_cell::sync::Lazy;

#[derive(Clone, Copy, Debug)]
pub struct ExternSpec {
    pub iface: &'static str,
    pub method: &'static str,
    pub min_arity: u8,
    pub max_arity: u8,
    pub slot: Option<u16>,
}

static EXTERNS: Lazy<Vec<ExternSpec>> = Lazy::new(|| vec![
    // console
    ExternSpec { iface: "env.console", method: "log", min_arity: 1, max_arity: 1, slot: Some(10) },
    // debug
    ExternSpec { iface: "env.debug", method: "trace", min_arity: 1, max_arity: 255, slot: Some(11) },
    // runtime
    ExternSpec { iface: "env.runtime", method: "checkpoint", min_arity: 0, max_arity: 0, slot: Some(12) },
    // future (scaffold)
    ExternSpec { iface: "env.future", method: "new", min_arity: 1, max_arity: 1, slot: Some(20) },
    ExternSpec { iface: "env.future", method: "birth", min_arity: 1, max_arity: 1, slot: Some(20) },
    ExternSpec { iface: "env.future", method: "set", min_arity: 2, max_arity: 2, slot: Some(21) },
    ExternSpec { iface: "env.future", method: "await", min_arity: 1, max_arity: 1, slot: Some(22) },
]);

pub fn resolve(iface: &str, method: &str) -> Option<ExternSpec> {
    EXTERNS.iter().copied().find(|e| e.iface == iface && e.method == method)
}

pub fn known_for_iface(iface: &str) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = EXTERNS.iter().filter(|e| e.iface == iface).map(|e| e.method).collect();
    v.sort(); v.dedup(); v
}

pub fn all_ifaces() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = EXTERNS.iter().map(|e| e.iface).collect();
    v.sort(); v.dedup(); v
}

/// Resolve slot id for an extern call (if assigned)
pub fn resolve_slot(iface: &str, method: &str) -> Option<u16> {
    resolve(iface, method).and_then(|s| s.slot)
}

/// Check arity against registry; returns Ok or an explanatory error string
pub fn check_arity(iface: &str, method: &str, argc: usize) -> Result<(), String> {
    if let Some(s) = resolve(iface, method) {
        if argc as u8 >= s.min_arity && argc as u8 <= s.max_arity { Ok(()) }
        else { Err(format!("arity {} out of range {}..{}", argc, s.min_arity, s.max_arity)) }
    } else { Err("unknown extern".to_string()) }
}
