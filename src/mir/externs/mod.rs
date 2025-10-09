//! Externs Registry (thin box) — centralized list of known extern endpoints.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternSig {
    pub argc: usize,
}

fn registry() -> &'static HashMap<&'static str, ExternSig> {
    use std::sync::OnceLock;
    static REG: OnceLock<HashMap<&'static str, ExternSig>> = OnceLock::new();
    REG.get_or_init(|| {
        let mut m = HashMap::new();
        // Console/logging
        m.insert("env.console.log", ExternSig { argc: 1 });
        m.insert("env.console.warn", ExternSig { argc: 1 });
        m.insert("env.console.error", ExternSig { argc: 1 });
        // Futures (minimal)
        m.insert("env.future.new", ExternSig { argc: 1 });
        m.insert("env.future.set", ExternSig { argc: 2 });
        m.insert("env.future.await", ExternSig { argc: 1 });
        // Local env access (minimal)
        m.insert("env.local.get", ExternSig { argc: 1 });
        // Ops (equality)
        m.insert("nyrt.ops.op_eq", ExternSig { argc: 2 });
        // Debug trace
        m.insert("env.debug.trace", ExternSig { argc: 1 });
        m
    })
}

pub fn is_known(name: &str) -> bool { registry().contains_key(name) }

pub fn get(name: &str) -> Option<ExternSig> { registry().get(name).copied() }

// Compatibility facade expected by various modules
pub mod registry {
    use super::{registry as inner, ExternSig};
    use crate::mir::EffectMask;
    use std::path::Path;

    pub struct Facade;
    impl Facade {
        pub fn get(&self, iface: &str, method: &str) -> Option<ExternSig> {
            let name = format!("{}.{}", iface, method);
            inner().get(name.as_str()).copied()
        }
    }

    // Return a lightweight facade object
    pub fn registry() -> Facade { Facade }

    // Optional: effect hints for externs (MVP: IO for console; PURE otherwise)
    pub fn effects_for(iface: &str, method: &str) -> Option<EffectMask> {
        let name = format!("{}.{}", iface, method);
        if name.starts_with("env.console.") || name == "env.debug.trace" {
            Some(EffectMask::PURE.add(crate::mir::Effect::Io))
        } else if name == "env.future.new" || name == "env.future.set" || name == "env.future.await" {
            Some(EffectMask::PURE.add(crate::mir::Effect::Io))
        } else {
            None
        }
    }

    // Export a minimal JSON spec for harness tools (best-effort; ignore errors)
    pub fn export_json(path: &Path) -> Result<(), String> {
        let mut arr = Vec::new();
        for (name, sig) in inner().iter() {
            arr.push(serde_json::json!({ "name": name, "argc": sig.argc }));
        }
        let root = serde_json::json!({ "externs": arr });
        std::fs::write(path, serde_json::to_string_pretty(&root).unwrap())
            .map_err(|e| format!("write externs spec: {}", e))
    }
}
