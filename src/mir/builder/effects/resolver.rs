use crate::mir::{Effect, EffectMask};
use crate::mir::externs::registry as extreg;
use std::collections::HashMap;

/// Effects決定の最小実装（テーブル駆動）。
pub struct EffectResolverBox {
    trace: bool,
}

impl EffectResolverBox {
    pub fn new(trace: bool) -> Self { Self { trace } }

    fn extern_table(&self) -> HashMap<(&'static str, &'static str), EffectMask> {
        // Registry 優先へ移行中。最低限のIO系だけフォールバックを持つ。
        let mut m: HashMap<(&'static str, &'static str), EffectMask> = HashMap::new();
        m.insert(("env.console", "log"), EffectMask::IO);
        m
    }

    fn method_table(&self) -> HashMap<(&'static str, &'static str), EffectMask> {
        use EffectMask as EM;
        let mut m: HashMap<(&'static str, &'static str), EffectMask> = HashMap::new();
        // Minimal builtins
        m.insert(("ArrayBox", "get"), EM::READ);
        m.insert(("ArrayBox", "length"), EM::READ);
        m.insert(("ArrayBox", "size"), EM::READ);
        m.insert(("ArrayBox", "set"), EM::READ.add(Effect::WriteHeap));
        m.insert(("ArrayBox", "push"), EM::READ.add(Effect::WriteHeap));
        m.insert(("ArrayBox", "pop"), EM::READ.add(Effect::WriteHeap));
        m
    }

    pub fn resolve_extern(&self, iface: &str, method: &str) -> Option<EffectMask> {
        let eff = extreg::effects_for(iface, method)
            .or_else(|| self.extern_table().get(&(iface, method)).copied());
        if self.trace {
            eprintln!("[EffectResolver] extern {}.{} -> {:?}", iface, method, eff);
        }
        eff
    }

    pub fn resolve_method(&self, box_name: &str, method: &str) -> Option<EffectMask> {
        let t = self.method_table();
        let eff = t.get(&(box_name, method)).copied();
        if self.trace {
            eprintln!("[EffectResolver] method {}.{} -> {:?}", box_name, method, eff);
        }
        eff
    }
}
