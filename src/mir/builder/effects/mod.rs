/*!
 * Effects subsystem (resolver; Phase-15 minimal)
 *
 * 目的: Effects 決定の散在を解消し、単一入口でのテーブル解決を提供する。
 * 既定OFF（NYASH_USE_EFFECT_RESOLVER=1 で有効）。Unknown は既存ロジックに委譲。
 */

mod resolver;
pub use resolver::EffectResolverBox;

use crate::mir::{Callee, EffectMask, MirInstruction};
use std::sync::OnceLock;

fn use_resolver_enabled() -> bool {
    matches!(std::env::var("NYASH_USE_EFFECT_RESOLVER").ok().as_deref(), Some("1"|"true"|"on"))
}

fn verify_effects_enabled() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| matches!(
        std::env::var("NYASH_VERIFY_EFFECTS").ok().as_deref(),
        Some("1" | "true" | "on")
    ))
}

/// Callee に対する効果を解決（resolver 使用時のみ）。None はフォールバックを意味する。
pub fn resolve_effects_for_callee(callee: &Callee) -> Option<EffectMask> {
    if !use_resolver_enabled() { return None; }
    let trace = matches!(std::env::var("NYASH_EFFECT_TRACE").ok().as_deref(), Some("1"|"true"|"on"));
    let r = EffectResolverBox::new(trace);
    match callee {
        Callee::Extern(name) => {
            // full name: iface.method
            let parts: Vec<&str> = name.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                return r.resolve_extern(parts[1], parts[0]);
            }
            None
        }
        Callee::Method { box_name, method, .. } => {
            r.resolve_method(box_name, method)
        }
        _ => None,
    }
}

/// 軽量検証: Call/BoxCall で PURE 判定が混入した場合に警告する。
pub fn verify_instruction_effects(inst: &MirInstruction) {
    if !verify_effects_enabled() {
        return;
    }
    let (kind, details, effects) = match inst {
        MirInstruction::Call { effects, func, .. } => ("Call", format!("func=%{}", func), effects),
        MirInstruction::BoxCall { effects, box_val, method, .. } => (
            "BoxCall",
            format!("recv=%{} method={}", box_val, method),
            effects,
        ),
        // ExternCall retired — external calls are represented as Call with callee=Extern
        _ => return,
    };
    if effects.is_pure() {
        eprintln!(
            "[EffectVerifier] WARN: {} {} emitted with PURE effects (0x{:04x}). Check resolver/legacy tables.",
            kind,
            details,
            effects.bits()
        );
    }
}
