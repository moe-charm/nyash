//! Minimal hostcall registry (v0): classify symbols as read-only or mutating

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostcallKind {
    ReadOnly,
    Mutating,
}

#[derive(Debug, Default)]
struct Registry {
    ro: HashSet<String>,
    mu: HashSet<String>,
    // Allow multiple signatures per symbol (overloads)
    sig: HashMap<String, Vec<Signature>>,
}

static REG: OnceCell<RwLock<Registry>> = OnceCell::new();

fn ensure_default() {
    if REG.get().is_some() {
        return;
    }
    let mut r = Registry::default();
    // Read-only defaults
    for s in [
        "nyash.array.len_h",
        "nyash.string.len_h",
        "nyash.any.length_h",
        "nyash.any.is_empty_h",
        "nyash.map.size_h",
        "nyash.map.get_h",
        "nyash.map.has_h",
        "nyash.string.charCodeAt_h",
        "nyash.string.concat_hh",
        "nyash.string.eq_hh",
        "nyash.string.lt_hh",
        "nyash.array.get_h",
    ] {
        r.ro.insert(s.to_string());
    }
    // Mutating defaults
    for s in ["nyash.array.push_h", "nyash.array.set_h", "nyash.map.set_h"] {
        r.mu.insert(s.to_string());
    }
    // Signatures (v0): register known symbols with simple arg/ret kinds
    // math.* thin bridge: f64 signatures only (allow when args match exactly)
    r.sig
        .entry("nyash.math.sin".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::F64],
            ret: ArgKind::F64,
        });
    r.sig
        .entry("nyash.math.cos".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::F64],
            ret: ArgKind::F64,
        });
    r.sig
        .entry("nyash.math.abs".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::F64],
            ret: ArgKind::F64,
        });
    r.sig
        .entry("nyash.math.min".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::F64, ArgKind::F64],
            ret: ArgKind::F64,
        });
    r.sig
        .entry("nyash.math.max".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::F64, ArgKind::F64],
            ret: ArgKind::F64,
        });
    // Collections (handle-based)
    // Map get: support both integer and handle keys (overload)
    r.sig
        .entry("nyash.map.get_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::I64],
            ret: ArgKind::Handle,
        });
    r.sig
        .entry("nyash.map.get_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::Handle],
            ret: ArgKind::Handle,
        });
    r.sig
        .entry("nyash.map.size_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle],
            ret: ArgKind::I64,
        });
    r.sig
        .entry("nyash.array.get_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::I64],
            ret: ArgKind::Handle,
        });
    r.sig
        .entry("nyash.array.len_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle],
            ret: ArgKind::I64,
        });
    // String helpers
    r.sig
        .entry("nyash.string.len_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle],
            ret: ArgKind::I64,
        });
    r.sig
        .entry("nyash.string.charCodeAt_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::I64],
            ret: ArgKind::I64,
        });
    r.sig
        .entry("nyash.string.concat_hh".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::Handle],
            ret: ArgKind::Handle,
        });
    r.sig
        .entry("nyash.semantics.add_hh".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::Handle],
            ret: ArgKind::Handle,
        });
    r.sig
        .entry("nyash.string.eq_hh".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::Handle],
            ret: ArgKind::I64,
        });
    r.sig
        .entry("nyash.string.lt_hh".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::Handle],
            ret: ArgKind::I64,
        });
    // Any helpers (length/is_empty)
    r.sig
        .entry("nyash.any.length_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle],
            ret: ArgKind::I64,
        });
    r.sig
        .entry("nyash.any.is_empty_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle],
            ret: ArgKind::I64,
        });
    // Map.has(handle, i64) -> i64(0/1)
    r.sig
        .entry("nyash.map.has_h".to_string())
        .or_default()
        .push(Signature {
            args: vec![ArgKind::Handle, ArgKind::I64],
            ret: ArgKind::I64,
        });
    let _ = REG.set(RwLock::new(r));
}

pub fn classify(symbol: &str) -> HostcallKind {
    ensure_default();
    if let Some(lock) = REG.get() {
        if let Ok(g) = lock.read() {
            if g.ro.contains(symbol) {
                return HostcallKind::ReadOnly;
            }
            if g.mu.contains(symbol) {
                return HostcallKind::Mutating;
            }
        }
    }
    // Default to read-only to be permissive in v0
    HostcallKind::ReadOnly
}

pub fn add_readonly(symbol: &str) {
    ensure_default();
    if let Some(lock) = REG.get() {
        if let Ok(mut w) = lock.write() {
            w.ro.insert(symbol.to_string());
        }
    }
}
pub fn add_mutating(symbol: &str) {
    ensure_default();
    if let Some(lock) = REG.get() {
        if let Ok(mut w) = lock.write() {
            w.mu.insert(symbol.to_string());
        }
    }
}
pub fn set_from_csv(ro_csv: &str, mu_csv: &str) {
    ensure_default();
    if let Some(lock) = REG.get() {
        if let Ok(mut w) = lock.write() {
            w.ro.clear();
            w.mu.clear();
            for s in ro_csv.split(',') {
                let t = s.trim();
                if !t.is_empty() {
                    w.ro.insert(t.to_string());
                }
            }
            for s in mu_csv.split(',') {
                let t = s.trim();
                if !t.is_empty() {
                    w.mu.insert(t.to_string());
                }
            }
        }
    }
}
pub fn snapshot() -> (Vec<String>, Vec<String>) {
    ensure_default();
    if let Some(lock) = REG.get() {
        if let Ok(g) = lock.read() {
            let mut ro: Vec<String> = g.ro.iter().cloned().collect();
            ro.sort();
            let mut mu: Vec<String> = g.mu.iter().cloned().collect();
            mu.sort();
            return (ro, mu);
        }
    }
    (Vec::new(), Vec::new())
}

// ==== Signature (v0 scaffolding) ====
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    I64,
    F64,
    Handle,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub args: Vec<ArgKind>,
    pub ret: ArgKind,
}

fn parse_kind(s: &str) -> Option<ArgKind> {
    match s.trim().to_ascii_lowercase().as_str() {
        "i64" | "int" | "integer" => Some(ArgKind::I64),
        "f64" | "float" => Some(ArgKind::F64),
        "handle" | "h" => Some(ArgKind::Handle),
        _ => None,
    }
}

pub fn set_signature_csv(symbol: &str, args_csv: &str, ret_str: &str) -> bool {
    ensure_default();
    let mut ok = true;
    let parsed: Vec<Option<ArgKind>> = args_csv
        .split(',')
        .filter(|t| !t.trim().is_empty())
        .map(|t| parse_kind(t))
        .collect();
    let mut args: Vec<ArgKind> = Vec::new();
    for p in parsed {
        if let Some(k) = p {
            args.push(k)
        } else {
            ok = false;
        }
    }
    let ret = match parse_kind(ret_str) {
        Some(k) => k,
        None => {
            ok = false;
            ArgKind::I64
        }
    };
    if !ok {
        return false;
    }
    let sig = Signature { args, ret };
    if let Some(lock) = REG.get() {
        if let Ok(mut w) = lock.write() {
            w.sig.entry(symbol.to_string()).or_default().push(sig);
            return true;
        }
    }
    false
}

/// Check observed args against a registered signature.
/// - If no signature is registered for the symbol, returns Ok(()) to be permissive in v0.
/// - Returns Err("sig_mismatch") when arg length or kinds differ.
pub fn check_signature(symbol: &str, observed_args: &[ArgKind]) -> Result<(), &'static str> {
    ensure_default();
    if let Some(lock) = REG.get() {
        if let Ok(g) = lock.read() {
            if let Some(sigs) = g.sig.get(symbol) {
                let cfg_now = crate::jit::config::current();
                let relax = cfg_now.relax_numeric || cfg_now.native_f64;
                // Match against any one of the overload signatures
                'outer: for sig in sigs.iter() {
                    if sig.args.len() != observed_args.len() {
                        continue;
                    }
                    for (expected, observed) in sig.args.iter().zip(observed_args.iter()) {
                        if expected == observed {
                            continue;
                        }
                        // v0 coercion: allow I64 → F64 only when relaxed numeric is enabled
                        if relax
                            && matches!(expected, ArgKind::F64)
                            && matches!(observed, ArgKind::I64)
                        {
                            continue;
                        }
                        // Mismatch for this candidate signature
                        continue 'outer;
                    }
                    // All args matched for this signature
                    return Ok(());
                }
                // No overload matched
                return Err("sig_mismatch");
            }
        }
    }
    Ok(())
}
