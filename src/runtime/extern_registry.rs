//! Extern interface registry (env.*) for diagnostics and optional slotting
//!
//! 目的: ExternCallの未登録/未対応時に候補提示やSTRICT診断を改善する。

use once_cell::sync::Lazy;

#[derive(Clone, Copy, Debug)]
pub struct ExternSpec { pub iface: &'static str, pub method: &'static str, pub min_arity: u8, pub max_arity: u8 }

static EXTERNS: Lazy<Vec<ExternSpec>> = Lazy::new(|| vec![
    // console
    ExternSpec { iface: "env.console", method: "log", min_arity: 1, max_arity: 1 },
    // debug
    ExternSpec { iface: "env.debug", method: "trace", min_arity: 1, max_arity: 255 },
    // runtime
    ExternSpec { iface: "env.runtime", method: "checkpoint", min_arity: 0, max_arity: 0 },
    // future (scaffold)
    ExternSpec { iface: "env.future", method: "new", min_arity: 1, max_arity: 1 },
    ExternSpec { iface: "env.future", method: "birth", min_arity: 1, max_arity: 1 },
    ExternSpec { iface: "env.future", method: "set", min_arity: 2, max_arity: 2 },
    ExternSpec { iface: "env.future", method: "await", min_arity: 1, max_arity: 1 },
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

